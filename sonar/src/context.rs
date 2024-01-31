use std::{path::PathBuf, sync::Arc};

use sqlx::Executor;

use crate::{
    album, artist, audio,
    blob::{self, BlobStorage},
    bytestream::{self},
    db::Db,
    extractor::{Extractor, SonarExtractor},
    image,
    importer::{self, Importer},
    metadata::{
        AlbumMetadata, AlbumMetadataRequest, AlbumTracksMetadata, AlbumTracksMetadataRequest,
        MetadataProvider, MetadataRequestKind, SonarMetadataProvider,
    },
    playlist, scrobble,
    scrobbler::{self, SonarScrobbler},
    track, user, Album, AlbumCreate, AlbumId, AlbumUpdate, Artist, ArtistCreate, ArtistId,
    ArtistUpdate, Audio, AudioCreate, AudioDownload, AudioId, ByteRange, Error, ErrorKind,
    ImageCreate, ImageDownload, ImageId, Import, ListParams, Lyrics, Playlist, PlaylistCreate,
    PlaylistId, PlaylistTrack, PlaylistUpdate, Result, Scrobble, ScrobbleCreate, ScrobbleId,
    ScrobbleUpdate, Track, TrackCreate, TrackId, TrackUpdate, User, UserCreate, UserId, Username,
    ValueUpdate,
};

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
    providers: Vec<SonarMetadataProvider>,
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
            providers: Vec::new(),
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

    pub fn register_provider(
        &mut self,
        name: impl Into<String>,
        provider: impl MetadataProvider,
    ) -> Result<()> {
        let name = name.into();
        if self.providers.iter().any(|p| p.name() == name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "provider already registered",
            ));
        }
        self.providers
            .push(SonarMetadataProvider::new(name, provider));
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
    providers: Arc<Vec<SonarMetadataProvider>>,
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
        providers: Arc::new(config.providers),
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

pub async fn user_authenticate(
    context: &Context,
    username: &Username,
    password: &str,
) -> Result<UserId> {
    let mut conn = context.db.acquire().await?;
    user::authenticate(&mut conn, username, password).await
}

pub async fn artist_list(context: &Context, params: ListParams) -> Result<Vec<Artist>> {
    let mut conn = context.db.acquire().await?;
    artist::list(&mut conn, params).await
}

pub async fn artist_get(context: &Context, artist_id: ArtistId) -> Result<Artist> {
    let mut conn = context.db.acquire().await?;
    artist::get(&mut conn, artist_id).await
}

pub async fn artist_get_bulk(context: &Context, artist_ids: &[ArtistId]) -> Result<Vec<Artist>> {
    let mut conn = context.db.acquire().await?;
    artist::get_bulk(&mut conn, artist_ids).await
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

pub async fn album_get_bulk(context: &Context, album_ids: &[AlbumId]) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::get_bulk(&mut conn, album_ids).await
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

pub async fn track_get_bulk(context: &Context, track_ids: &[TrackId]) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::get_bulk(&mut conn, track_ids).await
}

pub async fn track_create(context: &Context, create: TrackCreate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::create(&mut tx, create).await;
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
) -> Result<AudioDownload> {
    let mut conn = context.db.acquire().await?;
    track::download(&mut conn, &*context.storage, track_id, range).await
}

pub async fn track_get_lyrics(context: &Context, track_id: TrackId) -> Result<Lyrics> {
    let mut conn = context.db.acquire().await?;
    track::get_lyrics(&mut conn, track_id).await
}

pub async fn audio_create(context: &Context, create: AudioCreate) -> Result<Audio> {
    let mut tx = context.db.begin().await?;
    let result = audio::create(&mut tx, &*context.storage, create).await;
    tx.commit().await?;
    result
}

pub async fn audio_delete(context: &Context, audio_id: AudioId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = audio::delete(&mut tx, audio_id).await;
    tx.commit().await?;
    result
}

pub async fn audio_link(context: &Context, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = audio::link(&mut tx, audio_id, track_id).await;
    tx.commit().await?;
    result
}

pub async fn audio_unlink(context: &Context, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = audio::unlink(&mut tx, audio_id, track_id).await;
    tx.commit().await?;
    result
}

