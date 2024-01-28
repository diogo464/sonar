#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(concat_idents)]
#![feature(backtrace_frames)]

use std::{path::PathBuf, sync::Arc, time::Duration};

use bytes::Bytes;
use importer::Importer;
use metadata::{Extractor, SonarExtractor};
use scrobbler::SonarScrobbler;
use sqlx::Executor;

pub use async_trait::async_trait;

#[doc(hidden)]
#[cfg(feature = "test-utilities")]
pub mod test;

mod error;
pub use error::*;

mod id;
pub use id::*;

mod value_update;
pub use value_update::*;

mod timestamp;
pub use timestamp::*;

pub mod bytestream;
pub use bytestream::ByteStream;

pub mod metadata;
pub mod scrobbler;

mod blob;
pub(crate) use blob::BlobStorage;

pub(crate) mod genre;
pub use genre::{Genre, GenreUpdate, GenreUpdateAction, Genres, InvalidGenreError};

pub(crate) mod property;
pub use property::{
    InvalidPropertyKeyError, InvalidPropertyValueError, Properties, PropertyKey, PropertyUpdate,
    PropertyUpdateAction, PropertyValue,
};

pub(crate) mod user;
pub use user::Username;

pub(crate) mod importer;
pub use importer::Import;

pub(crate) mod album;
pub(crate) mod artist;
pub(crate) mod image;
pub(crate) mod ks;
pub(crate) mod playlist;
pub(crate) mod scrobble;
pub(crate) mod track;

pub(crate) type Db = sqlx::SqlitePool;
pub(crate) type DbC = sqlx::SqliteConnection;

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Memory,
    Filesystem { path: PathBuf },
}

#[derive(Debug)]
pub struct Config {
    database_url: String,
    storage_backend: StorageBackend,
    extractors: Vec<SonarExtractor>,
    scrobblers: Vec<SonarScrobbler>,
    max_import_size: usize,
    max_parallel_imports: usize,
}

impl Config {
    pub fn new(database_url: impl Into<String>, storage_backend: StorageBackend) -> Self {
        Self {
            database_url: database_url.into(),
            storage_backend,
            extractors: Vec::new(),
            scrobblers: Vec::new(),
            max_import_size: 1024 * 1024 * 1024,
            max_parallel_imports: 8,
        }
    }

    pub fn register_extractor(
        &mut self,
        name: impl Into<String>,
        extractor: impl Extractor,
    ) -> Result<()> {
        let name = name.into();
        if self.extractors.iter().any(|e| e.name() == name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "extractor already registered",
            ));
        }
        self.extractors.push(SonarExtractor::new(name, extractor));
        Ok(())
    }

    pub fn register_scrobbler(
        &mut self,
        identifier: impl Into<String>,
        scrobbler: impl scrobbler::Scrobbler,
    ) -> Result<()> {
        let identifier = identifier.into();
        if self.scrobblers.iter().any(|s| s.identifier() == identifier) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "scrobbler already registered",
            ));
        }
        self.scrobblers
            .push(SonarScrobbler::new(identifier, scrobbler));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    db: Db,
    storage: Arc<dyn BlobStorage>,
    importer: Arc<Importer>,
    extractors: Arc<Vec<SonarExtractor>>,
    scrobblers: Arc<Vec<SonarScrobbler>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteRange {
    pub offset: Option<u64>,
    pub length: Option<u64>,
}

impl ByteRange {
    pub fn new(offset: u64, length: u64) -> Self {
        Self {
            offset: Some(offset),
            length: Some(length),
        }
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ListParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl From<(Option<u32>, Option<u32>)> for ListParams {
    fn from((offset, limit): (Option<u32>, Option<u32>)) -> Self {
        Self { offset, limit }
    }
}

impl From<(u32, u32)> for ListParams {
    fn from((offset, limit): (u32, u32)) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }
}

impl ListParams {
    pub fn new(offset: u32, limit: u32) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn to_db_offset_limit(self) -> (i64, i64) {
        let limit = self.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let offset = self.offset.map(|offset| offset as i64).unwrap_or(0);
        (offset, limit)
    }
}

pub struct ImageCreate {
    pub data: ByteStream,
}

pub struct ImageDownload {
    mime_type: String,
    stream: ByteStream,
}

impl ImageDownload {
    pub(crate) fn new(mime_type: String, stream: ByteStream) -> Self {
        Self { mime_type, stream }
    }
}

impl std::fmt::Debug for ImageDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageDownload")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}

impl tokio_stream::Stream for ImageDownload {
    type Item = std::io::Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut *self.get_mut().stream).poll_next(cx)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub avatar: Option<ImageId>,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub username: Username,
    pub password: String,
    pub avatar: Option<ImageId>,
}

