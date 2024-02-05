use std::{
    collections::HashSet,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::OnceLock,
};

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
    Scrobble(ScrobbleArgs),
    Sync(SyncArgs),
    Pin(PinArgs),
    Admin(AdminArgs),
    Import(ImportArgs),
    Server(ServerArgs),
    Extractor(ExtractorArgs),
    Spotify(SpotifyArgs),
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

#[derive(Debug, Serialize)]
struct Scrobble {
    id: String,
    track: String,
    user: String,
    listen_at: u64,
    listen_duration: u32,
    listen_device: String,
    properties: Properties,
}

impl std::fmt::Display for Scrobble {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t{}", self.id, self.user, self.track)
    }
}

impl From<sonar_grpc::Scrobble> for Scrobble {
    fn from(value: sonar_grpc::Scrobble) -> Self {
        Self {
            id: value.id,
            track: value.track_id,
            user: value.user_id,
            listen_at: value.listen_at.unwrap().seconds as u64,
            listen_duration: value.listen_duration.unwrap().seconds as u32,
            listen_device: value.listen_device,
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
            PlaylistCommand::Update(cargs) => cmd_playlist_update(cargs).await?,
            PlaylistCommand::Delete(cargs) => cmd_playlist_delete(cargs).await?,
            PlaylistCommand::Add(cargs) => cmd_playlist_add(cargs).await?,
            PlaylistCommand::Remove(cargs) => cmd_playlist_remove(cargs).await?,
        },
        Command::Scrobble(cargs) => match cargs.command {
            ScrobbleCommand::List(cargs) => cmd_scrobble_list(cargs).await?,
            ScrobbleCommand::Create(cargs) => cmd_scrobble_create(cargs).await?,
            ScrobbleCommand::Delete(cargs) => cmd_scrobble_delete(cargs).await?,
        },
        Command::Sync(cargs) => cmd_sync(cargs).await?,
        Command::Pin(cargs) => match cargs.command {
            PinCommand::List(cargs) => cmd_pin_list(cargs).await?,
            PinCommand::Set(cargs) => cmd_pin_set(cargs).await?,
            PinCommand::Unset(cargs) => cmd_pin_unset(cargs).await?,
        },
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
            AdminCommand::MetadataFetch(cargs) => cmd_admin_metadata_fetch(cargs).await?,
        },
        Command::Import(cargs) => cmd_import(cargs).await?,
        Command::Server(cargs) => cmd_server(cargs).await?,
        Command::Extractor(cargs) => cmd_extractor(cargs).await?,
        Command::Spotify(cargs) => match cargs.command {
            SpotifyCommand::List(cargs) => cmd_spotify_list(cargs).await?,
            SpotifyCommand::Add(cargs) => cmd_spotify_add(cargs).await?,
            SpotifyCommand::Remove(cargs) => cmd_spotify_remove(cargs).await?,
        },
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
    let response = response.into_inner();
    let token = response.token;
    let user_id = response.user_id;
    auth_write(&user_id, &token).await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct LogoutArgs {}

async fn cmd_logout(_args: LogoutArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (_, token) = auth_read().await?;
    client
        .user_logout(sonar_grpc::UserLogoutRequest {
            token: token.as_str().to_string(),
        })
        .await?;
    auth_delete().await?;
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
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };

