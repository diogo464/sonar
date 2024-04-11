#![feature(let_chains)]
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, OnceLock},
};

use clap::Parser;
use eyre::{Context, Result};
use lofty::{Accessor, TaggedFileExt};
use serde::Serialize;
use sonar::{Genres, Properties};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

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
    Favorite(FavoriteArgs),
    Scrobble(ScrobbleArgs),
    Sync(SyncArgs),
    Pin(PinArgs),
    Search(SearchArgs),
    Subscription(SubscriptionArgs),
    Download(DownloadArgs),
    Metadata(MetadataArgs),
    Admin(AdminArgs),
    Import(ImportArgs),
    Server(ServerArgs),
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
    genres: Genres,
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
            genres: genres_from_pb(value.genres),
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
    genres: Genres,
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
            genres: genres_from_pb(value.genres),
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
struct Lyrics {
    synced: bool,
    lines: Vec<LyricsLine>,
}

#[derive(Debug, Serialize)]
struct LyricsLine {
    offset: u32,
    text: String,
}

impl std::fmt::Display for Lyrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}\t{}", line.offset, line.text)?;
        }
        Ok(())
    }
}

impl From<sonar_grpc::Lyrics> for Lyrics {
    fn from(value: sonar_grpc::Lyrics) -> Self {
        Self {
            synced: value.synced,
            lines: value
                .lines
                .into_iter()
                .map(|line| LyricsLine {
                    offset: line.offset,
                    text: line.text,
                })
                .collect(),
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

#[derive(Debug, Serialize)]
enum SearchResult {
    Artist(Artist),
    Album(Album),
    Track(Track),
    Playlist(Playlist),
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchResult::Artist(artist) => write!(f, "{}\t{}", artist.id, artist.name),
            SearchResult::Album(album) => write!(f, "{}\t{}", album.id, album.name),
            SearchResult::Track(track) => write!(f, "{}\t{}", track.id, track.name),
            SearchResult::Playlist(playlist) => write!(f, "{}\t{}", playlist.id, playlist.name),
        }
    }
}

#[derive(Debug, Serialize)]
struct SearchResults(Vec<SearchResult>);

impl std::fmt::Display for SearchResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for result in &self.0 {
            writeln!(f, "{}", result)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct Subscription {
    user: String,
    external_id: String,
}

impl From<sonar_grpc::Subscription> for Subscription {
    fn from(value: sonar_grpc::Subscription) -> Self {
        Self {
            user: value.user_id,
            external_id: value.external_id,
        }
    }
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.user, self.external_id)
    }
}

#[derive(Debug, Serialize)]
struct Download {
    user: String,
    external_id: String,
    description: String,
}

impl From<sonar_grpc::Download> for Download {
    fn from(value: sonar_grpc::Download) -> Self {
        Self {
            user: value.user_id,
            external_id: value.external_id,
            description: value.description,
        }
    }
}