#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub password: ValueUpdate<String>,
    pub avatar: ValueUpdate<ImageId>,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub album_count: u32,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ArtistCreate {
    pub name: String,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct ArtistUpdate {
    pub name: ValueUpdate<String>,
    pub cover_art: ValueUpdate<ImageId>,
    pub genres: Vec<GenreUpdate>,
    pub properties: Vec<PropertyUpdate>,
}

// TODO: add duration
// TODO: add created at
#[derive(Debug, Clone)]
pub struct Album {
    pub id: AlbumId,
    pub name: String,
    pub artist: ArtistId,
    pub release_date: DateTime,
    pub track_count: u32,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct AlbumCreate {
    pub name: String,
    pub artist: ArtistId,
    pub cover_art: Option<ImageId>,
    pub release_date: DateTime,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumUpdate {
    pub name: ValueUpdate<String>,
    pub artist: ValueUpdate<ArtistId>,
    pub release_date: ValueUpdate<DateTime>,
    pub cover_art: ValueUpdate<ImageId>,
    pub genres: Vec<GenreUpdate>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub artist: ArtistId,
    pub album: AlbumId,
    pub disc_number: u32,
    pub track_number: u32,
    pub duration: Duration,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct TrackLyrics {
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

pub struct TrackCreate {
    pub name: String,
    pub album: AlbumId,
    pub disc_number: Option<u32>,
    pub track_number: Option<u32>,
    pub duration: Duration,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub lyrics: Option<TrackLyrics>,
    pub properties: Properties,
    pub audio_stream: ByteStream,
    pub audio_filename: String,
}

impl std::fmt::Debug for TrackCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackCreate")
            .field("name", &self.name)
            .field("album", &self.album)
            .field("disc_number", &self.disc_number)
            .field("track_number", &self.track_number)
            .field("duration", &self.duration)
            .field("cover_art", &self.cover_art)
            .field("genres", &self.genres)
            .field("lyrics", &self.lyrics)
            .field("properties", &self.properties)
            .field("audio_filename", &self.audio_filename)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct TrackUpdate {
    pub name: ValueUpdate<String>,
    pub album: ValueUpdate<AlbumId>,
    pub disc_number: ValueUpdate<u32>,
    pub track_number: ValueUpdate<u32>,
    pub cover_art: ValueUpdate<ImageId>,
    pub genres: Vec<GenreUpdate>,
    pub lyrics: ValueUpdate<TrackLyrics>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone)]
pub struct PlaylistTrack {
    pub playlist: PlaylistId,
    pub track: TrackId,
    pub inserted_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub owner: UserId,
    pub track_count: u32,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct PlaylistCreate {
    pub name: String,
    pub owner: UserId,
    pub tracks: Vec<TrackId>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct PlaylistUpdate {
    pub name: ValueUpdate<String>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone)]
pub struct Scrobble {
    pub id: ScrobbleId,
    pub user: UserId,
    pub track: TrackId,
    pub listen_at: Timestamp,
    pub listen_duration: Duration,
    pub listen_device: String,
    pub created_at: Timestamp,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ScrobbleCreate {
    pub user: UserId,
    pub track: TrackId,
    pub listen_at: Timestamp,
    pub listen_duration: Duration,
    pub listen_device: String,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ScrobbleUpdate {
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LyricsKind {
    Synced,
    Unsynced,
}

#[derive(Debug, Clone)]
pub struct LyricsLine {
    pub offset: Duration,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Lyrics {
    pub track: TrackId,
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

pub async fn new(config: Config) -> Result<Context> {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&config.database_url)
        .await
        .map_err(|e| Error::with_source(ErrorKind::Internal, "failed to connect to database", e))?;
    db.execute("PRAGMA foreign_keys = ON").await?;
    sqlx::migrate!("./migrations").run(&db).await.map_err(|e| {
        Error::with_source(ErrorKind::Internal, "failed to run database migrations", e)
    })?;

    let storage = match config.storage_backend {
        StorageBackend::Memory => {
            Arc::new(blob::MemoryBlobStorage::default()) as Arc<dyn BlobStorage>
        }
        StorageBackend::Filesystem { ref path } => {
            Arc::new(blob::FilesystemBlobStorage::new(path.clone())) as Arc<dyn BlobStorage>
        }
    };

    let importer = importer::new(importer::Config {
        max_import_size: config.max_import_size,
        max_concurrent_imports: config.max_parallel_imports,
    });

    Ok(Context {
        db,
        storage,
        importer: Arc::new(importer),
        extractors: Arc::new(config.extractors),
        scrobblers: Arc::new(config.scrobblers),
    })
}

pub async fn image_create(context: &Context, create: ImageCreate) -> Result<ImageId> {
    let mut tx = context.db.begin().await?;
    let result = image::create(&mut tx, &*context.storage, create).await;
    tx.commit().await?;
    result
}

pub async fn image_delete(context: &Context, image_id: ImageId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = image::delete(&mut tx, image_id).await;
    tx.commit().await?;
    result
}

pub async fn image_download(context: &Context, image_id: ImageId) -> Result<ImageDownload> {
    let mut conn = context.db.acquire().await?;
    image::download(&mut conn, &*context.storage, image_id).await
}

pub async fn user_list(context: &Context, params: ListParams) -> Result<Vec<User>> {
    let mut conn = context.db.acquire().await?;
    user::list(&mut conn, params).await
}

pub async fn user_create(context: &Context, create: UserCreate) -> Result<User> {
    let mut tx = context.db.begin().await?;
    let result = user::create(&mut tx, create).await;
    tx.commit().await?;
    result
}

pub async fn user_get(context: &Context, user_id: UserId) -> Result<User> {
    let mut conn = context.db.acquire().await?;
    user::get(&mut conn, user_id).await
}

pub async fn user_delete(context: &Context, user_id: UserId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = user::delete(&mut tx, user_id).await;
    tx.commit().await?;
    result
}

pub async fn artist_list(context: &Context, params: ListParams) -> Result<Vec<Artist>> {
    let mut conn = context.db.acquire().await?;
    artist::list(&mut conn, params).await
}

pub async fn artist_get(context: &Context, artist_id: ArtistId) -> Result<Artist> {
    let mut conn = context.db.acquire().await?;
    artist::get(&mut conn, artist_id).await
}

pub async fn artist_create(context: &Context, create: ArtistCreate) -> Result<Artist> {
    let mut tx = context.db.begin().await?;
    let result = artist::create(&mut tx, create).await;
    tx.commit().await?;
    result
}

pub async fn artist_update(
    context: &Context,
    id: ArtistId,
    update: ArtistUpdate,
) -> Result<Artist> {
    let mut tx = context.db.begin().await?;
    let result = artist::update(&mut tx, id, update).await;
    tx.commit().await?;
    result
}

pub async fn artist_delete(context: &Context, id: ArtistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = artist::delete(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn album_list(context: &Context, params: ListParams) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::list(&mut conn, params).await
}

pub async fn album_list_by_artist(
    context: &Context,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::list_by_artist(&mut conn, artist_id, params).await
}

pub async fn album_get(context: &Context, album_id: AlbumId) -> Result<Album> {
    let mut conn = context.db.acquire().await?;
    album::get(&mut conn, album_id).await
}

pub async fn album_create(context: &Context, create: AlbumCreate) -> Result<Album> {
    let mut tx = context.db.begin().await?;
    let result = album::create(&mut tx, create).await;
    tx.commit().await?;
    result
}

pub async fn album_update(context: &Context, id: AlbumId, update: AlbumUpdate) -> Result<Album> {
    let mut tx = context.db.begin().await?;
    let result = album::update(&mut tx, id, update).await;
    tx.commit().await?;
    result
}

pub async fn album_delete(context: &Context, id: AlbumId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = album::delete(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn track_list(context: &Context, params: ListParams) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list(&mut conn, params).await
}

pub async fn track_list_by_album(
    context: &Context,
    album_id: AlbumId,
    params: ListParams,
) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list_by_album(&mut conn, album_id, params).await
}

pub async fn track_get(context: &Context, track_id: TrackId) -> Result<Track> {
    let mut conn = context.db.acquire().await?;
    track::get(&mut conn, track_id).await
}

pub async fn track_create(context: &Context, create: TrackCreate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::create(&mut tx, &*context.storage, create).await;
    tx.commit().await?;
    result
}

pub async fn track_update(context: &Context, id: TrackId, update: TrackUpdate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::update(&mut tx, id, update).await;
    tx.commit().await?;
    result
}

pub async fn track_delete(context: &Context, id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = track::delete(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn track_download(
    context: &Context,
    track_id: TrackId,
    range: ByteRange,
) -> Result<ByteStream> {
    let mut conn = context.db.acquire().await?;
    track::download(&mut conn, &*context.storage, track_id, range).await
}

pub async fn track_get_lyrics(context: &Context, track_id: TrackId) -> Result<Lyrics> {
    let mut conn = context.db.acquire().await?;
    track::get_lyrics(&mut conn, track_id).await
}

pub async fn playlist_list(context: &Context, params: ListParams) -> Result<Vec<Playlist>> {
    let mut conn = context.db.acquire().await?;
    playlist::list(&mut conn, params).await
}

pub async fn playlist_get(context: &Context, playlist_id: PlaylistId) -> Result<Playlist> {
    let mut conn = context.db.acquire().await?;
    playlist::get(&mut conn, playlist_id).await
}

pub async fn playlist_create(context: &Context, create: PlaylistCreate) -> Result<Playlist> {
    let mut tx = context.db.begin().await?;
    let result = playlist::create(&mut tx, create).await;
    tx.commit().await?;
    result
}

pub async fn playlist_update(
    context: &Context,
    id: PlaylistId,
    update: PlaylistUpdate,
) -> Result<Playlist> {
    let mut tx = context.db.begin().await?;
    let result = playlist::update(&mut tx, id, update).await;
    tx.commit().await?;
    result
}

pub async fn playlist_delete(context: &Context, id: PlaylistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = playlist::delete(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn playlist_list_tracks(
    context: &Context,
    id: PlaylistId,
    params: ListParams,
) -> Result<Vec<PlaylistTrack>> {
    let mut conn = context.db.acquire().await?;
    playlist::list_tracks(&mut conn, id, params).await
}

pub async fn playlist_clear_tracks(context: &Context, id: PlaylistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = playlist::clear_tracks(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn playlist_insert_tracks(
    context: &Context,
    id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = playlist::insert_tracks(&mut tx, id, tracks).await;
    tx.commit().await?;
    result
}

pub async fn playlist_remove_tracks(
    context: &Context,
    id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = playlist::remove_tracks(&mut tx, id, tracks).await;
    tx.commit().await?;
    result
}

pub async fn scrobble_list(context: &Context, params: ListParams) -> Result<Vec<Scrobble>> {
    let mut conn = context.db.acquire().await?;
    scrobble::list(&mut conn, params).await
}

pub async fn scrobble_get(context: &Context, scrobble_id: ScrobbleId) -> Result<Scrobble> {
    let mut conn = context.db.acquire().await?;
    scrobble::get(&mut conn, scrobble_id).await
}

pub async fn scrobble_create(context: &Context, create: ScrobbleCreate) -> Result<Scrobble> {
    let mut tx = context.db.begin().await?;
    let result = scrobble::create(&mut tx, create).await;
    tx.commit().await?;
    result
}

pub async fn scrobble_update(
    context: &Context,
    id: ScrobbleId,
    update: ScrobbleUpdate,
) -> Result<Scrobble> {
    let mut tx = context.db.begin().await?;
    let result = scrobble::update(&mut tx, id, update).await;
    tx.commit().await?;
    result
}

pub async fn scrobble_delete(context: &Context, id: ScrobbleId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = scrobble::delete(&mut tx, id).await;
    tx.commit().await?;
    result
}

pub async fn import(context: &Context, import: Import) -> Result<Track> {
    importer::import(
        &context.importer,
        &context.db,
        &*context.storage,
        &context.extractors,
        import,
    )
    .await
}
