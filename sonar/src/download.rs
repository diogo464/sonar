use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    album, artist, audio,
    blob::BlobStorage,
    bytestream,
    db::Db,
    external::{
        ExternalAlbum, ExternalArtist, ExternalMediaId, ExternalMediaType, ExternalPlaylist,
        ExternalTrack, SonarExternalService,
    },
    image, playlist, track, Album, AlbumCreate, AlbumId, AlbumUpdate, Artist, ArtistCreate,
    ArtistId, AudioCreate, Error, ErrorKind, ImageCreate, Playlist, PlaylistCreate, Result, Track,
    TrackCreate, TrackId, UserId, ValueUpdate,
};

type Sender<T> = tokio::sync::mpsc::Sender<T>;
type Receiver<T> = tokio::sync::mpsc::Receiver<T>;
type SharedList = Arc<Mutex<Vec<Download>>>;

const DOWNLOAD_MANAGER_CAPACITY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DownloadStatus {
    Downloading,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Download {
    pub user_id: UserId,
    pub external_id: ExternalMediaId,
    pub status: DownloadStatus,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DownloadCreate {
    pub user_id: UserId,
    pub external_id: ExternalMediaId,
}

#[derive(Debug, Clone)]
pub struct DownloadDelete {
    pub user_id: UserId,
    pub external_id: ExternalMediaId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DownloadKey {
    user_id: UserId,
    external_id: ExternalMediaId,
}

impl DownloadKey {
    fn new(user_id: UserId, external_id: ExternalMediaId) -> Self {
        Self {
            user_id,
            external_id,
        }
    }
}

#[derive(Debug)]
enum Message {
    Request {
        user_id: UserId,
        external_id: ExternalMediaId,
    },
    Delete {
        user_id: UserId,
        external_id: ExternalMediaId,
    },
    Complete {
        user_id: UserId,
        external_id: ExternalMediaId,
    },
    Failed {
        user_id: UserId,
        external_id: ExternalMediaId,
        error: String,
    },
}

#[derive(Debug)]
struct Inner {
    sender: Sender<Message>,
    list: SharedList,
    handle: tokio::task::AbortHandle,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[derive(Debug, Clone)]
pub struct DownloadController(Arc<Inner>);

impl DownloadController {
    pub fn new(
        db: Db,
        storage: Arc<dyn BlobStorage>,
        mut services: Vec<SonarExternalService>,
    ) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(DOWNLOAD_MANAGER_CAPACITY);
        let list = Arc::new(Mutex::new(Vec::new()));
        services.sort_by_key(|s| s.priority());
        let mut process = Process {
            db,
            storage,
            sender: sender.clone(),
            receiver,
            list: list.clone(),
            downloads: Default::default(),
            external_services: services.into(),
        };
        let handle = tokio::spawn(async move { process.run().await }).abort_handle();
        Self(Arc::new(Inner {
            sender,
            list,
            handle,
        }))
    }

    pub fn list(&self, user_id: UserId) -> Vec<Download> {
        self.0
            .list
            .lock()
            .unwrap()
            .iter()
            .filter(|d| d.user_id == user_id)
            .cloned()
            .collect()
    }

    pub async fn request(&self, user_id: UserId, external_id: ExternalMediaId) {
        self.0
            .sender
            .send(Message::Request {
                user_id,
                external_id,
            })
            .await
            .unwrap();
    }

    pub async fn delete(&self, user_id: UserId, external_id: ExternalMediaId) {
        self.0
            .sender
            .send(Message::Delete {
                user_id,
                external_id,
            })
            .await
            .unwrap();
    }
}

struct ProcessDownload {
    user_id: UserId,
    external_id: ExternalMediaId,
    status: DownloadStatus,
    description: String,
    handle: tokio::task::AbortHandle,
}

struct Process {
    db: Db,
    storage: Arc<dyn BlobStorage>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    list: SharedList,
    downloads: HashMap<DownloadKey, ProcessDownload>,
    external_services: Arc<[SonarExternalService]>,
}

impl Drop for Process {
    fn drop(&mut self) {
        for download in self.downloads.values() {
            download.handle.abort();
        }
    }
}

impl Process {
    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                Message::Request {
                    user_id,
                    external_id,
                } => {
                    let download_id = DownloadKey::new(user_id, external_id);
                    if let Some(download) = self.downloads.get(&download_id) {
                        if download.status == DownloadStatus::Downloading {
                            return;
                        }
                    }

                    let handle = tokio::spawn({
                        let db = self.db.clone();
                        let storage = self.storage.clone();
                        let sender = self.sender.clone();
                        let services = self.external_services.clone();
                        let external_id = download_id.external_id.clone();
                        async move {
                            download_task(db, storage, sender, &services, user_id, external_id)
                                .await;
                        }
                    });

                    let download = ProcessDownload {
                        user_id,
                        external_id: download_id.external_id.clone(),
                        status: DownloadStatus::Downloading,
                        description: String::new(),
                        handle: handle.abort_handle(),
                    };

                    self.downloads.insert(download_id, download);
                }
                Message::Delete {
                    user_id,
                    external_id,
                } => {
                    let download_id = DownloadKey::new(user_id, external_id);
                    if let Some(download) = self.downloads.remove(&download_id) {
                        download.handle.abort();
                    }
                }
                Message::Complete {
                    user_id,
                    external_id,
                } => {
                    let download_id = DownloadKey::new(user_id, external_id);
                    if let Some(download) = self.downloads.get_mut(&download_id) {
                        download.status = DownloadStatus::Complete;
                    }
                }
                Message::Failed {
                    user_id,
                    external_id,
                    error,
                } => {
                    let download_id = DownloadKey::new(user_id, external_id);
                    if let Some(download) = self.downloads.get_mut(&download_id) {
                        download.status = DownloadStatus::Failed;
                        download.description = error;
                    }
                }
            };
            self.update_list();
        }
    }

    fn update_list(&mut self) {
        let list = self.downloads.values().map(|d| Download {
            user_id: d.user_id,
            external_id: d.external_id.clone(),
            status: d.status,
            description: d.description.clone(),
        });
        *self.list.lock().unwrap() = list.collect();
    }
}