pub async fn audio_set_preferred(
    context: &Context,
    audio_id: AudioId,
    track_id: TrackId,
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    let result = audio::set_preferred(&mut tx, audio_id, track_id).await;
    tx.commit().await?;
    result
}

pub async fn playlist_list(context: &Context, params: ListParams) -> Result<Vec<Playlist>> {
    let mut conn = context.db.acquire().await?;
    playlist::list(&mut conn, params).await
}

pub async fn playlist_get(context: &Context, playlist_id: PlaylistId) -> Result<Playlist> {
    let mut conn = context.db.acquire().await?;
    playlist::get(&mut conn, playlist_id).await
}

pub async fn playlist_get_by_name(
    context: &Context,
    user_id: UserId,
    name: &str,
) -> Result<Playlist> {
    let mut conn = context.db.acquire().await?;
    playlist::get_by_name(&mut conn, user_id, name).await
}

pub async fn playlist_find_by_name(
    context: &Context,
    user_id: UserId,
    name: &str,
) -> Result<Option<Playlist>> {
    let mut conn = context.db.acquire().await?;
    playlist::find_by_name(&mut conn, user_id, name).await
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

pub async fn metadata_fetch_album(context: &Context, album_id: AlbumId) -> Result<()> {
    let metadata = metadata_view_album(context, album_id).await?;

    let image_id = if let Some(cover) = metadata.cover {
        match image_create(
            context,
            ImageCreate {
                data: bytestream::from_bytes(cover),
            },
        )
        .await
        {
            Ok(image_id) => Some(image_id),
            Err(err) => {
                tracing::warn!("failed to create image: {}", err);
                None
            }
        }
    } else {
        None
    };

    let mut update = AlbumUpdate::default();
    update.name = ValueUpdate::from_option_unchanged(metadata.name);
    update.properties = metadata.properties.into_property_updates();
    update.cover_art = ValueUpdate::from_option_unchanged(image_id);
    album_update(context, album_id, update).await?;
    Ok(())
}

pub async fn metadata_fetch_album_tracks(context: &Context, album_id: AlbumId) -> Result<()> {
    let metadata = metadata_view_album_tracks(context, album_id).await?;
    for (track_id, track_metadata) in metadata.tracks {
        let mut update = TrackUpdate::default();
        // TODO: update cover
        update.name = ValueUpdate::from_option_unchanged(track_metadata.name);
        update.properties = track_metadata.properties.into_property_updates();
        track_update(context, track_id, update).await?;
    }
    Ok(())
}

pub async fn metadata_view_artist(_context: &Context, _artist_id: ArtistId) -> Result<()> {
    todo!()
}

pub async fn metadata_view_album(context: &Context, album_id: AlbumId) -> Result<AlbumMetadata> {
    let album = album_get(context, album_id).await?;
    let artist = artist_get(context, album.artist).await?;
    let request = AlbumMetadataRequest { artist, album };
    for fetcher in context.providers.iter() {
        if !fetcher.supports(MetadataRequestKind::Album) {
            continue;
        }
        match fetcher.album_metadata(context, &request).await {
            Ok(metadata) => return Ok(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch album metadata from provider '{}': {}",
                    fetcher.name(),
                    err
                );
            }
        }
    }
    Ok(Default::default())
}

pub async fn metadata_view_album_tracks(
    context: &Context,
    album_id: AlbumId,
) -> Result<AlbumTracksMetadata> {
    let album = album_get(context, album_id).await?;
    let artist = artist_get(context, album.artist).await?;
    let tracks = track_list_by_album(context, album_id, ListParams::default()).await?;
    let request = AlbumTracksMetadataRequest {
        artist,
        album,
        tracks,
    };
    for fetcher in context.providers.iter() {
        if !fetcher.supports(MetadataRequestKind::AlbumTracks) {
            continue;
        }
        match fetcher.album_tracks_metadata(context, &request).await {
            Ok(metadata) => return Ok(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch album tracks metadata from provider '{}': {}",
                    fetcher.name(),
                    err
                );
            }
        }
    }
    Ok(Default::default())
}

pub async fn metadata_view_track(_context: &Context, _track_id: TrackId) -> Result<()> {
    todo!()
}