    let response = client
        .artist_create(sonar_grpc::ArtistCreateRequest {
            name: args.name,
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    let artist = Artist::from(response.into_inner());
    stdout_value(&artist)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistUpdateArgs {
    id: sonar::ArtistId,

    #[clap(long)]
    name: Option<String>,

    #[clap(long)]
    cover_art: Option<PathBuf>,
}

async fn cmd_artist_update(args: ArtistUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };
    client
        .artist_update(sonar_grpc::ArtistUpdateRequest {
            artist_id: args.id.to_string(),
            name: args.name,
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistDeleteArgs {
    id: sonar::ArtistId,
}

async fn cmd_artist_delete(args: ArtistDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .artist_delete(sonar_grpc::ArtistDeleteRequest {
            artist_id: args.id.to_string(),
        })
        .await?;
    Ok(())
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
}

async fn cmd_album_create(args: AlbumCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };

    let response = client
        .album_create(sonar_grpc::AlbumCreateRequest {
            name: args.name,
            artist_id: args.artist_id.to_string(),
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    let album = Album::from(response.into_inner());
    stdout_value(&album)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumUpdateArgs {
    id: sonar::AlbumId,

    #[clap(long)]
    name: Option<String>,

    #[clap(long)]
    cover_art: Option<PathBuf>,
}

async fn cmd_album_update(args: AlbumUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };
    client
        .album_update(sonar_grpc::AlbumUpdateRequest {
            album_id: args.id.to_string(),
            name: args.name,
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumDeleteArgs {
    id: sonar::AlbumId,
}

async fn cmd_album_delete(args: AlbumDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .album_delete(sonar_grpc::AlbumDeleteRequest {
            album_id: args.id.to_string(),
        })
        .await?;
    Ok(())
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

async fn cmd_track_create(args: TrackCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };

    let response = client
        .track_create(sonar_grpc::TrackCreateRequest {
            name: args.name,
            album_id: args.album_id.to_string(),
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    let track = Track::from(response.into_inner());
    stdout_value(&track)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackUpdateArgs {
    id: sonar::TrackId,

    #[clap(long)]
    name: Option<String>,

    #[clap(long)]
    album_id: Option<sonar::AlbumId>,

    #[clap(long)]
    cover_art: Option<PathBuf>,
}

async fn cmd_track_update(args: TrackUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };
    client
        .track_update(sonar_grpc::TrackUpdateRequest {
            track_id: args.id.to_string(),
            name: args.name,
            album_id: args.album_id.map(|x| x.to_string()),
            coverart_id: image_id.map(|x| x.to_string()),
            ..Default::default()
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackDeleteArgs {
    id: sonar::TrackId,
}

async fn cmd_track_delete(args: TrackDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .track_delete(sonar_grpc::TrackDeleteRequest {
            track_id: args.id.to_string(),
        })
        .await?;
    Ok(())
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
    Update(PlaylistUpdateArgs),
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
    let (user_id, _) = auth_read().await?;
    let response = client
        .playlist_list(sonar_grpc::PlaylistListRequest {
            offset: args.params.offset,
            count: args.params.limit,
            user_id: Some(user_id.to_string()),
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
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .playlist_create(sonar_grpc::PlaylistCreateRequest {
            name: args.name,
            owner_id: user_id.to_string(),
            ..Default::default()
        })
        .await?;
    let playlist = Playlist::from(response.into_inner());
    stdout_value(&playlist)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistUpdateArgs {
    id: sonar::PlaylistId,

    #[clap(long)]
    name: Option<String>,
}

async fn cmd_playlist_update(args: PlaylistUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .playlist_update(sonar_grpc::PlaylistUpdateRequest {
            playlist_id: args.id.to_string(),
            name: args.name,
            ..Default::default()
        })
        .await?;
    println!("{:?}", response.into_inner());
    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistDeleteArgs {
    #[clap(long)]
    force: bool,

    id: sonar::PlaylistId,
}

async fn cmd_playlist_delete(args: PlaylistDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .playlist_track_list(sonar_grpc::PlaylistTrackListRequest {
            playlist_id: args.id.to_string(),
        })
        .await?;

    if !response.into_inner().tracks.is_empty() && !args.force {
        eyre::bail!("playlist is not empty, use --force to delete");
    }

    client
        .playlist_delete(sonar_grpc::PlaylistDeleteRequest {
            playlist_id: args.id.to_string(),
        })
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistAddArgs {
    playlist_id: sonar::PlaylistId,

    track_ids: Vec<sonar::TrackId>,
}

async fn cmd_playlist_add(args: PlaylistAddArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .playlist_track_insert(sonar_grpc::PlaylistTrackInsertRequest {
            playlist_id: args.playlist_id.to_string(),
            track_ids: args.track_ids.iter().map(|x| x.to_string()).collect(),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistRemoveArgs {
    playlist_id: sonar::PlaylistId,

    track_ids: Vec<sonar::TrackId>,
}

async fn cmd_playlist_remove(args: PlaylistRemoveArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .playlist_track_remove(sonar_grpc::PlaylistTrackRemoveRequest {
            playlist_id: args.playlist_id.to_string(),
            track_ids: args.track_ids.iter().map(|x| x.to_string()).collect(),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ScrobbleArgs {
    #[clap(subcommand)]
    command: ScrobbleCommand,
}

#[derive(Debug, Parser)]
enum ScrobbleCommand {
    List(ScrobbleListArgs),
    Create(ScrobbleCreateArgs),
    Delete(ScrobbleDeleteArgs),
}

#[derive(Debug, Parser)]
struct ScrobbleListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_scrobble_list(args: ScrobbleListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .scrobble_list(sonar_grpc::ScrobbleListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    let scrobbles = response
        .into_inner()
        .scrobbles
        .into_iter()
        .map(Scrobble::from)
        .collect::<Vec<_>>();
    stdout_values(&scrobbles)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ScrobbleCreateArgs {
    user_id: sonar::UserId,

    track_id: sonar::TrackId,

    listen_at: u64,

    listen_duration: u32,

    listen_device: String,
}

async fn cmd_scrobble_create(args: ScrobbleCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .scrobble_create(sonar_grpc::ScrobbleCreateRequest {
            user_id: args.user_id.to_string(),
            track_id: args.track_id.to_string(),
            listen_at: Some(prost_types::Timestamp {
                seconds: args.listen_at as i64,
                nanos: 0,
            }),
            listen_duration: Some(prost_types::Duration {
                seconds: args.listen_duration as i64,
                nanos: 0,
            }),
            listen_device: args.listen_device,
            ..Default::default()
        })
        .await?;
    let scrobble = Scrobble::from(response.into_inner());
    stdout_value(&scrobble)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ScrobbleDeleteArgs {
    id: sonar::ScrobbleId,
}

async fn cmd_scrobble_delete(args: ScrobbleDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .scrobble_delete(sonar_grpc::ScrobbleDeleteRequest {
            scrobble_id: args.id.to_string(),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct SyncArgs {}

async fn cmd_sync(_args: SyncArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct PinArgs {
    #[clap(subcommand)]
    command: PinCommand,
}

#[derive(Debug, Parser)]
enum PinCommand {
    List(PinListArgs),
    Set(PinSetArgs),
    Unset(PinUnsetArgs),
}

#[derive(Debug, Parser)]
struct PinListArgs {}

async fn cmd_pin_list(_args: PinListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .pin_list(sonar_grpc::PinListRequest { user_id })
        .await?;
    for pin in response.into_inner().sonar_ids {
        println!("{}", pin);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct PinSetArgs {
    id: String,
}

async fn cmd_pin_set(args: PinSetArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    client
        .pin_set(sonar_grpc::PinSetRequest {
            user_id,
            sonar_ids: vec![args.id],
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct PinUnsetArgs {
    id: String,
}

async fn cmd_pin_unset(args: PinUnsetArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    client
        .pin_unset(sonar_grpc::PinUnsetRequest {
            user_id,
            sonar_ids: vec![args.id],
        })
        .await?;
    Ok(())
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
    MetadataFetch(AdminMetadataFetchArgs),
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
struct AdminUserUpdateArgs {
    userid: sonar::UserId,

    #[clap(long)]
    password: Option<String>,

    #[clap(long)]
    avatar: Option<PathBuf>,
}

async fn cmd_admin_user_update(args: AdminUserUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let avatar_id = match args.avatar {
        Some(path) => Some(upload_image(&mut client, &path).await?),
        None => None,
    };
    client
        .user_update(sonar_grpc::UserUpdateRequest {
            user_id: args.userid.to_string(),
            password: args.password,
            avatar_id: avatar_id.map(|x| x.to_string()),
        })
        .await?;
    Ok(())
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
    let mut client = create_client().await?;
    let response = client
        .playlist_list(sonar_grpc::PlaylistListRequest {
            offset: args.params.offset,
            count: args.params.limit,
            ..Default::default()
        })
        .await?;
    for playlist in response.into_inner().playlists {
        println!("{:?}", playlist);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminPlaylistCreateArgs {
    user_id: sonar::UserId,

    name: String,
}

async fn cmd_admin_playlist_create(args: AdminPlaylistCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .playlist_create(sonar_grpc::PlaylistCreateRequest {
            name: args.name,
            owner_id: args.user_id.to_string(),
            ..Default::default()
        })
        .await?;
    stdout_value(&Playlist::from(response.into_inner()))?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminPlaylistUpdateArgs {
    id: sonar::PlaylistId,

    #[clap(long)]
    name: Option<String>,
}

async fn cmd_admin_playlist_update(args: AdminPlaylistUpdateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .playlist_update(sonar_grpc::PlaylistUpdateRequest {
            playlist_id: args.id.to_string(),
            name: args.name,
            ..Default::default()
        })
        .await?;
    stdout_value(&Playlist::from(response.into_inner()))?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminPlaylistDeleteArgs {
    id: sonar::PlaylistId,
}

async fn cmd_admin_playlist_delete(args: AdminPlaylistDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .playlist_delete(sonar_grpc::PlaylistDeleteRequest {
            playlist_id: args.id.to_string(),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AdminMetadataFetchArgs {
    id: sonar::SonarId,
}

async fn cmd_admin_metadata_fetch(args: AdminMetadataFetchArgs) -> Result<()> {
    let mut client = create_client().await?;
    match args.id {
        sonar::SonarId::Artist(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Artist as i32,
                    item_id: args.id.to_string(),
                })
                .await?;
        }
        sonar::SonarId::Album(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Albumtracks as i32,
                    item_id: args.id.to_string(),
                })
                .await?;

            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Album as i32,
                    item_id: args.id.to_string(),
                })
                .await?;
        }
        sonar::SonarId::Track(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Album as i32,
                    item_id: args.id.to_string(),
                })
                .await?;
        }
        _ => {
            eyre::bail!("unsupported id type");
        }
    };
    Ok(())
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

    #[clap(long, env = "SONAR_SPOTIFY_USERNAME")]
    spotify_username: Option<String>,

    #[clap(long, env = "SONAR_SPOTIFY_PASSWORD")]
    spotify_password: Option<String>,
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
    config
        .register_scrobbler_for_user(
            "listenbrainz/admin",
            "admin".parse()?,
            sonar_listenbrainz::ListenBrainzScrobbler::new(std::env!("LISTENBRAINZ_API_KEY")),
        )
        .context("registering listenbrainz scrobbler")?;
    let context = sonar::new(config).await.context("creating sonar context")?;

    let spotify =
        if let (Some(username), Some(password)) = (args.spotify_username, args.spotify_password) {
            let spotify = sonar_spotify::Context::new(
                context.clone(),
                sonar_spotify::LoginCredentials { username, password },
                data_dir.join("spotify"),
            )
            .await
            .context("creating spotify context")?;
            Some(spotify)
        } else {
            None
        };

    let grpc_context = context.clone();
    let f0 = tokio::spawn(async move {
        sonar_grpc::start_server(args.address, grpc_context, spotify)
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

#[derive(Debug, Parser)]
struct SpotifyArgs {
    #[clap(subcommand)]
    command: SpotifyCommand,
}

#[derive(Debug, Parser)]
enum SpotifyCommand {
    List(SpotifyListArgs),
    Add(SpotifyAddArgs),
    Remove(SpotifyRemoveArgs),
}

#[derive(Debug, Parser)]
struct SpotifyListArgs {}

async fn cmd_spotify_list(_args: SpotifyListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .spotify_list(sonar_grpc::SpotifyListRequest {})
        .await?;
    for id in response.into_inner().spotify_ids {
        println!("{}", id);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct SpotifyAddArgs {
    uri: String,
}

async fn cmd_spotify_add(args: SpotifyAddArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .spotify_add(sonar_grpc::SpotifyAddRequest {
            spotify_ids: vec![args.uri],
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct SpotifyRemoveArgs {
    uri: String,
}

async fn cmd_spotify_remove(args: SpotifyRemoveArgs) -> Result<()> {
    let mut client = create_client().await?;
    client
        .spotify_remove(sonar_grpc::SpotifyRemoveRequest {
            spotify_ids: vec![args.uri],
        })
        .await?;
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

fn auth_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .expect("failed to get home dir")
        .join(".config")
        .join("sonar")
        .join("auth")
}

async fn auth_read() -> Result<(String, String)> {
    let path = auth_path();
    let auth = tokio::fs::read_to_string(path)
        .await
        .context("reading auth token")?;
    let (user_id, token) = auth.split_once(' ').expect("invalid auth token");
    if user_id.is_empty() || token.is_empty() {
        eyre::bail!("invalid auth token");
    }
    Ok((user_id.to_string(), token.to_string()))
}

async fn auth_write(user_id: &str, token: &str) -> Result<()> {
    let path = auth_path();
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .context("creating auth token dir")?;
    let auth = format!("{} {}", user_id, token);
    tokio::fs::write(path, auth).await.context("writing auth")?;
    Ok(())
}

async fn auth_delete() -> Result<()> {
    let path = auth_path();
    tokio::fs::remove_file(path)
        .await
        .context("removing auth token")?;
    Ok(())
}

async fn upload_image(client: &mut sonar_grpc::Client, path: &Path) -> Result<sonar::ImageId> {
    tracing::info!("uploading image from {}", path.display());
    let content = tokio::fs::read(path).await.context("reading cover art")?;
    let response = client
        .image_create(sonar_grpc::ImageCreateRequest { content })
        .await
        .context("uploading image")?;
    let response = response.into_inner();
    tracing::info!("image uploaded with id {}", response.image_id);
    Ok(response.image_id.parse()?)
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
