use std::{collections::HashSet, net::SocketAddr, path::PathBuf, sync::OnceLock};

use clap::Parser;
use eyre::{Context, Result};
use serde::Serialize;
use sonar::Properties;
use tokio_stream::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const IMPORT_FILETYPES: &[&str] = &["flac", "mp3", "ogg", "opus", "wav"];

static SERVER_ENDPOINT: OnceLock<String> = OnceLock::new();
static OUTPUT_JSON: OnceLock<bool> = OnceLock::new();

#[derive(Debug, Parser)]
struct ListParams {
    #[clap(long)]
    offset: Option<u32>,

    #[clap(long)]
    limit: Option<u32>,
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, default_value = "http://localhost:3000", env = "SONAR_SERVER")]
    server: String,

    #[clap(long, env = "SONAR_JSON")]
    json: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Login(LoginArgs),
    Logout(LogoutArgs),
    Artist(ArtistArgs),
    Album(AlbumArgs),
    Track(TrackArgs),
    Playlist(PlaylistArgs),
    Sync(SyncArgs),
    Admin(AdminArgs),
    Import(ImportArgs),
    Metadata(MetadataArgs),
    Server(ServerArgs),
    Extractor(ExtractorArgs),
}

// Types used to generate json
#[derive(Debug, Serialize)]
struct User {
    id: String,
    username: String,
    avatar: Option<String>,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.username)
    }
}

impl From<sonar_grpc::User> for User {
    fn from(value: sonar_grpc::User) -> Self {
        Self {
            id: value.user_id,
            username: value.username,
            avatar: value.avatar_id,
        }
    }
}

#[derive(Debug, Serialize)]
struct Artist {
    id: String,
    name: String,
    album_count: u32,
    listen_count: u32,
    coverart: Option<String>,
    properties: Properties,
}

impl std::fmt::Display for Artist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.name)
    }
}

impl From<sonar_grpc::Artist> for Artist {
    fn from(value: sonar_grpc::Artist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            album_count: value.album_count,
            listen_count: value.listen_count,
            coverart: value.coverart_id,
            properties: properties_from_pb(value.properties),
        }
    }
}

#[derive(Debug, Serialize)]
struct Album {
    id: String,
    name: String,
    track_count: u32,
    duration: Option<u32>,
    listen_count: u32,
    artist: String,
    coverart: Option<String>,
    properties: Properties,
}

impl std::fmt::Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.name)
    }
}

impl From<sonar_grpc::Album> for Album {
    fn from(value: sonar_grpc::Album) -> Self {
        Self {
            id: value.id,
            name: value.name,
            track_count: value.track_count,
            duration: value.duration.map(|x| x.seconds as u32),
            listen_count: value.listen_count,
            artist: value.artist_id,
            coverart: value.coverart_id,
            properties: properties_from_pb(value.properties),
        }
    }
}

#[derive(Debug, Serialize)]
struct Track {
    id: String,
    name: String,
    album: String,
    artist: String,
    duration: Option<u32>,
    listen_count: u32,
    properties: Properties,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.name)
    }
}

impl From<sonar_grpc::Track> for Track {
    fn from(value: sonar_grpc::Track) -> Self {
        Self {
            id: value.id,
            name: value.name,
            album: value.album_id,
            artist: value.artist_id,
            duration: value.duration.map(|x| x.seconds as u32),
            listen_count: value.listen_count,
            properties: properties_from_pb(value.properties),
        }
    }
}

#[derive(Debug, Serialize)]
struct Playlist {
    id: String,
    name: String,
    track_count: u32,
    duration: Option<u32>,
    properties: Properties,
}

impl std::fmt::Display for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.name)
    }
}

