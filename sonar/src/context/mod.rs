use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;
use tokio::sync::Notify;

use crate::{
    album, artist, audio,
    blob::{self, BlobStorage},
    bytestream,
    db::Db,
    download,
    external::{ExternalService, ExternalServices, ExternalServicesEntry},
    extractor::{Extractor, SonarExtractor},
    favorite, gc,
    genre::GenreStats,
    image,
    importer::{self, Importer},
    metadata::{
        AlbumMetadata, AlbumMetadataRequest, AlbumTracksMetadata, AlbumTracksMetadataRequest,
        MetadataProvider, MetadataRequestKind, SonarMetadataProvider,
    },
    migrations, pin, playlist, property, scrobble,
    scrobbler::{self, SonarScrobbler},
    search::{BuiltInSearchEngine, MeiliSearchEngine, SearchEngine, SearchResults},
    subscription,
    track::{self, TrackListRandom},
    user, Album, AlbumCreate, AlbumId, AlbumUpdate, Artist, ArtistCreate, ArtistId, ArtistMetadata,
    ArtistMetadataRequest, ArtistUpdate, Audio, AudioCreate, AudioDownload, AudioId, AudioStat,
    ByteRange, Error, ErrorKind, ExternalMediaRequest, ExternalMediaType, Favorite, Genre, Genres,
    ImageCreate, ImageDownload, ImageId, Import, ListParams, Lyrics, MetadataFetchMask,
    MetadataFetchParams, Playlist, PlaylistCreate, PlaylistId, PlaylistTrack, PlaylistUpdate,
    Properties, PropertyKey, PropertyUpdate, Result, Scrobble, ScrobbleCreate, ScrobbleId,
    ScrobbleUpdate, SearchQuery, SonarId, Subscription, SubscriptionCreate, SubscriptionId, Track,
    TrackCreate, TrackId, TrackMetadata, TrackMetadataRequest, TrackUpdate, User, UserCreate,
    UserId, UserToken, UserUpdate, Username, ValueUpdate, METADATA_FETCH_MASK_COVER,
    METADATA_FETCH_MASK_GENRES, METADATA_FETCH_MASK_NAME, METADATA_FETCH_MASK_PROPERTIES,
};

mod memory_indexes;
use memory_indexes::*;

mod playlist_cover_process;
mod scrobbler_process;
mod subscription_process;

#[derive(Debug, Default, Clone)]
pub enum StorageBackend {
    #[default]
    Memory,
    Filesystem {
        path: PathBuf,
    },
}

#[derive(Debug, Default, Clone)]
pub enum SearchBackend {
    #[default]
    BuiltIn,
    Meilisearch {
        endpoint: String,
        key: String,
    },
}

#[derive(Debug)]
pub struct Config {
    database_url: String,
    storage_backend: StorageBackend,
    search_backend: SearchBackend,
    extractors: Vec<SonarExtractor>,
    scrobblers: Vec<SonarScrobbler>,
    providers: Vec<SonarMetadataProvider>,
    external: Vec<ExternalServicesEntry>,
    max_import_size: usize,
    max_parallel_imports: usize,
}

impl Config {
    pub fn new(
        database_url: impl Into<String>,
        storage_backend: StorageBackend,
        search_backend: SearchBackend,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            storage_backend,
            search_backend,
            extractors: Vec::new(),
            scrobblers: Vec::new(),
            providers: Vec::new(),
            external: Vec::new(),
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
        let scrobbler = SonarScrobbler::new(identifier.into(), None, scrobbler);
        self.register_scrobbler_internal(scrobbler)
    }

    pub fn register_scrobbler_for_user(
        &mut self,
        identifier: impl Into<String>,
        username: Username,
        scrobbler: impl scrobbler::Scrobbler,
    ) -> Result<()> {
        let scrobbler = SonarScrobbler::new(identifier.into(), Some(username), scrobbler);
        self.register_scrobbler_internal(scrobbler)
    }

    fn register_scrobbler_internal(&mut self, scrobber: SonarScrobbler) -> Result<()> {
        let identifier = scrobber.identifier();
        if self.scrobblers.iter().any(|s| s.identifier() == identifier) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "scrobbler already registered",
            ));
        }
        self.scrobblers.push(scrobber);
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

    /// register a new external service.
    /// names have to be unique.
    /// services with lower priority number have a higher precedence.
    pub fn register_external_service(
        &mut self,
        priority: u32,
        name: impl Into<String>,
        service: impl ExternalService,
    ) -> Result<()> {
        let name = name.into();
        if self.external.iter().any(|s| s.identifier == name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "external service already registered",
            ));
        }
        self.external.push(ExternalServicesEntry {
            identifier: name,
            service: Box::new(service),
            priority,
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) db: Db,
    tokens: Arc<Mutex<HashMap<UserToken, UserId>>>,
    storage: Arc<dyn BlobStorage>,
    importer: Arc<Importer>,
    search: Arc<dyn SearchEngine>,
    extractors: Arc<Vec<SonarExtractor>>,
    scrobblers: Arc<Vec<SonarScrobbler>>,
    providers: Arc<Vec<SonarMetadataProvider>>,
    external: ExternalServices,
    scrobbler_notify: Arc<Notify>,
    memory_indexes: Arc<Mutex<MemoryIndexes>>,
}