impl std::fmt::Display for Download {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            self.user, self.external_id, self.description
        )
    }
}

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    if let Command::Server(ref args) = args.command
        && args.jaeger_exporter
    {
        let mut tracer = opentelemetry_jaeger::new_agent_pipeline();
        if let Some(ref endpoint) = args.jaeger_exporter_endpoint {
            tracer = tracer.with_endpoint(endpoint);
        }
        let tracer = tracer.with_service_name("sonar").install_simple()?;

        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        tracing_subscriber::registry()
            .with(opentelemetry)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
            )
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
            ArtistCommand::Search(cargs) => cmd_artist_search(cargs).await?,
            ArtistCommand::Albums(cargs) => cmd_artist_albums(cargs).await?,
        },
        Command::Album(cargs) => match cargs.command {
            AlbumCommand::List(cargs) => cmd_album_list(cargs).await?,
            AlbumCommand::Create(cargs) => cmd_album_create(cargs).await?,
            AlbumCommand::Update(cargs) => cmd_album_update(cargs).await?,
            AlbumCommand::Delete(cargs) => cmd_album_delete(cargs).await?,
            AlbumCommand::Search(cargs) => cmd_album_search(cargs).await?,
            AlbumCommand::Tracks(cargs) => cmd_album_tracks(cargs).await?,
        },
        Command::Track(cargs) => match cargs.command {
            TrackCommand::List(cargs) => cmd_track_list(cargs).await?,
            TrackCommand::Create(cargs) => cmd_track_create(cargs).await?,
            TrackCommand::Update(cargs) => cmd_track_update(cargs).await?,
            TrackCommand::Delete(cargs) => cmd_track_delete(cargs).await?,
            TrackCommand::Search(cargs) => cmd_track_search(cargs).await?,
            TrackCommand::Lyrics(cargs) => cmd_track_lyrics(cargs).await?,
            TrackCommand::Download(cargs) => cmd_track_download(cargs).await?,
        },
        Command::Favorite(cargs) => match cargs.command {
            FavoriteCommand::List(cargs) => cmd_favorite_list(cargs).await?,
            FavoriteCommand::Add(cargs) => cmd_favorite_add(cargs).await?,
            FavoriteCommand::Remove(cargs) => cmd_favorite_remove(cargs).await?,
        },
        Command::Playlist(cargs) => match cargs.command {
            PlaylistCommand::List(cargs) => cmd_playlist_list(cargs).await?,
            PlaylistCommand::Create(cargs) => cmd_playlist_create(cargs).await?,
            PlaylistCommand::Duplicate(cargs) => cmd_playlist_duplicate(cargs).await?,
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
        Command::Search(cargs) => cmd_search(cargs).await?,
        Command::Subscription(cargs) => match cargs.command {
            SubscriptionCommand::List(cargs) => cmd_subscription_list(cargs).await?,
            SubscriptionCommand::Create(cargs) => cmd_subscription_create(cargs).await?,
            SubscriptionCommand::Delete(cargs) => cmd_subscription_delete(cargs).await?,
        },
        Command::Download(cargs) => match cargs.command {
            DownloadCommand::List(cargs) => cmd_download_list(cargs).await?,
            DownloadCommand::Start(cargs) => cmd_download_start(cargs).await?,
            DownloadCommand::Stop(cargs) => cmd_download_stop(cargs).await?,
        },
        Command::Metadata(cargs) => match cargs.command {
            MetadataCommand::Providers => cmd_metadata_providers().await?,
            MetadataCommand::Fetch(cargs) => cmd_metadata_fetch(cargs).await?,
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
            AdminCommand::MetadataPreview(cargs) => cmd_admin_metadata_preview(cargs).await?,
        },
        Command::Import(cargs) => cmd_import(cargs).await?,
        Command::Server(cargs) => cmd_server(cargs).await?,
    }

    Ok(())
}