impl From<sonar_grpc::Playlist> for Playlist {
    fn from(value: sonar_grpc::Playlist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            track_count: value.track_count,
            duration: value.duration.map(|x| x.seconds as u32),
            properties: properties_from_pb(value.properties),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    if std::matches!(args.command, Command::Server(_)) {
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("sonar")
            .install_simple()?;

        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        tracing_subscriber::registry()
            .with(opentelemetry)
            .with(tracing_subscriber::fmt::layer())
            .try_init()?;
    } else {
        tracing_subscriber::fmt::init();
    }

    SERVER_ENDPOINT.set(args.server.clone()).unwrap();
    OUTPUT_JSON.set(args.json).unwrap();

    match args.command {
        Command::Login(args) => cmd_login(args).await?,
        Command::Logout(args) => cmd_logout(args).await?,
        Command::Artist(cargs) => match cargs.command {
            ArtistCommand::List(cargs) => cmd_artist_list(cargs).await?,
            ArtistCommand::Create(cargs) => cmd_artist_create(cargs).await?,
            ArtistCommand::Update(cargs) => cmd_artist_update(cargs).await?,
            ArtistCommand::Delete(cargs) => cmd_artist_delete(cargs).await?,
        },
        Command::Album(cargs) => match cargs.command {
            AlbumCommand::List(cargs) => cmd_album_list(cargs).await?,
            AlbumCommand::Create(cargs) => cmd_album_create(cargs).await?,
            AlbumCommand::Update(cargs) => cmd_album_update(cargs).await?,
            AlbumCommand::Delete(cargs) => cmd_album_delete(cargs).await?,
        },
        Command::Track(cargs) => match cargs.command {
            TrackCommand::List(cargs) => cmd_track_list(cargs).await?,
            TrackCommand::Create(cargs) => cmd_track_create(cargs).await?,
            TrackCommand::Update(cargs) => cmd_track_update(cargs).await?,
            TrackCommand::Delete(cargs) => cmd_track_delete(cargs).await?,
            TrackCommand::Download(cargs) => cmd_track_download(cargs).await?,
        },
        Command::Playlist(cargs) => match cargs.command {
            PlaylistCommand::List(cargs) => cmd_playlist_list(cargs).await?,
            PlaylistCommand::Create(cargs) => cmd_playlist_create(cargs).await?,
            PlaylistCommand::Delete(cargs) => cmd_playlist_delete(cargs).await?,
            PlaylistCommand::Add(cargs) => cmd_playlist_add(cargs).await?,
            PlaylistCommand::Remove(cargs) => cmd_playlist_remove(cargs).await?,
        },
        Command::Sync(cargs) => cmd_sync(cargs).await?,
        Command::Admin(cargs) => match cargs.command {
            AdminCommand::User(cargs) => match cargs.command {
                AdminUserCommand::List(cargs) => cmd_admin_user_list(cargs).await?,
                AdminUserCommand::Create(cargs) => cmd_admin_user_create(cargs).await?,
                AdminUserCommand::Update(cargs) => cmd_admin_user_update(cargs).await?,
                AdminUserCommand::Delete(cargs) => cmd_admin_user_delete(cargs).await?,
            },
            AdminCommand::Playlist(cargs) => match cargs.command {
                AdminPlaylistCommand::List(cargs) => cmd_admin_playlist_list(cargs).await?,
                AdminPlaylistCommand::Create(cargs) => cmd_admin_playlist_create(cargs).await?,
                AdminPlaylistCommand::Update(cargs) => cmd_admin_playlist_update(cargs).await?,
                AdminPlaylistCommand::Delete(cargs) => cmd_admin_playlist_delete(cargs).await?,
            },
        },
        Command::Import(cargs) => cmd_import(cargs).await?,
        Command::Metadata(margs) => match margs.command {
            MetadataCommand::Album(cargs) => cmd_metadata_album(cargs, margs.view).await?,
            MetadataCommand::AlbumTracks(cargs) => {
                cmd_metadata_album_tracks(cargs, margs.view).await?
            }
        },
        Command::Server(cargs) => cmd_server(cargs).await?,
        Command::Extractor(cargs) => cmd_extractor(cargs).await?,
    }

    Ok(())
}

async fn create_client() -> Result<sonar_grpc::Client> {
    let endpoint = SERVER_ENDPOINT.get().unwrap();
    sonar_grpc::client(endpoint)
        .await
        .with_context(|| format!("connecting to grpc server at {}", endpoint))
}

#[derive(Debug, Parser)]
struct LoginArgs {
    username: String,