async fn download_task(
    db: Db,
    storage: Arc<dyn BlobStorage>,
    sender: Sender<Message>,
    services: &[SonarExternalService],
    user_id: UserId,
    external_id: ExternalMediaId,
) {
    let message = match download(&db, &*storage, services, user_id, &external_id).await {
        Ok(_) => Message::Complete {
            user_id,
            external_id,
        },
        Err(err) => Message::Failed {
            user_id,
            external_id,
            error: err.to_string(),
        },
    };
    sender.send(message).await.unwrap();
}

async fn download(
    db: &Db,
    storage: &dyn BlobStorage,
    services: &[SonarExternalService],
    user_id: UserId,
    external_id: &ExternalMediaId,
) -> Result<()> {
    let (service, media_type) = find_service(services, external_id).await?;
    match media_type {
        ExternalMediaType::Artist => {
            let external_artist = service.fetch_artist(external_id).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;
            for external_id in external_artist.albums.iter() {
                let album_service =
                    find_service_for(services, external_id, ExternalMediaType::Album).await?;
                let external_album = album_service.fetch_album(external_id).await?;
                let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
                for external_id in external_album.tracks.iter() {
                    let track_service =
                        find_service_for(services, external_id, ExternalMediaType::Track).await?;
                    let external_track = track_service.fetch_track(external_id).await?;
                    let track = find_or_create_track(db, &external_track, album.id).await?;
                    download_track(db, storage, track_service, external_id, track.id).await?;
                }
            }
            Ok(())
        }
        ExternalMediaType::Album => {
            let external_album = service.fetch_album(external_id).await?;
            let artist_service =
                find_service_for(services, &external_album.artist, ExternalMediaType::Artist)
                    .await?;
            let external_artist = artist_service.fetch_artist(&external_album.artist).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;
            let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
            for external_id in external_album.tracks.iter() {
                let track_service =
                    find_service_for(services, external_id, ExternalMediaType::Track).await?;
                let external_track = track_service.fetch_track(external_id).await?;
                let track = find_or_create_track(db, &external_track, album.id).await?;
                download_track(db, storage, track_service, external_id, track.id).await?;
            }
            Ok(())
        }
        ExternalMediaType::Track => {
            let external_track = service.fetch_track(external_id).await?;
            let album_service =
                find_service_for(services, &external_track.album, ExternalMediaType::Album).await?;
            let external_album = album_service.fetch_album(&external_track.album).await?;
            let artist_service =
                find_service_for(services, &external_album.artist, ExternalMediaType::Artist)
                    .await?;
            let external_artist = artist_service.fetch_artist(&external_album.artist).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;
            let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
            let track = find_or_create_track(db, &external_track, album.id).await?;
            download_track(db, storage, service, external_id, track.id).await?;
            Ok(())
        }
        ExternalMediaType::Playlist => {
            let external_playlist = service.fetch_playlist(external_id).await?;
            let mut playlist_tracks = Vec::new();
            for external_id in external_playlist.tracks.iter() {
                let track_service =
                    find_service_for(services, external_id, ExternalMediaType::Track).await?;
                let external_track = track_service.fetch_track(external_id).await?;
                let album_service =
                    find_service_for(services, &external_track.album, ExternalMediaType::Album)
                        .await?;
                let external_album = album_service.fetch_album(&external_track.album).await?;
                let artist_service =
                    find_service_for(services, &external_album.artist, ExternalMediaType::Artist)
                        .await?;
                let external_artist = artist_service.fetch_artist(&external_album.artist).await?;
                let artist = find_or_create_artist(db, &external_artist).await?;
                let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
                let track = find_or_create_track(db, &external_track, album.id).await?;
                download_track(db, storage, track_service, external_id, track.id).await?;
                playlist_tracks.push(track.id);
            }
            let playlist = find_or_create_playlist(db, &external_playlist, user_id).await?;

            let mut tx = db.begin().await?;
            playlist::clear_tracks(&mut tx, playlist.id).await?;
            playlist::insert_tracks(&mut tx, playlist.id, &playlist_tracks).await?;
            tx.commit().await?;

            Ok(())
        }
    }
}