pub async fn new(config: Config) -> Result<Context> {
    let opts: sqlx::sqlite::SqliteConnectOptions = config.database_url.parse()?;
    let opts = opts
        .create_if_missing(true)
        .read_only(false)
        .foreign_keys(true)
        .pragma("cache_size", format!("{}", -64 * 1024))
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
        .map_err(|e| Error::with_source(ErrorKind::Internal, "failed to connect to database", e))?;
    migrations::run(&db).await?;

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

    let search_engine = match config.search_backend {
        SearchBackend::BuiltIn => {
            Arc::new(BuiltInSearchEngine::new(db.clone())) as Arc<dyn SearchEngine>
        }
        SearchBackend::Meilisearch { endpoint, key } => {
            Arc::new(MeiliSearchEngine::new(db.clone(), endpoint, key).await?)
                as Arc<dyn SearchEngine>
        }
    };

    let context = Context {
        db,
        tokens: Default::default(),
        storage,
        importer: Arc::new(importer),
        search: search_engine,
        extractors: Arc::new(config.extractors),
        scrobblers: Arc::new(config.scrobblers),
        providers: Arc::new(config.providers),
        external: ExternalServices::new(config.external),
        scrobbler_notify: Arc::new(Notify::new()),
        memory_indexes: Default::default(),
    };

    for scrobbler in context.scrobblers.iter().cloned() {
        tracing::info!("starting scrobbler: {}", scrobbler.identifier());
        tokio::spawn(scrobbler_process::run(
            context.clone(),
            scrobbler,
            context.scrobbler_notify.clone(),
        ));
    }

    tokio::spawn({
        let context = context.clone();
        async move { subscription_process::run(&context).await }
    });

    tokio::spawn({
        let context = context.clone();
        async move { playlist_cover_process::run(&context).await }
    });

    tokio::spawn({
        let context = context.clone();
        async move { update_listen_counts(&context).await }
    });

    memory_indexes_rebuild(&context).await;
    context.search.synchronize_all().await;

    context.scrobbler_notify.notify_waiters();

    Ok(context)
}