    password: String,
}

async fn cmd_login(args: LoginArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .user_login(sonar_grpc::UserLoginRequest {
            username: args.username,
            password: args.password,
        })
        .await?;
    let token = response.into_inner().token;
    auth_token_write(&token).await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct LogoutArgs {}

async fn cmd_logout(_args: LogoutArgs) -> Result<()> {
    let mut client = create_client().await?;
    let token = auth_token_read().await?;
    client
        .user_logout(sonar_grpc::UserLogoutRequest {
            token: token.as_str().to_string(),
        })
        .await?;
    auth_token_delete().await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistArgs {
    #[clap(subcommand)]
    command: ArtistCommand,
}

#[derive(Debug, Parser)]
enum ArtistCommand {
    List(ArtistListArgs),
    Create(ArtistCreateArgs),
    Update(ArtistUpdateArgs),
    Delete(ArtistDeleteArgs),
}

#[derive(Debug, Parser)]
struct ArtistListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_artist_list(args: ArtistListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .artist_list(sonar_grpc::ArtistListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    let artists = response
        .into_inner()
        .artists
        .into_iter()
        .map(Artist::from)
        .collect::<Vec<_>>();
    stdout_values(&artists)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistCreateArgs {
    name: String,

    #[clap(long)]
    cover_art: Option<PathBuf>,

    #[clap(long, default_value = "")]
    genres: sonar::Genres,
}

async fn cmd_artist_create(args: ArtistCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => {
            tracing::info!("uploading cover art from {}", path.display());
            let content = tokio::fs::read(path).await.context("reading cover art")?;
            let response = client
                .image_create(sonar_grpc::ImageCreateRequest { content })
                .await
                .context("uploading image")?;
            Some(response.into_inner().image_id)
        }
        None => None,
    };

    let response = client
        .artist_create(sonar_grpc::ArtistCreateRequest {
            name: args.name,
            coverart_id: image_id,
            ..Default::default()
        })
        .await?;
    let artist = Artist::from(response.into_inner());
    stdout_value(&artist)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistUpdateArgs {}

async fn cmd_artist_update(_args: ArtistUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct ArtistDeleteArgs {}

async fn cmd_artist_delete(_args: ArtistDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AlbumArgs {
    #[clap(subcommand)]
    command: AlbumCommand,
}

#[derive(Debug, Parser)]
enum AlbumCommand {
    List(AlbumListArgs),
    Create(AlbumCreateArgs),
    Update(AlbumUpdateArgs),
    Delete(AlbumDeleteArgs),
}

#[derive(Debug, Parser)]
struct AlbumListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_album_list(args: AlbumListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .album_list(sonar_grpc::AlbumListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    let albums = response
        .into_inner()
        .albums
        .into_iter()
        .map(Album::from)
        .collect::<Vec<_>>();
    stdout_values(&albums)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumCreateArgs {
    artist_id: sonar::ArtistId,

    name: String,

    #[clap(long)]
    cover_art: Option<PathBuf>,

    #[clap(long, default_value = "")]
    genres: sonar::Genres,
}

async fn cmd_album_create(_args: AlbumCreateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AlbumUpdateArgs {}

async fn cmd_album_update(_args: AlbumUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AlbumDeleteArgs {}

async fn cmd_album_delete(_args: AlbumDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct TrackArgs {
    #[clap(subcommand)]
    command: TrackCommand,
}

#[derive(Debug, Parser)]
enum TrackCommand {
    List(TrackListArgs),
    Create(TrackCreateArgs),
    Update(TrackUpdateArgs),
    Delete(TrackDeleteArgs),
    Download(TrackDownloadArgs),
}

#[derive(Debug, Parser)]
struct TrackListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_track_list(args: TrackListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .track_list(sonar_grpc::TrackListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    let tracks = response
        .into_inner()
        .tracks
        .into_iter()
        .map(Track::from)
        .collect::<Vec<_>>();
    stdout_values(&tracks)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackCreateArgs {
    album_id: sonar::AlbumId,

    name: String,

    #[clap(long)]
    cover_art: Option<PathBuf>,

    #[clap(long, default_value = "")]
    genres: sonar::Genres,
}

async fn cmd_track_create(_args: TrackCreateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct TrackUpdateArgs {}

async fn cmd_track_update(_args: TrackUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct TrackDeleteArgs {}

async fn cmd_track_delete(_args: TrackDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct TrackDownloadArgs {
    id: sonar::TrackId,

    #[clap(long)]
    output: Option<PathBuf>,
}

async fn cmd_track_download(args: TrackDownloadArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct PlaylistArgs {
    #[clap(subcommand)]
    command: PlaylistCommand,
}

#[derive(Debug, Parser)]
enum PlaylistCommand {
    List(PlaylistListArgs),
    Create(PlaylistCreateArgs),
    Delete(PlaylistDeleteArgs),
    Add(PlaylistAddArgs),
    Remove(PlaylistRemoveArgs),
}

#[derive(Debug, Parser)]
struct PlaylistListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_playlist_list(args: PlaylistListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .playlist_list(sonar_grpc::PlaylistListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    let playlists = response
        .into_inner()
        .playlists
        .into_iter()
        .map(Playlist::from)
        .collect::<Vec<_>>();
    stdout_values(&playlists)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistCreateArgs {
    name: String,
}

async fn cmd_playlist_create(args: PlaylistCreateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct PlaylistDeleteArgs {
    id: sonar::PlaylistId,
}

async fn cmd_playlist_delete(args: PlaylistDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct PlaylistAddArgs {
    playlist_id: sonar::PlaylistId,

    track_ids: Vec<sonar::TrackId>,
}

async fn cmd_playlist_add(args: PlaylistAddArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct PlaylistRemoveArgs {
    playlist_id: sonar::PlaylistId,

    track_ids: Vec<sonar::TrackId>,
}

async fn cmd_playlist_remove(args: PlaylistRemoveArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct SyncArgs {}

async fn cmd_sync(_args: SyncArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AdminArgs {
    #[clap(subcommand)]
    command: AdminCommand,
}

#[derive(Debug, Parser)]
enum AdminCommand {
    User(AdminUserArgs),
    Playlist(AdminPlaylistArgs),
}

#[derive(Debug, Parser)]
struct AdminUserArgs {
    #[clap(subcommand)]
    command: AdminUserCommand,
}

#[derive(Debug, Parser)]
enum AdminUserCommand {
    List(AdminUserListArgs),
    Create(AdminUserCreateArgs),
    Update(AdminUserUpdateArgs),
    Delete(AdminUserDeleteArgs),
}

#[derive(Debug, Parser)]
struct AdminUserListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_admin_user_list(args: AdminUserListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .user_list(sonar_grpc::UserListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    for user in response.into_inner().users {
        println!("{:?}", user);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminUserCreateArgs {
    username: String,

    password: String,

    #[clap(long)]
    avatar: Option<PathBuf>,
}

async fn cmd_admin_user_create(args: AdminUserCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.avatar {
        Some(path) => {
            tracing::info!("uploading avatar from {}", path.display());
            let content = tokio::fs::read(path).await.context("reading avatar")?;
            let response = client
                .image_create(sonar_grpc::ImageCreateRequest { content })
                .await
                .context("uploading image")?;
            Some(response.into_inner().image_id)
        }
        None => None,
    };

    let response = client
        .user_create(sonar_grpc::UserCreateRequest {
            username: args.username,
            password: args.password,
            avatar_id: image_id,
        })
        .await?;
    println!("{:?}", response.into_inner());

    Ok(())
}

#[derive(Debug, Parser)]
struct AdminUserUpdateArgs {}

async fn cmd_admin_user_update(_args: AdminUserUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AdminUserDeleteArgs {
    id: sonar::UserId,
}

async fn cmd_admin_user_delete(args: AdminUserDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .user_delete(sonar_grpc::UserDeleteRequest {
            user_id: args.id.to_string(),
        })
        .await?;
    println!("{:?}", response.into_inner());
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminPlaylistArgs {
    #[clap(subcommand)]
    command: AdminPlaylistCommand,
}

#[derive(Debug, Parser)]
enum AdminPlaylistCommand {
    List(AdminPlaylistListArgs),
    Create(AdminPlaylistCreateArgs),
    Update(AdminPlaylistUpdateArgs),
    Delete(AdminPlaylistDeleteArgs),
}

#[derive(Debug, Parser)]
struct AdminPlaylistListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_admin_playlist_list(args: AdminPlaylistListArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AdminPlaylistCreateArgs {}

async fn cmd_admin_playlist_create(_args: AdminPlaylistCreateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AdminPlaylistUpdateArgs {}

async fn cmd_admin_playlist_update(_args: AdminPlaylistUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AdminPlaylistDeleteArgs {}

async fn cmd_admin_playlist_delete(_args: AdminPlaylistDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct ServerArgs {
    #[clap(long, default_value = "0.0.0.0:3000", env = "SONAR_ADDRESS")]
    address: SocketAddr,

    #[clap(
        long,
        default_value = "0.0.0.0:3001",
        env = "SONAR_OPENSUBSONIC_ADDRESS"
    )]
    address_opensubsonic: SocketAddr,

    #[clap(long, default_value = ".", env = "SONAR_DATA_DIR")]
    data_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct ImportArgs {
    paths: Vec<PathBuf>,
}

async fn cmd_import(args: ImportArgs) -> Result<()> {
    let mut client = create_client().await?;

    tracing::info!("scanning for files");
    let files = list_files_in_directories(args.paths)
        .await?
        .into_iter()
        .filter(|path| {
            IMPORT_FILETYPES
                .iter()
                .any(|filetype| path.extension().map(|x| x == *filetype).unwrap_or(false))
        });

    for filepath in files {
        tracing::info!("importing {}", filepath.display());
        let file = tokio::fs::File::open(&filepath).await?;
        let reader = tokio::io::BufReader::new(file);
        let stream =
            tokio_util::io::ReaderStream::new(reader).map(move |x| sonar_grpc::ImportRequest {
                chunk: x.unwrap().to_vec(),
                filepath: Some(filepath.display().to_string()),
                artist_id: None,
                album_id: None,
            });
        let response = client.import(stream).await?;
        let track = response.into_inner();
        println!("{:?}", track);
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataArgs {
    #[clap(long)]
    view: bool,

    #[clap(subcommand)]
    command: MetadataCommand,
}

#[derive(Debug, Parser)]
enum MetadataCommand {
    Album(MetadataAlbumArgs),
    AlbumTracks(MetadataAlbumTracksArgs),
}

#[derive(Debug, Parser)]
struct MetadataAlbumArgs {
    album_id: sonar::AlbumId,
}

async fn cmd_metadata_album(args: MetadataAlbumArgs, _view: bool) -> Result<()> {
    let mut client = create_client().await?;
    let _response = client
        .metadata_fetch(sonar_grpc::MetadataFetchRequest {
            kind: sonar_grpc::MetadataFetchKind::Album as i32,
            item_id: From::from(args.album_id),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataAlbumTracksArgs {
    album_id: sonar::AlbumId,
}

async fn cmd_metadata_album_tracks(args: MetadataAlbumTracksArgs, view: bool) -> Result<()> {
    let mut client = create_client().await?;
    if view {
        let response = client
            .metadata_album_tracks(sonar_grpc::MetadataAlbumTracksRequest {
                album_id: From::from(args.album_id),
            })
            .await?;
        for track in response.into_inner().tracks {
            let track_id = sonar::TrackId::try_from(track.0)?;
            let metadata = track.1;
            println!("Track: {}", track_id);
            println!("\tName: {:?}", metadata.name);
            for property in metadata.properties {
                println!("\t{}: {}", property.key, property.value);
            }
            if let Some(cover) = metadata.cover {
                println!("\tCover: {} bytes", cover.len());
            }
        }
    } else {
        let _response = client
            .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                kind: sonar_grpc::MetadataFetchKind::Albumtracks as i32,
                item_id: From::from(args.album_id),
            })
            .await?;
    }
    Ok(())
}

async fn cmd_server(args: ServerArgs) -> Result<()> {
    let data_dir = args
        .data_dir
        .canonicalize()
        .context("canonicalizing data dir")?;
    let storage_dir = data_dir.join("storage");
    let database_url = format!("sqlite://{}?mode=rwc", data_dir.join("sonar.db").display());

    tracing::info!("starting sonar server");
    tracing::info!("\taddress: {}", args.address);
    tracing::info!("\tdatabase: {}", database_url);
    tracing::info!("\tstorage: {}", storage_dir.display());

    let storage_backend = sonar::StorageBackend::Filesystem {
        path: data_dir.join("storage"),
    };
    let mut config = sonar::Config::new(database_url, storage_backend);
    config
        .register_extractor("lofty", sonar_extractor_lofty::LoftyExtractor)
        .context("registering lofty extractor")?;
    config
        .register_provider("beets", sonar_beets::BeetsMetadataImporter)
        .context("registering beets metadata importer")?;
    let context = sonar::new(config).await.context("creating sonar context")?;

    let grpc_context = context.clone();
    let f0 = tokio::spawn(async move {
        sonar_grpc::start_server(grpc_context, args.address)
            .await
            .context("starting grpc server")?;
        Ok::<(), eyre::Report>(())
    });

    let opensubsonic_context = context.clone();
    let f1 = tokio::spawn(async move {
        sonar_opensubsonic::start_server(args.address_opensubsonic, opensubsonic_context)
            .await
            .context("starting opensubsonic server")?;
        Ok::<(), eyre::Report>(())
    });

    let (r0, r1) = tokio::try_join!(f0, f1)?;
    r0?;
    r1?;

    Ok(())
}

#[derive(Debug, Parser)]
struct ExtractorArgs {
    files: Vec<PathBuf>,
}

async fn cmd_extractor(args: ExtractorArgs) -> Result<()> {
    use sonar::extractor::Extractor;

    let extractors = [("lofty", Box::new(sonar_extractor_lofty::LoftyExtractor))];

    for file in args.files {
        for extractor in extractors.iter() {
            tracing::info!(
                "extracting metadata from {} using {}",
                file.display(),
                extractor.0
            );
            let extracted = match extractor.1.extract(&file) {
                Ok(extracted) => extracted,
                Err(err) => {
                    tracing::warn!(
                        "failed to extract metadata from {}: {}",
                        file.display(),
                        err
                    );
                    continue;
                }
            };

            tracing::info!("extracted metadata:\n{:#?}", extracted);
        }
    }

    Ok(())
}

async fn list_files_in_directories(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut queue = paths;
    let mut files = vec![];
    let mut inserted = HashSet::<PathBuf>::new();
    while let Some(p) = queue.pop() {
        let metadata = tokio::fs::metadata(&p).await?;
        if metadata.is_dir() {
            let mut entries = tokio::fs::read_dir(&p).await?;
            while let Some(entry) = entries.next_entry().await? {
                queue.push(entry.path());
            }
        } else if metadata.is_file() {
            match p.canonicalize() {
                Ok(p) => {
                    if inserted.contains(&p) {
                        continue;
                    }
                    inserted.insert(p.clone());
                    files.push(p)
                }
                Err(err) => {
                    tracing::warn!("failed to canonicalize {}: {}", p.display(), err);
                }
            }
        }
    }
    Ok(files)
}

fn auth_token_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .expect("failed to get home dir")
        .join(".config")
        .join("sonar")
        .join("auth_token")
}

async fn auth_token_read() -> Result<String> {
    let path = auth_token_path();
    let token = tokio::fs::read_to_string(path)
        .await
        .context("reading auth token")?;
    Ok(token)
}

async fn auth_token_write(token: &str) -> Result<()> {
    let path = auth_token_path();
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .context("creating auth token dir")?;
    tokio::fs::write(path, token)
        .await
        .context("writing auth token")?;
    Ok(())
}

async fn auth_token_delete() -> Result<()> {
    let path = auth_token_path();
    tokio::fs::remove_file(path)
        .await
        .context("removing auth token")?;
    Ok(())
}

fn properties_from_pb(properties: Vec<sonar_grpc::Property>) -> sonar::Properties {
    let mut props = sonar::Properties::new();
    for property in properties {
        let key = property.key.parse::<sonar::PropertyKey>().unwrap();
        let value = property.value.parse::<sonar::PropertyValue>().unwrap();
        props.insert(key, value);
    }
    props
}

fn stdout_value<T: std::fmt::Display + Serialize>(value: T) -> Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    if *OUTPUT_JSON.get().unwrap() {
        serde_json::to_writer_pretty(&mut stdout, &value)?;
        writeln!(stdout)?;
    } else {
        writeln!(stdout, "{}", value)?;
    }
    Ok(())
}

fn stdout_values<T: std::fmt::Display + Serialize>(values: &[T]) -> Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    if *OUTPUT_JSON.get().unwrap() {
        serde_json::to_writer_pretty(&mut stdout, &values)?;
        writeln!(stdout)?;
    } else {
        for value in values {
            writeln!(stdout, "{}", value)?;
        }
    }
    Ok(())
}