async fn find_or_create_artist(db: &Db, external_artist: &ExternalArtist) -> Result<Artist> {
    let create = ArtistCreate {
        name: external_artist.name.clone(),
        cover_art: None,
        properties: external_artist.properties.clone(),
    };
    let artist = artist::find_or_create_by_name_tx(db, create).await?;
    Ok(artist)
}

async fn find_or_create_album(
    db: &Db,
    storage: &dyn BlobStorage,
    external_album: &ExternalAlbum,
    artist_id: ArtistId,
) -> Result<Album> {
    let create = AlbumCreate {
        name: external_album.name.clone(),
        artist: artist_id,
        cover_art: None,
        properties: external_album.properties.clone(),
    };
    let album = album::find_or_create_by_name_tx(db, create).await?;
    if let Some(ref cover) = external_album.cover
        && album.cover_art.is_none()
    {
        let mut tx = db.begin().await?;
        if let Ok(image_id) = image::create(
            &mut tx,
            storage,
            ImageCreate {
                data: bytestream::from_bytes(cover.data.clone()),
            },
        )
        .await
        {
            let mut update = AlbumUpdate::default();
            update.cover_art = ValueUpdate::Set(image_id);
            album::update(&mut tx, album.id, update).await?;
            tx.commit().await?;
        }
    }
    Ok(album)
}

async fn find_or_create_track(
    db: &Db,
    external_track: &ExternalTrack,
    album_id: AlbumId,
) -> Result<Track> {
    let create = TrackCreate {
        name: external_track.name.clone(),
        album: album_id,
        cover_art: None,
        lyrics: None,
        audio: None,
        properties: external_track.properties.clone(),
    };
    let track = track::find_or_create_by_name_tx(db, create).await?;
    Ok(track)
}

async fn find_or_create_playlist(
    db: &Db,
    external_playlist: &ExternalPlaylist,
    user_id: UserId,
) -> Result<Playlist> {
    let create = PlaylistCreate {
        name: external_playlist.name.clone(),
        owner: user_id,
        tracks: Vec::new(),
        properties: external_playlist.properties.clone(),
    };
    let playlist = playlist::find_or_create_by_name_tx(db, create).await?;
    Ok(playlist)
}