#[tracing::instrument(skip(context))]
pub async fn user_list(context: &Context, params: ListParams) -> Result<Vec<User>> {
    let mut conn = context.db.acquire().await?;
    user::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn user_create(context: &Context, create: UserCreate) -> Result<User> {
    let mut tx = context.db.begin().await?;
    let result = user::create(&mut tx, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn user_get(context: &Context, user_id: UserId) -> Result<User> {
    let mut conn = context.db.acquire().await?;
    user::get(&mut conn, user_id).await
}

#[tracing::instrument(skip(context))]
pub async fn user_update(context: &Context, user_id: UserId, update: UserUpdate) -> Result<User> {
    let mut tx = context.db.begin().await?;
    let result = user::update(&mut tx, user_id, update).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn user_lookup(context: &Context, username: &Username) -> Result<Option<UserId>> {
    let mut conn = context.db.acquire().await?;
    user::lookup(&mut conn, username).await
}

#[tracing::instrument(skip(context))]
pub async fn user_delete(context: &Context, user_id: UserId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    user::delete(&mut tx, user_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn user_authenticate(
    context: &Context,
    username: &Username,
    password: &str,
) -> Result<UserId> {
    let mut conn = context.db.acquire().await?;
    user::authenticate(&mut conn, username, password).await
}

#[tracing::instrument(skip(context))]
pub async fn user_login(
    context: &Context,
    username: &Username,
    password: &str,
) -> Result<(UserId, UserToken)> {
    let user_id = user_authenticate(context, username, password).await?;
    let token = UserToken::random();
    context
        .tokens
        .lock()
        .unwrap()
        .insert(token.clone(), user_id);
    Ok((user_id, token))
}

#[tracing::instrument(skip(context))]
pub async fn user_logout(context: &Context, token: &UserToken) -> Result<()> {
    context.tokens.lock().unwrap().remove(token);
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn user_validate_token(context: &Context, token: &UserToken) -> Result<UserId> {
    let tokens = context.tokens.lock().unwrap();
    if let Some(user_id) = tokens.get(token) {
        Ok(*user_id)
    } else {
        Err(Error::new(ErrorKind::Unauthorized, "invalid user token"))
    }
}

#[tracing::instrument(skip(context))]
pub async fn user_property_get(
    context: &Context,
    user_id: UserId,
    id: SonarId,
) -> Result<Properties> {
    let mut conn = context.db.acquire().await?;
    property::user_get(&mut conn, user_id, id).await
}

#[tracing::instrument(skip(context))]
pub async fn user_property_get_bulk(
    context: &Context,
    user_id: UserId,
    ids: &[SonarId],
) -> Result<Vec<Properties>> {
    let mut conn = context.db.acquire().await?;
    property::user_get_bulk(&mut conn, user_id, ids.iter().copied()).await
}

pub async fn user_property_update(
    context: &Context,
    user_id: UserId,
    id: SonarId,
    updates: &[PropertyUpdate],
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    property::user_update(&mut tx, user_id, id, updates).await?;
    tx.commit().await?;
    Ok(())
}

/// List all item identifiers that have a user property with the given key.
#[tracing::instrument(skip(context))]
pub async fn list_with_user_property(
    context: &Context,
    user_id: UserId,
    key: &PropertyKey,
) -> Result<Vec<SonarId>> {
    let mut conn = context.db.acquire().await?;
    property::user_list_with_property(&mut conn, user_id, key).await
}

#[tracing::instrument(skip(create))]
pub async fn image_create(context: &Context, create: ImageCreate) -> Result<ImageId> {
    let mut tx = context.db.begin().await?;
    let result = image::create(&mut tx, &*context.storage, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn image_delete(context: &Context, image_id: ImageId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    image::delete(&mut tx, image_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn image_download(context: &Context, image_id: ImageId) -> Result<ImageDownload> {
    let mut conn = context.db.acquire().await?;
    image::download(&mut conn, &*context.storage, image_id).await
}

#[tracing::instrument(skip(context))]
pub async fn artist_list(context: &Context, params: ListParams) -> Result<Vec<Artist>> {
    let mut conn = context.db.acquire().await?;
    artist::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn artist_get(context: &Context, artist_id: ArtistId) -> Result<Artist> {
    let mut conn = context.db.acquire().await?;
    artist::get(&mut conn, artist_id).await
}

#[tracing::instrument(skip(context))]
pub async fn artist_get_bulk(context: &Context, artist_ids: &[ArtistId]) -> Result<Vec<Artist>> {
    let mut conn = context.db.acquire().await?;
    artist::get_bulk(&mut conn, artist_ids).await
}

pub async fn artist_get_by_name(context: &Context, name: &str) -> Result<Artist> {
    let mut conn = context.db.acquire().await?;
    artist::get_by_name(&mut conn, name).await
}

#[tracing::instrument(skip(context))]
pub async fn artist_create(context: &Context, create: ArtistCreate) -> Result<Artist> {
    let mut tx = context.db.begin().await?;
    let result = artist::create(&mut tx, create).await?;
    tx.commit().await?;
    on_artist_crud(context, result.id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn artist_update(
    context: &Context,
    id: ArtistId,
    update: ArtistUpdate,
) -> Result<Artist> {
    let mut tx = context.db.begin().await?;
    let result = artist::update(&mut tx, id, update).await?;
    tx.commit().await?;
    on_artist_crud(context, id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn artist_delete(context: &Context, id: ArtistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    artist::delete(&mut tx, id).await?;
    tx.commit().await?;
    on_artist_crud(context, id).await;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn artist_find_or_create_by_name(
    context: &Context,
    create: ArtistCreate,
) -> Result<Artist> {
    let mut tx = context.db.begin().await?;
    let result = artist::find_or_create_by_name(&mut tx, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn album_list(context: &Context, params: ListParams) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn album_list_by_artist(
    context: &Context,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::list_by_artist(&mut conn, artist_id, params).await
}

#[tracing::instrument(skip(context))]
pub async fn album_list_by_genre(
    context: &Context,
    genre: &Genre,
    params: ListParams,
) -> Result<Vec<Album>> {
    let indexes = clone_memory_indexes(context);
    let album_ids = indexes.genres().list_albums_by_genre(genre, params);
    album_get_bulk(context, &album_ids).await
}

#[tracing::instrument(skip(context))]
pub async fn album_get(context: &Context, album_id: AlbumId) -> Result<Album> {
    let mut conn = context.db.acquire().await?;
    album::get(&mut conn, album_id).await
}

#[tracing::instrument(skip(context))]
pub async fn album_get_bulk(context: &Context, album_ids: &[AlbumId]) -> Result<Vec<Album>> {
    let mut conn = context.db.acquire().await?;
    album::get_bulk(&mut conn, album_ids).await
}

#[tracing::instrument(skip(context))]
pub async fn album_get_by_name(context: &Context, name: &str) -> Result<Album> {
    let mut conn = context.db.acquire().await?;
    album::get_by_name(&mut conn, name).await
}

#[tracing::instrument(skip(context))]
pub async fn album_create(context: &Context, create: AlbumCreate) -> Result<Album> {
    let mut tx = context.db.begin().await?;
    let result = album::create(&mut tx, create).await?;
    tx.commit().await?;
    on_album_crud(context, result.id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn album_update(context: &Context, id: AlbumId, update: AlbumUpdate) -> Result<Album> {
    let mut tx = context.db.begin().await?;
    let result = album::update(&mut tx, id, update).await?;
    tx.commit().await?;
    on_album_crud(context, id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn album_delete(context: &Context, id: AlbumId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    album::delete(&mut tx, id).await?;
    tx.commit().await?;
    on_album_crud(context, id).await;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn album_find_or_create_by_name(context: &Context, create: AlbumCreate) -> Result<Album> {
    let mut tx = context.db.begin().await?;
    let result = album::find_or_create_by_name(&mut tx, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn track_list(context: &Context, params: ListParams) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn track_list_by_album(
    context: &Context,
    album_id: AlbumId,
    params: ListParams,
) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list_by_album(&mut conn, album_id, params).await
}

#[tracing::instrument(skip(context))]
pub async fn track_list_random(context: &Context, params: TrackListRandom) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list_random(&mut conn, params).await
}

pub async fn track_list_top_from_artist(
    context: &Context,
    artist_id: ArtistId,
) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::list_top_from_artist(&mut conn, artist_id, ListParams::new(0, 50)).await
}

#[tracing::instrument(skip(context))]
pub async fn track_get(context: &Context, track_id: TrackId) -> Result<Track> {
    let mut conn = context.db.acquire().await?;
    track::get(&mut conn, track_id).await
}

#[tracing::instrument(skip(context))]
pub async fn track_get_bulk(context: &Context, track_ids: &[TrackId]) -> Result<Vec<Track>> {
    let mut conn = context.db.acquire().await?;
    track::get_bulk(&mut conn, track_ids).await
}

#[tracing::instrument(skip(context))]
pub async fn track_get_by_name(context: &Context, name: &str) -> Result<Track> {
    let mut conn = context.db.acquire().await?;
    track::get_by_name(&mut conn, name).await
}

#[tracing::instrument(skip(context))]
pub async fn track_create(context: &Context, create: TrackCreate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::create(&mut tx, create).await?;
    tx.commit().await?;
    on_track_crud(context, result.id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn track_update(context: &Context, id: TrackId, update: TrackUpdate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::update(&mut tx, id, update).await?;
    tx.commit().await?;
    on_track_crud(context, id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn track_delete(context: &Context, id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    track::delete(&mut tx, id).await?;
    tx.commit().await?;
    on_track_crud(context, id).await;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn track_find_or_create_by_name(context: &Context, create: TrackCreate) -> Result<Track> {
    let mut tx = context.db.begin().await?;
    let result = track::find_or_create_by_name(&mut tx, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn track_download(
    context: &Context,
    track_id: TrackId,
    range: ByteRange,
) -> Result<AudioDownload> {
    let mut conn = context.db.acquire().await?;
    track::download(&mut conn, &*context.storage, track_id, range).await
}

#[tracing::instrument(skip(context))]
pub async fn track_stat(context: &Context, track_id: TrackId) -> Result<AudioStat> {
    let mut conn = context.db.acquire().await?;
    track::stat(&mut conn, track_id).await
}

#[tracing::instrument(skip(context))]
pub async fn track_get_lyrics(context: &Context, track_id: TrackId) -> Result<Lyrics> {
    let mut conn = context.db.acquire().await?;
    track::get_lyrics(&mut conn, track_id).await
}

#[tracing::instrument(skip(context))]
pub async fn audio_get(context: &Context, audio_id: AudioId) -> Result<Audio> {
    let mut conn = context.db.acquire().await?;
    audio::get(&mut conn, audio_id).await
}

#[tracing::instrument(skip(context))]
pub async fn audio_get_bulk(context: &Context, audio_ids: &[AudioId]) -> Result<Vec<Audio>> {
    let mut conn = context.db.acquire().await?;
    audio::get_bulk(&mut conn, audio_ids).await
}

#[tracing::instrument(skip(context))]
pub async fn audio_list_by_track(context: &Context, track_id: TrackId) -> Result<Vec<Audio>> {
    let mut conn = context.db.acquire().await?;
    audio::list_by_track(&mut conn, track_id).await
}

#[tracing::instrument(skip(context))]
pub async fn audio_create(context: &Context, create: AudioCreate) -> Result<Audio> {
    let mut tx = context.db.begin().await?;
    let result = audio::create(&mut tx, &*context.storage, create).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn audio_delete(context: &Context, audio_id: AudioId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    audio::delete(&mut tx, audio_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn audio_link(context: &Context, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    audio::link(&mut tx, audio_id, track_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn audio_unlink(context: &Context, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    audio::unlink(&mut tx, audio_id, track_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn audio_set_preferred(
    context: &Context,
    audio_id: AudioId,
    track_id: TrackId,
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    audio::set_preferred(&mut tx, audio_id, track_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn playlist_list(context: &Context, params: ListParams) -> Result<Vec<Playlist>> {
    let mut conn = context.db.acquire().await?;
    playlist::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn playlist_get(context: &Context, playlist_id: PlaylistId) -> Result<Playlist> {
    let mut conn = context.db.acquire().await?;
    playlist::get(&mut conn, playlist_id).await
}

#[tracing::instrument(skip(context))]
pub async fn playlist_get_by_name(
    context: &Context,
    user_id: UserId,
    name: &str,
) -> Result<Playlist> {
    let mut conn = context.db.acquire().await?;
    playlist::get_by_name(&mut conn, user_id, name).await
}

#[tracing::instrument(skip(context))]
pub async fn playlist_find_by_name(
    context: &Context,
    user_id: UserId,
    name: &str,
) -> Result<Option<Playlist>> {
    let mut conn = context.db.acquire().await?;
    playlist::find_by_name(&mut conn, user_id, name).await
}

#[tracing::instrument(skip(context))]
pub async fn playlist_create(context: &Context, create: PlaylistCreate) -> Result<Playlist> {
    let mut tx = context.db.begin().await?;
    let result = playlist::create(&mut tx, create).await?;
    tx.commit().await?;
    on_playlist_crud(context, result.id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn playlist_duplicate(
    context: &Context,
    playlist_id: PlaylistId,
    new_name: &str,
) -> Result<Playlist> {
    let mut tx = context.db.begin().await?;
    let result = playlist::duplicate(&mut tx, playlist_id, new_name).await?;
    tx.commit().await?;
    on_playlist_crud(context, result.id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn playlist_update(
    context: &Context,
    id: PlaylistId,
    update: PlaylistUpdate,
) -> Result<Playlist> {
    let mut tx = context.db.begin().await?;
    let result = playlist::update(&mut tx, id, update).await?;
    tx.commit().await?;
    on_playlist_crud(context, id).await;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn playlist_delete(context: &Context, id: PlaylistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    playlist::delete(&mut tx, id).await?;
    tx.commit().await?;
    on_playlist_crud(context, id).await;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn playlist_list_tracks(
    context: &Context,
    id: PlaylistId,
    params: ListParams,
) -> Result<Vec<PlaylistTrack>> {
    let mut conn = context.db.acquire().await?;
    playlist::list_tracks(&mut conn, id, params).await
}

#[tracing::instrument(skip(context))]
pub async fn playlist_clear_tracks(context: &Context, id: PlaylistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    playlist::clear_tracks(&mut tx, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn playlist_insert_tracks(
    context: &Context,
    id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    playlist::insert_tracks(&mut tx, id, tracks).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn playlist_remove_tracks(
    context: &Context,
    id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    playlist::remove_tracks(&mut tx, id, tracks).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn playlist_generate_cover(context: &Context, id: PlaylistId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    playlist::generate_cover(&mut tx, &*context.storage, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn genre_list(context: &Context) -> Result<Vec<GenreStats>> {
    let indexes = clone_memory_indexes(context);
    Ok(indexes.genres().list_genres())
}

#[tracing::instrument(skip(context))]
pub async fn favorite_list(context: &Context, user_id: UserId) -> Result<Vec<Favorite>> {
    let mut conn = context.db.acquire().await?;
    favorite::user_list(&mut conn, user_id).await
}

#[tracing::instrument(skip(context))]
pub async fn favorite_get_bulk(
    context: &Context,
    user_id: UserId,
    ids: &[SonarId],
) -> Result<Vec<Favorite>> {
    let mut conn = context.db.acquire().await?;
    favorite::user_get_bulk(&mut conn, user_id, ids).await
}

#[tracing::instrument(skip(context))]
pub async fn favorite_find(
    context: &Context,
    user_id: UserId,
    id: SonarId,
) -> Result<Option<Favorite>> {
    let mut conn = context.db.acquire().await?;
    favorite::user_find(&mut conn, user_id, id).await
}

#[tracing::instrument(skip(context))]
pub async fn favorite_add(context: &Context, user_id: UserId, id: SonarId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    favorite::user_put(&mut tx, user_id, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn favorite_remove(context: &Context, user_id: UserId, id: SonarId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    favorite::user_remove(&mut tx, user_id, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn search(
    context: &Context,
    user_id: UserId,
    query: SearchQuery,
) -> Result<SearchResults> {
    context.search.search(user_id, &query).await
}

#[tracing::instrument(skip(context))]
pub async fn scrobble_list(context: &Context, params: ListParams) -> Result<Vec<Scrobble>> {
    let mut conn = context.db.acquire().await?;
    scrobble::list(&mut conn, params).await
}

#[tracing::instrument(skip(context))]
pub async fn scrobble_get(context: &Context, scrobble_id: ScrobbleId) -> Result<Scrobble> {
    let mut conn = context.db.acquire().await?;
    scrobble::get(&mut conn, scrobble_id).await
}

#[tracing::instrument(skip(context))]
pub async fn scrobble_create(context: &Context, create: ScrobbleCreate) -> Result<Scrobble> {
    let mut tx = context.db.begin().await?;
    let result = scrobble::create(&mut tx, create).await?;
    tx.commit().await?;
    context.scrobbler_notify.notify_waiters();
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn scrobble_update(
    context: &Context,
    id: ScrobbleId,
    update: ScrobbleUpdate,
) -> Result<Scrobble> {
    let mut tx = context.db.begin().await?;
    let result = scrobble::update(&mut tx, id, update).await?;
    tx.commit().await?;
    Ok(result)
}

#[tracing::instrument(skip(context))]
pub async fn scrobble_delete(context: &Context, id: ScrobbleId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    scrobble::delete(&mut tx, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub(crate) async fn scrobble_list_unsubmitted(
    context: &Context,
    scrobbler: &str,
) -> Result<Vec<Scrobble>> {
    let mut conn = context.db.acquire().await?;
    scrobble::list_unsubmitted(&mut conn, scrobbler).await
}

#[tracing::instrument(skip(context))]
pub(crate) async fn scrobble_list_unsubmitted_for_user(
    context: &Context,
    scrobbler: &str,
    user_id: UserId,
) -> Result<Vec<Scrobble>> {
    let mut conn = context.db.acquire().await?;
    scrobble::list_unsubmitted_for_user(&mut conn, user_id, scrobbler).await
}

#[tracing::instrument(skip(context))]
pub(crate) async fn scrobble_register_submission(
    context: &Context,
    scrobble_id: ScrobbleId,
    scrobbler: &str,
) -> Result<()> {
    let mut tx = context.db.begin().await?;
    scrobble::register_submission(&mut tx, scrobble_id, scrobbler).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn pin_list(context: &Context, user_id: UserId) -> Result<Vec<SonarId>> {
    let mut conn = context.db.acquire().await?;
    pin::list(&mut conn, user_id).await
}

#[tracing::instrument(skip(context))]
pub async fn pin_set(context: &Context, user_id: UserId, id: SonarId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    pin::set(&mut tx, user_id, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn pin_unset(context: &Context, user_id: UserId, id: SonarId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    pin::unset(&mut tx, user_id, id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn external_service_list(context: &Context) -> Result<Vec<String>> {
    todo!()
    // Ok(context
    //     .external
    //     .iter()
    //     .map(|s| s.identifier().to_string())
    //     .collect())
}

#[tracing::instrument(skip(context))]
pub async fn subscription_list(context: &Context, user_id: UserId) -> Result<Vec<Subscription>> {
    let mut conn = context.db.acquire().await?;
    let subscription = subscription::list(&mut conn, user_id, Default::default()).await?;
    Ok(subscription)
}

#[tracing::instrument(skip(context))]
pub async fn subscription_list_all(context: &Context) -> Result<Vec<Subscription>> {
    let mut conn = context.db.acquire().await?;
    let subscription = subscription::list_all(&mut conn, Default::default()).await?;
    Ok(subscription)
}

#[tracing::instrument(skip(context))]
pub async fn subscription_create(
    context: &Context,
    create: SubscriptionCreate,
) -> Result<Subscription> {
    let mut tx = context.db.begin().await?;
    let subscription = subscription::create(&mut tx, create).await?;
    tx.commit().await?;
    subscription_submit(context, subscription.id).await?;
    Ok(subscription)
}

#[tracing::instrument(skip(context))]
pub async fn subscription_delete(context: &Context, subscription_id: SubscriptionId) -> Result<()> {
    let mut tx = context.db.begin().await?;
    subscription::remove(&mut tx, subscription_id).await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn subscription_submit(context: &Context, subscription_id: SubscriptionId) -> Result<()> {
    let sub = {
        let mut tx = context.db.begin().await?;
        subscription::update_last_submitted_to_now(&mut tx, subscription_id).await?;
        let sub = subscription::get(&mut tx, subscription_id).await?;
        tx.commit().await?;
        sub
    };

    tokio::spawn({
        let db = context.db.clone();
        let services = context.external.clone();
        let storage = context.storage.clone();
        let request = ExternalMediaRequest {
            artist: sub.artist,
            album: sub.album,
            track: sub.track,
            playlist: sub.playlist,
            duration: None,
            media_type: sub.media_type.map(|t| match t {
                subscription::SubscriptionMediaType::Artist => ExternalMediaType::Artist,
                subscription::SubscriptionMediaType::Album => ExternalMediaType::Album,
                subscription::SubscriptionMediaType::Track => ExternalMediaType::Track,
                subscription::SubscriptionMediaType::Playlist => ExternalMediaType::Playlist,
            }),
            external_ids: sub.external_id.into_iter().map(From::from).collect(),
        };

        async move {
            if let Err(err) = download::download(&db, &services, &*storage, sub.user, request).await
            {
                tracing::error!("failed to download {}: {}", sub.id, err);
            }
        }
    });

    Ok(())
}

// #[tracing::instrument(skip(context))]
// pub async fn download_list(context: &Context, user_id: UserId) -> Result<Vec<Download>> {
//     Ok(context.downloads.list(user_id))
// }
//
// #[tracing::instrument(skip(context))]
// pub async fn download_request(context: &Context, request: DownloadCreate) -> Result<()> {
//     context
//         .downloads
//         .request(request.user_id, request.external_id)
//         .await;
//     Ok(())
// }
//
// #[tracing::instrument(skip(context))]
// pub async fn download_delete(context: &Context, delete: DownloadDelete) -> Result<()> {
//     context
//         .downloads
//         .delete(delete.user_id, delete.external_id)
//         .await;
//     Ok(())
// }

#[tracing::instrument(skip(context))]
pub async fn garbage_collection_candidates(context: &Context) -> Result<Vec<SonarId>> {
    let mut conn = context.db.acquire().await?;
    gc::list_gc_candidates(&mut conn).await
}

#[tracing::instrument(skip(context, import))]
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

fn merge_metadata_covers(a: Option<Bytes>, b: Option<Bytes>) -> Option<Bytes> {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a.len() > b.len() {
                Some(a)
            } else {
                Some(b)
            }
        }
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn merge_metadata_genres(mut a: Genres, b: Genres) -> Genres {
    Genres::merge(&mut a, &b);
    a
}

fn merge_metadata_properties(mut a: Properties, b: Properties) -> Properties {
    Properties::merge(&mut a, &b);
    a
}

async fn metadata_create_image_opt(context: &Context, image: Option<Bytes>) -> Option<ImageId> {
    match image {
        Some(cover) => match image_create(
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
        },
        None => None,
    }
}

fn metadata_providers_iter<'s>(
    providers: &'s [SonarMetadataProvider],
    kind: MetadataRequestKind,
    params: &'s MetadataFetchParams,
) -> impl Iterator<Item = &'s SonarMetadataProvider> + 's {
    providers.iter().filter(move |p| {
        p.supports(kind)
            && if params.providers.is_empty() {
                true
            } else {
                params
                    .providers
                    .iter()
                    .map(|s| s.as_str())
                    .any(|s| s == p.name())
            }
    })
}

fn metadata_mask_artist_update(update: &mut ArtistUpdate, mask: MetadataFetchMask) {
    if mask & METADATA_FETCH_MASK_NAME == 0 {
        update.name = ValueUpdate::Unchanged;
    }
    if mask & METADATA_FETCH_MASK_GENRES == 0 {
        update.genres = Default::default();
    }
    if mask & METADATA_FETCH_MASK_PROPERTIES == 0 {
        update.properties = Default::default();
    }
    if mask & METADATA_FETCH_MASK_COVER == 0 {
        update.cover_art = ValueUpdate::Unchanged;
    }
}

fn metadata_mask_album_update(update: &mut AlbumUpdate, mask: MetadataFetchMask) {
    if mask & METADATA_FETCH_MASK_NAME == 0 {
        update.name = ValueUpdate::Unchanged;
    }
    if mask & METADATA_FETCH_MASK_GENRES == 0 {
        update.genres = Default::default();
    }
    if mask & METADATA_FETCH_MASK_PROPERTIES == 0 {
        update.properties = Default::default();
    }
    if mask & METADATA_FETCH_MASK_COVER == 0 {
        update.cover_art = ValueUpdate::Unchanged;
    }
}

fn metadata_mask_track_update(update: &mut TrackUpdate, mask: MetadataFetchMask) {
    if mask & METADATA_FETCH_MASK_NAME == 0 {
        update.name = ValueUpdate::Unchanged;
    }
    if mask & METADATA_FETCH_MASK_PROPERTIES == 0 {
        update.properties = Default::default();
    }
    if mask & METADATA_FETCH_MASK_COVER == 0 {
        update.cover_art = ValueUpdate::Unchanged;
    }
}

pub fn metadata_providers(context: &Context) -> Vec<String> {
    context
        .providers
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

#[tracing::instrument(skip(context))]
pub async fn metadata_fetch_artist(
    context: &Context,
    artist_id: ArtistId,
    params: MetadataFetchParams,
) -> Result<()> {
    let metadata = metadata_view_artist(context, artist_id, &params).await?;
    let image_id = metadata_create_image_opt(context, metadata.cover).await;
    let mut update = ArtistUpdate {
        name: ValueUpdate::from_option_unchanged(metadata.name),
        genres: metadata.genres.into_genre_updates(),
        properties: metadata.properties.into_property_updates(),
        cover_art: ValueUpdate::from_option_unchanged(image_id),
    };
    metadata_mask_artist_update(&mut update, params.mask);
    artist_update(context, artist_id, update).await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn metadata_fetch_album(
    context: &Context,
    album_id: AlbumId,
    params: MetadataFetchParams,
) -> Result<()> {
    let metadata = metadata_view_album(context, album_id, &params).await?;
    let image_id = metadata_create_image_opt(context, metadata.cover).await;
    let mut update = AlbumUpdate {
        name: ValueUpdate::from_option_unchanged(metadata.name),
        genres: metadata.genres.into_genre_updates(),
        properties: metadata.properties.into_property_updates(),
        cover_art: ValueUpdate::from_option_unchanged(image_id),
        ..Default::default()
    };
    metadata_mask_album_update(&mut update, params.mask);
    album_update(context, album_id, update).await?;
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn metadata_fetch_album_tracks(
    context: &Context,
    album_id: AlbumId,
    params: MetadataFetchParams,
) -> Result<()> {
    let metadata = metadata_view_album_tracks(context, album_id, &params).await?;
    for (track_id, track_metadata) in metadata.tracks {
        let mut update = TrackUpdate {
            // NOTE: we don't add a cover here because we just assume the album has a cover.
            name: ValueUpdate::from_option_unchanged(track_metadata.name),
            properties: track_metadata.properties.into_property_updates(),
            ..Default::default()
        };
        metadata_mask_track_update(&mut update, params.mask);
        track_update(context, track_id, update).await?;
    }
    Ok(())
}

#[tracing::instrument(skip(context))]
pub async fn metadata_fetch_track(
    context: &Context,
    track_id: TrackId,
    params: MetadataFetchParams,
) -> Result<()> {
    let metadata = metadata_view_track(context, track_id, &params).await?;
    let image_id = metadata_create_image_opt(context, metadata.cover).await;
    let mut update = TrackUpdate {
        name: ValueUpdate::from_option_unchanged(metadata.name),
        properties: metadata.properties.into_property_updates(),
        cover_art: ValueUpdate::from_option_unchanged(image_id),
        ..Default::default()
    };
    metadata_mask_track_update(&mut update, params.mask);
    track_update(context, track_id, update).await?;
    Ok(())
}

fn merge_metadata_artist(a: ArtistMetadata, b: ArtistMetadata) -> ArtistMetadata {
    ArtistMetadata {
        name: a.name.or(b.name),
        genres: merge_metadata_genres(a.genres, b.genres),
        properties: merge_metadata_properties(a.properties, b.properties),
        cover: merge_metadata_covers(a.cover, b.cover),
    }
}

#[tracing::instrument(skip(context))]
pub async fn metadata_view_artist(
    context: &Context,
    artist_id: ArtistId,
    params: &MetadataFetchParams,
) -> Result<ArtistMetadata> {
    let artist = artist_get(context, artist_id).await?;
    let request = ArtistMetadataRequest { artist };
    let providers =
        metadata_providers_iter(&context.providers, MetadataRequestKind::Artist, params);
    let mut metadatas = Vec::new();
    for provider in providers {
        match provider.artist_metadata(context, &request).await {
            Ok(metadata) => metadatas.push(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch artist metadata from provider '{}': {}",
                    provider.name(),
                    err
                );
            }
        }
    }

    Ok(metadatas
        .into_iter()
        .fold(Default::default(), merge_metadata_artist))
}

fn merge_metadata_album(a: AlbumMetadata, b: AlbumMetadata) -> AlbumMetadata {
    AlbumMetadata {
        name: a.name.or(b.name),
        genres: merge_metadata_genres(a.genres, b.genres),
        properties: merge_metadata_properties(a.properties, b.properties),
        cover: merge_metadata_covers(a.cover, b.cover),
    }
}

#[tracing::instrument(skip(context))]
pub async fn metadata_view_album(
    context: &Context,
    album_id: AlbumId,
    params: &MetadataFetchParams,
) -> Result<AlbumMetadata> {
    let album = album_get(context, album_id).await?;
    let artist = artist_get(context, album.artist).await?;
    let request = AlbumMetadataRequest { artist, album };
    let providers = metadata_providers_iter(&context.providers, MetadataRequestKind::Album, params);
    let mut metadatas = Vec::new();
    for fetcher in providers {
        match fetcher.album_metadata(context, &request).await {
            Ok(metadata) => metadatas.push(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch album metadata from provider '{}': {}",
                    fetcher.name(),
                    err
                );
            }
        }
    }
    Ok(metadatas
        .into_iter()
        .fold(Default::default(), merge_metadata_album))
}

fn merge_metadata_album_tracks(
    mut a: AlbumTracksMetadata,
    b: AlbumTracksMetadata,
) -> AlbumTracksMetadata {
    for (track_id, track_metadata) in b.tracks {
        a.tracks
            .entry(track_id)
            .and_modify(|a| *a = merge_metadata_track(a.clone(), track_metadata.clone()))
            .or_insert(track_metadata);
    }
    a
}

#[tracing::instrument(skip(context))]
pub async fn metadata_view_album_tracks(
    context: &Context,
    album_id: AlbumId,
    params: &MetadataFetchParams,
) -> Result<AlbumTracksMetadata> {
    let album = album_get(context, album_id).await?;
    let artist = artist_get(context, album.artist).await?;
    let tracks = track_list_by_album(context, album_id, ListParams::default()).await?;
    let request = AlbumTracksMetadataRequest {
        artist,
        album,
        tracks,
    };
    let providers =
        metadata_providers_iter(&context.providers, MetadataRequestKind::AlbumTracks, params);
    let mut metadatas = Vec::new();
    for fetcher in providers {
        match fetcher.album_tracks_metadata(context, &request).await {
            Ok(metadata) => metadatas.push(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch album tracks metadata from provider '{}': {}",
                    fetcher.name(),
                    err
                );
            }
        }
    }
    Ok(metadatas
        .into_iter()
        .fold(Default::default(), merge_metadata_album_tracks))
}

fn merge_metadata_track(a: TrackMetadata, b: TrackMetadata) -> TrackMetadata {
    TrackMetadata {
        name: a.name.or(b.name),
        properties: merge_metadata_properties(a.properties, b.properties),
        cover: merge_metadata_covers(a.cover, b.cover),
    }
}

#[tracing::instrument(skip(context))]
pub async fn metadata_view_track(
    context: &Context,
    track_id: TrackId,
    params: &MetadataFetchParams,
) -> Result<TrackMetadata> {
    let track = track_get(context, track_id).await?;
    let album = album_get(context, track.album).await?;
    let artist = artist_get(context, album.artist).await?;
    let request = TrackMetadataRequest {
        artist,
        album,
        track,
    };
    let providers = context
        .providers
        .iter()
        .filter(|p| p.supports(MetadataRequestKind::Track));
    let mut metadatas = Vec::new();
    for fetcher in providers {
        match fetcher.track_metadata(context, &request).await {
            Ok(metadata) => metadatas.push(metadata),
            Err(err) => {
                tracing::warn!(
                    "failed to fetch track metadata from provider '{}': {}",
                    fetcher.name(),
                    err
                );
            }
        }
    }
    Ok(metadatas
        .into_iter()
        .fold(Default::default(), merge_metadata_track))
}

async fn on_artist_crud(context: &Context, artist_id: ArtistId) {
    let context = context.clone();
    tokio::spawn(async move {
        context.search.synchronize_artist(artist_id).await;
        memory_indexes_rebuild(&context).await;
    });
}
async fn on_album_crud(context: &Context, album_id: AlbumId) {
    let context = context.clone();
    tokio::spawn(async move {
        context.search.synchronize_album(album_id).await;
        memory_indexes_rebuild(&context).await;
    });
}
async fn on_track_crud(context: &Context, track_id: TrackId) {
    let context = context.clone();
    tokio::spawn(async move {
        context.search.synchronize_track(track_id).await;
        memory_indexes_rebuild(&context).await;
    });
}
async fn on_playlist_crud(context: &Context, playlist_id: PlaylistId) {
    let search = context.search.clone();
    tokio::spawn(async move {
        search.synchronize_playlist(playlist_id).await;
    });
}

fn clone_memory_indexes(context: &Context) -> MemoryIndexes {
    context.memory_indexes.lock().unwrap().clone()
}

async fn update_listen_counts(context: &Context) {
    loop {
        tracing::info!("started listen count update");
        let result = sqlx::query(
            r#"
        WITH listen_count AS (
            SELECT scrobble.track, count(*) 'count' FROM scrobble
            GROUP BY scrobble.track
        )
        UPDATE track
        SET listen_count = COALESCE((SELECT count FROM listen_count WHERE track.id = listen_count.track), 0);
    "#,
        )
        .execute(&context.db)
        .await;

        if let Err(err) = result {
            tracing::error!("failed to update listen counts: {}", err);
        } else {
            tracing::info!("finished listen count update");
        }

        tokio::time::sleep(Duration::from_mins(10)).await;
    }
}