async fn create_client() -> Result<sonar_grpc::Client> {
    let endpoint = SERVER_ENDPOINT.get().unwrap();
    let (_, token) = auth_read().await.unwrap_or_default();
    sonar_grpc::client_with_token(endpoint, token.as_str())
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
    Search(ArtistSearchArgs),
    Albums(ArtistAlbumsArgs),
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
    stdout_value(artist)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistUpdateArgs {
    id: String,

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
    id: String,
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
struct ArtistSearchArgs {
    query: String,
}

async fn cmd_artist_search(args: ArtistSearchArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .search(sonar_grpc::SearchRequest {
            user_id,
            query: args.query,
            ..Default::default()
        })
        .await?;
    let mut artists = Vec::new();
    response
        .into_inner()
        .results
        .into_iter()
        .for_each(|result| {
            if let Some(sonar_grpc::search_result::Result::Artist(artist)) = result.result {
                artists.push(Artist::from(artist));
            }
        });
    stdout_values(&artists)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistAlbumsArgs {
    artist_id: String,
}

async fn cmd_artist_albums(args: ArtistAlbumsArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .album_list_by_artist(sonar_grpc::AlbumListByArtistRequest {
            artist_id: args.artist_id.to_string(),
            ..Default::default()
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
    Search(AlbumSearchArgs),
    Tracks(AlbumTracksArgs),
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
    artist_id: String,

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
    stdout_value(album)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumUpdateArgs {
    id: String,

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
    id: String,
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
struct AlbumSearchArgs {
    query: String,
}

async fn cmd_album_search(args: AlbumSearchArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .search(sonar_grpc::SearchRequest {
            user_id,
            query: args.query,
            ..Default::default()
        })
        .await?;
    let mut albums = Vec::new();
    response
        .into_inner()
        .results
        .into_iter()
        .for_each(|result| {
            if let Some(sonar_grpc::search_result::Result::Album(album)) = result.result {
                albums.push(Album::from(album));
            }
        });
    stdout_values(&albums)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumTracksArgs {
    album_id: String,
}

async fn cmd_album_tracks(args: AlbumTracksArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .track_list_by_album(sonar_grpc::TrackListByAlbumRequest {
            album_id: args.album_id.to_string(),
            ..Default::default()
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
    Search(TrackSearchArgs),
    Lyrics(TrackLyricsArgs),
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
    album_id: String,

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
    stdout_value(track)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackUpdateArgs {
    id: String,

    #[clap(long)]
    name: Option<String>,

    #[clap(long)]
    album_id: Option<String>,

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
struct TrackSearchArgs {
    query: String,
}

async fn cmd_track_search(args: TrackSearchArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .search(sonar_grpc::SearchRequest {
            user_id,
            query: args.query,
            ..Default::default()
        })
        .await?;
    let mut tracks = Vec::new();
    response
        .into_inner()
        .results
        .into_iter()
        .for_each(|result| {
            if let Some(sonar_grpc::search_result::Result::Track(track)) = result.result {
                tracks.push(Track::from(track));
            }
        });
    stdout_values(&tracks)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackLyricsArgs {
    id: String,
}

async fn cmd_track_lyrics(args: TrackLyricsArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .track_lyrics(sonar_grpc::TrackLyricsRequest { track_id: args.id })
        .await?;
    let lyrics = Lyrics::from(response.into_inner().lyrics.unwrap());
    stdout_value(lyrics)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct TrackDownloadArgs {
    id: sonar::TrackId,

    #[clap(long)]
    output: Option<PathBuf>,
}

async fn cmd_track_download(_args: TrackDownloadArgs) -> Result<()> {
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
    Duplicate(PlaylistDuplicateArgs),
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
    stdout_value(playlist)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct PlaylistDuplicateArgs {
    id: sonar::PlaylistId,
    new_name: String,
}

async fn cmd_playlist_duplicate(args: PlaylistDuplicateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .playlist_duplicate(sonar_grpc::PlaylistDuplicateRequest {
            user_id: user_id.to_string(),
            playlist_id: args.id.to_string(),
            new_playlist_name: args.new_name,
        })
        .await?;
    let playlist = Playlist::from(response.into_inner());
    stdout_value(playlist)?;
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
struct FavoriteArgs {
    #[clap(subcommand)]
    command: FavoriteCommand,
}

#[derive(Debug, Parser)]
enum FavoriteCommand {
    List(FavoriteListArgs),
    Add(FavoriteAddArgs),
    Remove(FavoriteRemoveArgs),
}

#[derive(Debug, Parser)]
struct FavoriteListArgs {}

async fn cmd_favorite_list(_args: FavoriteListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .favorite_list(sonar_grpc::FavoriteListRequest { user_id })
        .await?;
    let favorites = response.into_inner().favorites;

    for favorite in favorites {
        println!("{}", favorite.item_id);
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct FavoriteAddArgs {
    items_ids: Vec<sonar::SonarId>,
}

async fn cmd_favorite_add(args: FavoriteAddArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    for item_id in args.items_ids {
        client
            .favorite_add(sonar_grpc::FavoriteAddRequest {
                user_id: user_id.clone(),
                item_id: item_id.to_string(),
            })
            .await?;
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct FavoriteRemoveArgs {
    items_ids: Vec<sonar::SonarId>,
}

async fn cmd_favorite_remove(args: FavoriteRemoveArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    for item_id in args.items_ids {
        client
            .favorite_remove(sonar_grpc::FavoriteRemoveRequest {
                user_id: user_id.clone(),
                item_id: item_id.to_string(),
            })
            .await?;
    }
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
    stdout_value(scrobble)?;
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
struct SyncArgs {
    /// output directory for downloaded files
    #[clap(long, default_value = ".")]
    output: PathBuf,

    /// list of sonar ids to sync
    ids: Vec<sonar::SonarId>,
}

async fn cmd_sync(args: SyncArgs) -> Result<()> {
    let client = create_client().await?;
    let (track_tx, mut track_rx) = tokio::sync::mpsc::channel::<sonar_grpc::Track>(100);
    let (album_tx, mut album_rx) = tokio::sync::mpsc::channel::<sonar_grpc::Album>(100);
    let (artist_tx, mut artist_rx) = tokio::sync::mpsc::channel::<sonar_grpc::Artist>(100);

    tracing::info!("fetching track information...");
    for id in args.ids {
        let mut client = client.clone();
        let track_tx = track_tx.clone();
        tokio::spawn(async move {
            match id {
                sonar::SonarId::Artist(artist) => {
                    let response = client
                        .album_list_by_artist(sonar_grpc::AlbumListByArtistRequest {
                            artist_id: artist.to_string(),
                            ..Default::default()
                        })
                        .await
                        .unwrap();
                    for album in response.into_inner().albums {
                        let response = client
                            .track_list_by_album(sonar_grpc::TrackListByAlbumRequest {
                                album_id: album.id,
                                ..Default::default()
                            })
                            .await
                            .unwrap();
                        for track in response.into_inner().tracks {
                            let _ = track_tx.send(track).await;
                        }
                    }
                }
                sonar::SonarId::Album(album) => {
                    let response = client
                        .track_list_by_album(sonar_grpc::TrackListByAlbumRequest {
                            album_id: album.to_string(),
                            ..Default::default()
                        })
                        .await
                        .unwrap();
                    for track in response.into_inner().tracks {
                        let _ = track_tx.send(track).await;
                    }
                }
                sonar::SonarId::Track(track) => {
                    let response = client
                        .track_get(sonar_grpc::TrackGetRequest {
                            track: track.to_string(),
                        })
                        .await
                        .unwrap();
                    let _ = track_tx.send(response.into_inner()).await;
                }
                sonar::SonarId::Playlist(playlist) => {
                    let response = client
                        .playlist_track_list(sonar_grpc::PlaylistTrackListRequest {
                            playlist_id: playlist.to_string(),
                        })
                        .await
                        .unwrap();
                    for track in response.into_inner().tracks {
                        let _ = track_tx.send(track).await;
                    }
                }
                _ => tracing::warn!("unsupported sonar id: {}", id),
            }
        });
    }
    drop(track_tx);

    let mut tracks: Vec<sonar_grpc::Track> = Default::default();
    let mut albums: HashMap<String, sonar_grpc::Album> = Default::default();
    let mut artists: HashMap<String, sonar_grpc::Artist> = Default::default();

    let mut pending_ids: HashSet<String> = Default::default();
    while let Some(track) = track_rx.recv().await {
        // fetch track artist
        if pending_ids.insert(track.artist_id.clone()) {
            let mut client = client.clone();
            let artist_id = track.artist_id.clone();
            let artist_tx = artist_tx.clone();
            tokio::spawn(async move {
                let response = client
                    .artist_get(sonar_grpc::ArtistGetRequest { artist: artist_id })
                    .await
                    .unwrap();
                let _ = artist_tx.send(response.into_inner()).await;
            });
        }

        // fetch track album
        if pending_ids.insert(track.album_id.clone()) {
            let mut client = client.clone();
            let album_id = track.album_id.clone();
            let album_tx = album_tx.clone();
            tokio::spawn(async move {
                let response = client
                    .album_get(sonar_grpc::AlbumGetRequest { album: album_id })
                    .await
                    .unwrap();
                let _ = album_tx.send(response.into_inner()).await;
            });
        }

        tracks.push(track);
    }
    drop(album_tx);
    drop(artist_tx);

    // collect artists and albums
    while let Some(album) = album_rx.recv().await {
        albums.insert(album.id.clone(), album);
    }
    while let Some(artist) = artist_rx.recv().await {
        artists.insert(artist.id.clone(), artist);
    }

    tracing::info!(
        "fetched {} artists, {} albums, and {} tracks",
        artists.len(),
        albums.len(),
        tracks.len()
    );
    tracing::info!("downloading tracks...");

    // create directories and start track downloads
    let mut handles = Vec::new();
    for track in tracks {
        let artist = &artists[&track.artist_id];
        let album = &albums[&track.album_id];

        let disc_number = track
            .properties
            .iter()
            .find(|p| p.key == sonar::prop::DISC_NUMBER.as_str())
            .map(|p| p.value.parse::<u32>().ok())
            .flatten()
            .unwrap_or_else(|| 0);
        let track_number = track
            .properties
            .iter()
            .find(|p| p.key == sonar::prop::TRACK_NUMBER.as_str())
            .map(|p| p.value.parse::<u32>().ok())
            .flatten()
            .unwrap_or_else(|| 0);

        let parent_dir = args.output.join(&artist.name).join(&album.name);
        let track_path = parent_dir
            .join(format!("{:02} - {}", track_number, track.name))
            .with_extension("mp3");
        let cover_path_png = parent_dir.join("cover.png");
        let cover_path_jpg = parent_dir.join("cover.jpg");

        tokio::fs::create_dir_all(&parent_dir)
            .await
            .with_context(|| format!("creating directory {}", parent_dir.display()))?;

        if !track_path.exists() {
            tracing::info!("downloading track: {}", track_path.display());
            let mut client = client.clone();
            let artist = artist.clone();
            let album = album.clone();
            let track = track.clone();
            let track_id = track.id.clone();
            let handle = tokio::spawn(async move {
                let download_path = track_path.with_extension("part");
                let response = client
                    .track_download(sonar_grpc::TrackDownloadRequest {
                        track_id: track_id.clone(),
                    })
                    .await
                    .with_context(|| format!("downloading track {}", track_id))?;

                {
                    let mut stream = response.into_inner();
                    let file = tokio::fs::File::create(&download_path)
                        .await
                        .with_context(|| format!("creating file {}", download_path.display()))?;
                    let mut writer = tokio::io::BufWriter::new(file);
                    while let Some(part) = stream.next().await {
                        let chunk = part?.chunk;
                        writer.write_all(&chunk).await?;
                    }
                }

                tokio::task::spawn_blocking({
                    let artist_name = artist.name.clone();
                    let album_name = album.name.clone();
                    let track_name = track.name.clone();
                    let genre = artist
                        .genres
                        .iter()
                        .map(|g| g.as_str())
                        .chain(album.genres.iter().map(|g| g.as_str()))
                        .collect::<Vec<_>>()
                        .join(";");
                    let download_path = download_path.clone();
                    move || {
                        let file = std::fs::File::open(&download_path)
                            .with_context(|| format!("opening file {}", download_path.display()))?;
                        let reader = std::io::BufReader::new(file);
                        let probe = lofty::Probe::new(reader).set_file_type(lofty::FileType::Mpeg);
                        let mut tagged = probe
                            .read()
                            .with_context(|| format!("reading file {}", download_path.display()))?;
                        let tag_type = tagged.primary_tag_type();
                        tagged.insert_tag(lofty::Tag::new(tag_type));

                        let tag = tagged.primary_tag_mut().unwrap();
                        tag.set_artist(artist_name);
                        tag.set_album(album_name);
                        tag.set_title(track_name);
                        tag.set_disk(disc_number);
                        tag.set_track(track_number);
                        tag.set_genre(genre);

                        Ok::<_, eyre::Error>(())
                    }
                })
                .await??;

                tokio::fs::rename(&download_path, &track_path)
                    .await
                    .with_context(|| {
                        format!(
                            "renaming {} to {}",
                            download_path.display(),
                            track_path.display()
                        )
                    })?;

                tracing::info!("track downloaded: {}", track_path.display());
                Ok::<_, eyre::Error>(())
            });
            handles.push(handle);
        } else {
            tracing::info!("track already exists: {}", track_path.display());
        }

        if !cover_path_jpg.exists()
            && !cover_path_png.exists()
            && (album.coverart_id.is_some() || artist.coverart_id.is_some())
        {
            tracing::info!("downloading cover: {}", parent_dir.display());
            let mut client = client.clone();
            let album_dir = parent_dir.clone();
            let image_id = album
                .coverart_id
                .as_ref()
                .or(artist.coverart_id.as_ref())
                .cloned()
                .unwrap();

            let handle = tokio::spawn(async move {
                let response = client
                    .image_download(sonar_grpc::ImageDownloadRequest {
                        image_id: image_id.to_string(),
                    })
                    .await
                    .with_context(|| format!("downloading image {}", image_id))?;
                let mut stream = response.into_inner();

                let mut chunks = Vec::new();
                let mut mime_type = None;
                while let Some(part) = stream.next().await {
                    let part = part?;
                    if mime_type.is_none() {
                        mime_type = Some(part.mime_type);
                    }
                    chunks.push(part.content);
                }

                let filetype = match mime_type.as_deref() {
                    Some("image/png") => "png",
                    Some("image/jpeg") => "jpg",
                    _ => {
                        tracing::warn!("unsupported mime type: {:?}", mime_type);
                        return Ok::<_, eyre::Error>(());
                    }
                };

                let cover_path = album_dir.join("cover").with_extension(filetype);
                tokio::fs::write(&cover_path, chunks.concat())
                    .await
                    .with_context(|| format!("writing file {}", cover_path.display()))?;

                tracing::info!("cover downloaded: {}", cover_path.display());
                Ok::<_, eyre::Error>(())
            });
            handles.push(handle);
        } else {
            tracing::info!("cover already exists: {}", parent_dir.display());
        }
    }

    let mut errors = 0;
    for handle in handles {
        if let Err(err) = handle.await? {
            errors += 1;
            tracing::error!("error: {}", err);
        }
    }

    tracing::info!("sync complete: {} errors", errors);
    Ok(())
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
struct SearchArgs {
    query: String,

    #[clap(long, default_value = "50")]
    limit: u32,

    #[clap(long)]
    artist: bool,

    #[clap(long)]
    album: bool,

    #[clap(long)]
    track: bool,

    #[clap(long)]
    playlist: bool,
}

async fn cmd_search(args: SearchArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;

    let mut request = sonar_grpc::SearchRequest::default();
    request.user_id = user_id;
    request.query = args.query;
    request.limit = Some(args.limit);
    if args.artist || args.album || args.track || args.playlist {
        let mut flags = 0;
        if args.artist {
            flags |= sonar_grpc::search_request::Flags::FlagArtist as u32;
        }
        if args.album {
            flags |= sonar_grpc::search_request::Flags::FlagAlbum as u32;
        }
        if args.track {
            flags |= sonar_grpc::search_request::Flags::FlagTrack as u32;
        }
        if args.playlist {
            flags |= sonar_grpc::search_request::Flags::FlagPlaylist as u32;
        }
        request.flags = Some(flags);
    }

    let response = client.search(request).await?;
    let response = response.into_inner();
    let mut results = vec![];
    for result in response.results {
        results.push(match result.result.unwrap() {
            sonar_grpc::search_result::Result::Artist(artist) => {
                SearchResult::Artist(Artist::from(artist))
            }
            sonar_grpc::search_result::Result::Album(album) => {
                SearchResult::Album(Album::from(album))
            }
            sonar_grpc::search_result::Result::Track(track) => {
                SearchResult::Track(Track::from(track))
            }
            sonar_grpc::search_result::Result::Playlist(playlist) => {
                SearchResult::Playlist(Playlist::from(playlist))
            }
        });
    }

    let results = SearchResults(results);
    stdout_value(results)?;

    Ok(())
}

#[derive(Debug, Parser)]
struct SubscriptionArgs {
    #[clap(subcommand)]
    command: SubscriptionCommand,
}

#[derive(Debug, Parser)]
enum SubscriptionCommand {
    List(SubscriptionListArgs),
    Create(SubscriptionCreateArgs),
    Delete(SubscriptionDeleteArgs),
}

#[derive(Debug, Parser)]
struct SubscriptionListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_subscription_list(_args: SubscriptionListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .subscription_list(sonar_grpc::SubscriptionListRequest { user_id })
        .await?;
    let subscriptions = response
        .into_inner()
        .subscriptions
        .into_iter()
        .map(Subscription::from)
        .collect::<Vec<_>>();
    stdout_values(&subscriptions)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct SubscriptionCreateArgs {
    external_id: String,
}

async fn cmd_subscription_create(args: SubscriptionCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    client
        .subscription_create(sonar_grpc::SubscriptionCreateRequest {
            user_id,
            external_id: args.external_id,
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct SubscriptionDeleteArgs {
    /// delete by subscription index, starts at 0.
    #[clap(long)]
    index: bool,
    external_id: String,
}

async fn cmd_subscription_delete(args: SubscriptionDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let external_id = if args.index {
        let index = args
            .external_id
            .parse::<usize>()
            .context("invalid subscription index")?;
        let response = client
            .subscription_list(sonar_grpc::SubscriptionListRequest {
                user_id: user_id.clone(),
            })
            .await?;
        let subscriptions = response
            .into_inner()
            .subscriptions
            .into_iter()
            .map(Subscription::from)
            .collect::<Vec<_>>();
        subscriptions[index].external_id.clone()
    } else {
        args.external_id
    };
    client
        .subscription_delete(sonar_grpc::SubscriptionDeleteRequest {
            user_id,
            external_id,
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct DownloadArgs {
    #[clap(subcommand)]
    command: DownloadCommand,
}

#[derive(Debug, Parser)]
enum DownloadCommand {
    List(DownloadListArgs),
    Start(DownloadStartArgs),
    Stop(DownloadStopArgs),
}

#[derive(Debug, Parser)]
struct DownloadListArgs {}

async fn cmd_download_list(_args: DownloadListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    let response = client
        .download_list(sonar_grpc::DownloadListRequest {
            user_id,
            ..Default::default()
        })
        .await?;
    let downloads = response
        .into_inner()
        .downloads
        .into_iter()
        .map(Download::from)
        .collect::<Vec<_>>();
    stdout_values(&downloads)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct DownloadStartArgs {
    external_id: String,
}

async fn cmd_download_start(args: DownloadStartArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    client
        .download_start(sonar_grpc::DownloadStartRequest {
            user_id,
            external_id: args.external_id,
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct DownloadStopArgs {
    external_id: String,
}

async fn cmd_download_stop(args: DownloadStopArgs) -> Result<()> {
    let mut client = create_client().await?;
    let (user_id, _) = auth_read().await?;
    client
        .download_cancel(sonar_grpc::DownloadCancelRequest {
            user_id,
            external_id: args.external_id,
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataArgs {
    #[clap(subcommand)]
    command: MetadataCommand,
}

#[derive(Debug, Parser)]
enum MetadataCommand {
    Providers,
    Fetch(MetadataFetchArgs),
}

async fn cmd_metadata_providers() -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .metadata_providers(sonar_grpc::MetadataProvidersRequest::default())
        .await?;
    let providers = response.into_inner().providers;
    stdout_values(&providers)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataFetchArgs {
    id: sonar::SonarId,

    /// comma separated list of providers to fetch metadata from.
    /// if not provided, fetch from all providers.
    #[clap(long)]
    providers: Option<String>,

    /// comma separated list of fields to fetch.
    /// if not provided, fetch all fields.
    /// possible fields are: name, genres, properties, cover.
    #[clap(long)]
    fields: Option<String>,
}

async fn cmd_metadata_fetch(args: MetadataFetchArgs) -> Result<()> {
    let mut client = create_client().await?;
    let providers = args
        .providers
        .map(|x| x.split(',').map(|x| x.to_string()).collect())
        .unwrap_or_default();
    let fields = args
        .fields
        .map(|x| x.split(',').map(|x| x.to_string()).collect())
        .unwrap_or_default();
    match args.id {
        sonar::SonarId::Artist(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Artist as i32,
                    item_id: args.id.to_string(),
                    fields,
                    providers,
                })
                .await?;
        }
        sonar::SonarId::Album(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Albumtracks as i32,
                    item_id: args.id.to_string(),
                    fields: fields.clone(),
                    providers: providers.clone(),
                })
                .await?;

            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Album as i32,
                    item_id: args.id.to_string(),
                    fields,
                    providers,
                })
                .await?;
        }
        sonar::SonarId::Track(_) => {
            client
                .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                    kind: sonar_grpc::MetadataFetchKind::Album as i32,
                    item_id: args.id.to_string(),
                    fields,
                    providers,
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
struct AdminArgs {
    #[clap(subcommand)]
    command: AdminCommand,
}

#[derive(Debug, Parser)]
enum AdminCommand {
    User(AdminUserArgs),
    Playlist(AdminPlaylistArgs),
    MetadataPreview(AdminMetadataPreviewArgs),
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

    #[clap(long)]
    admin: bool,
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
            admin: Some(args.admin),
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

    #[clap(long)]
    admin: Option<bool>,
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
            admin: args.admin,
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
    stdout_value(Playlist::from(response.into_inner()))?;
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
    stdout_value(Playlist::from(response.into_inner()))?;
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
struct AdminMetadataPreviewArgs {
    id: sonar::SonarId,
}

async fn cmd_admin_metadata_preview(args: AdminMetadataPreviewArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .metadata_album_tracks(sonar_grpc::MetadataAlbumTracksRequest {
            album_id: args.id.to_string(),
        })
        .await?
        .into_inner();
    println!("{:#?}", response);
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

    #[clap(long, env = "SONAR_DEFAULT_ADMIN_USERNAME")]
    default_admin_username: Option<String>,

    #[clap(long, env = "SONAR_DEFAULT_ADMIN_PASSWORD")]
    default_admin_password: Option<String>,

    #[clap(long, env = "SONAR_SPOTIFY_USERNAME")]
    spotify_username: Option<String>,

    #[clap(long, env = "SONAR_SPOTIFY_PASSWORD")]
    spotify_password: Option<String>,

    #[clap(long, env = "SONAR_SPOTIFY_CLIENT_ID")]
    spotify_client_id: Option<String>,

    #[clap(long, env = "SONAR_SPOTIFY_CLIENT_SECRET")]
    spotify_client_secret: Option<String>,

    #[clap(long)]
    jaeger_exporter: bool,

    #[clap(long)]
    jaeger_exporter_endpoint: Option<String>,
}

#[derive(Debug, Parser)]
struct ImportArgs {
    paths: Vec<PathBuf>,
}

async fn cmd_import(args: ImportArgs) -> Result<()> {
    tracing::info!("scanning for files");
    let files = list_files_in_directories(args.paths)
        .await?
        .into_iter()
        .filter(|path| {
            IMPORT_FILETYPES
                .iter()
                .any(|filetype| path.extension().map(|x| x == *filetype).unwrap_or(false))
        });

    let semaphore = Arc::new(tokio::sync::Semaphore::new(16));
    let mut handles = Vec::new();
    for filepath in files {
        let mut client = create_client().await?;
        let permit = semaphore.clone().acquire_owned().await;
        let handle = tokio::spawn(async move {
            tracing::info!("importing {}", filepath.display());
            let _permit = permit;
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
            Ok::<(), eyre::Report>(())
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(err) = handle.await? {
            tracing::error!("import failed: {}", err);
        }
    }

    Ok(())
}

async fn cmd_server(args: ServerArgs) -> Result<()> {
    let data_dir = args
        .data_dir
        .canonicalize()
        .context("canonicalizing data dir")?;
    let storage_dir = data_dir.join("storage");
    let database_url = data_dir.join("sonar.db").display().to_string();

    let spotify_cache = data_dir.join("spotify");
    tokio::fs::create_dir_all(&spotify_cache).await?;

    tracing::info!("starting sonar server");
    tracing::info!("\taddress: {}", args.address);
    tracing::info!("\tdatabase: {}", database_url);
    tracing::info!("\tstorage: {}", storage_dir.display());
    tracing::info!("\tspotify_cache: {}", spotify_cache.display());

    let storage_backend = sonar::StorageBackend::Filesystem {
        path: data_dir.join("storage"),
    };
    let search_backend = sonar::SearchBackend::BuiltIn;
    let mut config = sonar::Config::new(database_url, storage_backend, search_backend);
    config
        .register_extractor("lofty", sonar_extractor_lofty::LoftyExtractor)
        .context("registering lofty extractor")?;
    config
        .register_provider("beets", sonar_beets::BeetsMetadataProvider)
        .context("registering beets metadata importer")?;
    // config
    //     .register_scrobbler_for_user(
    //         "listenbrainz/admin",
    //         "admin".parse()?,
    //         sonar_listenbrainz::ListenBrainzScrobbler::new(std::env!("LISTENBRAINZ_API_KEY")),
    //     )
    //     .context("registering listenbrainz scrobbler")?;

    if let (Some(username), Some(password)) = (args.spotify_username, args.spotify_password) {
        let spotify = sonar_spotify::SpotifyService::new(
            sonar_spotify::LoginCredentials { username, password },
            &spotify_cache,
        )
        .await
        .context("creating spotify context")?;
        config
            .register_external_service(1, "spotify", spotify)
            .context("registering spotify")?;
    }

    if let (Some(client_id), Some(client_secret)) =
        (args.spotify_client_id, args.spotify_client_secret)
    {
        let spotify = sonar_spotify::SpotifyMetadata::new(client_id, client_secret)
            .await
            .context("creating spotify metadata provider")?;
        config
            .register_provider("spotify", spotify)
            .context("registering spotify metadata provider")?;
    }

    let context = sonar::new(config).await.context("creating sonar context")?;
    if let (Some(default_username), Some(default_password)) =
        (args.default_admin_username, args.default_admin_password)
    {
        let username: sonar::Username = default_username
            .parse()
            .context("parsing default username")?;
        if sonar::user_lookup(&context, &username).await?.is_none() {
            tracing::info!("creating default admin user {}", username);
            sonar::user_create(
                &context,
                sonar::UserCreate {
                    username,
                    password: default_password,
                    avatar: Default::default(),
                    admin: true,
                },
            )
            .await?;
        } else {
            tracing::info!("default admin user {} already exists", username);
        }
    }

    let grpc_context = context.clone();
    let f0 = tokio::spawn(async move {
        sonar_grpc::start_server(args.address, grpc_context)
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

// #[derive(Debug, Parser)]
// struct ExtractorArgs {
//     files: Vec<PathBuf>,
// }
//
// async fn cmd_extractor(args: ExtractorArgs) -> Result<()> {
//     use sonar::extractor::Extractor;
//
//     let extractors = [("lofty", Box::new(sonar_extractor_lofty::LoftyExtractor))];
//
//     for file in args.files {
//         for extractor in extractors.iter() {
//             tracing::info!(
//                 "extracting metadata from {} using {}",
//                 file.display(),
//                 extractor.0
//             );
//             let extracted = match extractor.1.extract(&file) {
//                 Ok(extracted) => extracted,
//                 Err(err) => {
//                     tracing::warn!(
//                         "failed to extract metadata from {}: {}",
//                         file.display(),
//                         err
//                     );
//                     continue;
//                 }
//             };
//
//             tracing::info!("extracted metadata:\n{:#?}", extracted);
//         }
//     }
//
//     Ok(())
// }

async fn list_files_in_directories(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    type Sender = tokio::sync::mpsc::UnboundedSender<PathBuf>;

    fn scan_path(
        tx: Sender,
        path: PathBuf,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + Sync + 'static>> {
        Box::pin(async move {
            if path.is_dir() {
                let mut entries = tokio::fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    tokio::spawn(scan_path(tx.clone(), entry.path()));
                }
            } else {
                match path.canonicalize() {
                    Ok(p) => tx.send(p).unwrap(),
                    Err(err) => {
                        tracing::warn!("failed to canonicalize {}: {}", path.display(), err)
                    }
                }
            }
            Ok(())
        })
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    for path in paths {
        tokio::spawn(scan_path(tx.clone(), path));
    }
    drop(tx);

    let mut files = Vec::new();
    while let Some(file) = rx.recv().await {
        files.push(file);
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

fn genres_from_pb(genres: Vec<String>) -> Genres {
    Genres::new(genres).expect("received invalid genres from server")
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