async fn download_track(
    db: &Db,
    storage: &dyn BlobStorage,
    service: &SonarExternalService,
    external_id: &ExternalMediaId,
    track_id: TrackId,
) -> Result<()> {
    let mut conn = db.acquire().await?;
    if !audio::list_by_track(&mut conn, track_id).await?.is_empty() {
        tracing::info!("audio already exists for track: {}", track_id);
        return Ok(());
    }

    let stream = service.download_track(external_id).await?;
    let create = AudioCreate {
        stream,
        filename: Some(external_id.as_str().to_owned()),
    };
    let mut tx = db.begin().await?;
    let audio = audio::create(&mut tx, storage, create).await?;
    audio::set_preferred(&mut tx, audio.id, track_id).await?;
    tx.commit().await?;
    Ok(())
}

async fn find_service<'s>(
    services: &'s [SonarExternalService],
    external_id: &'_ ExternalMediaId,
) -> Result<(&'s SonarExternalService, ExternalMediaType)> {
    for service in services {
        match service.validate_id(external_id).await {
            Ok(id_type) => return Ok((service, id_type)),
            Err(err) => {
                if err.kind() != ErrorKind::Invalid {
                    tracing::warn!("failed to validate id: {}: {}", service.identifier(), err);
                }
            }
        }
    }
    Err(Error::internal(format!(
        "no service found for id: {}",
        external_id
    )))
}

async fn find_service_for<'s>(
    services: &'s [SonarExternalService],
    external_id: &'_ ExternalMediaId,
    media_type: ExternalMediaType,
) -> Result<&'s SonarExternalService> {
    let (service, found_media_type) = find_service(services, external_id).await?;
    if media_type != found_media_type {
        return Err(Error::internal(format!(
            "invalid id type for {}: {}",
            media_type, external_id
        )));
    }
    Ok(service)
}

// download(db, external_id, user_id)
//  service, external_id_type = find_service(services, external_id)
//  external_id is artist =>
//      external_artist = fetch_artist(service, external_id)
//
//      artist = find_or_create_artist(db, external_artist)
//
//      for external_id in external_artist.albums
//          album_service = find_service_for(services, external_id, Album)
//          external_album = fetch_album(album_service, external_id)
//
//          album = find_or_create_album(db, external_album, artist.id)
//          for external_id in external_album.tracks
//              track_service = find_service_for(services, external_id, Track)
//              external_track = fetch_track(track_service, external_id)
//              track = find_or_create_track(db, external_track, album.id)
//              download_track(track_service, external_id, track.id)
//
//
//  external_id is album =>
//      external_album = fetch_album(service, external_id)
//      artist_service = find_service_for(services, external_album.artist, Artist)
//      external_artist = fetch_artist(artist_service, external_album.artist)
//      artist = find_or_create_artist(db, external_artist)
//
//      album = find_or_create_album(db, external_album, artist.id)
//      for external_id in external_album.tracks
//          track_service = find_service_for(services, external_id, Track)
//          external_track = fetch_track(track_service, external_id)
//          track = find_or_create_track(db, external_track, album.id)
//          download_track(track_service, external_id, track.id)
//
//  external_id is track =>
//      external_track = fetch_track(service, external_id)
//      album_service = find_service_for(services, external_track.album, Album)
//      external_album = fetch_album(album_service, external_track.album)
//      artist_service = find_service_for(services, external_album.artist, Artist)
//      external_artist = fetch_artist(artist_service, external_album.artist)
//
//      artist = find_or_create_artist(db, external_artist)
//      album = find_or_create_album(db, external_album, artist.id)
//      track = find_or_create_track(db, external_track, album.id)
//
//      download_track(service, external_id, track.id)
//
//  external_id is playlist =>
//      external_playlist = fetch_playlist(service, external_id)
//      playlist_tracks = Vec::new()
//      for external_id in external_playlist.tracks
//          track_service = find_service_for(services, external_id)
//          external_track = fetch_track(track_service, external_id)
//          album_service = find_service_for(services, external_track.album, Album)
//          external_album = fetch_album(album_service, external_track.album)
//          artist_service = find_service_for(services, external_album.artist, Artist)
//          external_artist = fetch_artist(artist_service, external_album.artist)
//
//          artist = find_or_create_artist(db, external_artist)
//          album = find_or_create_album(db, external_album, artist.id)
//          track = find_or_create_track(db, external_track, album.id)
//
//          download_track(service, external_id, track.id)
//          playlist_tracks.push(track.id)
//
//      playlist = find_or_create_playlist(db, external_playlist, user_id)
//      playlist_clear(db, playlist.id)
//      playlist_set_tracks(db, playlist.id, playlist_tracks)
