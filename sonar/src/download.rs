use crate::{
    album, artist, audio,
    blob::BlobStorage,
    bytestream,
    db::Db,
    external::{
        ExternalAlbum, ExternalArtist, ExternalMediaId, ExternalMediaType, ExternalServices,
        ExternalTrack,
    },
    image, playlist, track, Album, AlbumCreate, AlbumId, AlbumUpdate, Artist, ArtistCreate,
    ArtistId, AudioCreate, ExternalMediaRequest, ExternalService, ImageCreate, Playlist,
    PlaylistCreate, Properties, Result, Track, TrackCreate, TrackId, UserId, ValueUpdate,
};

pub async fn download(
    db: &Db,
    services: &ExternalServices,
    storage: &dyn BlobStorage,
    user_id: UserId,
    mut request: ExternalMediaRequest,
) -> Result<()> {
    tracing::debug!("enriching {request:#?}");
    services.enrich(&mut request).await?;

    tracing::debug!("extracting {request:#?}");
    let (service, media_type, external_id) = services.extract(&request).await?;

    match media_type {
        ExternalMediaType::Artist => {
            let external_artist = service.fetch_artist(&external_id).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;

            for album_external_id in external_artist.albums.iter() {
                let album_request = ExternalMediaRequest {
                    external_ids: vec![album_external_id.clone()],
                    ..Default::default()
                };
                let download = Box::pin(download(db, services, storage, user_id, album_request));
                if let Err(err) = download.await {
                    tracing::error!(
                        "failed to download album {} for artist {}: {}",
                        album_external_id,
                        artist.name,
                        err
                    );
                }
            }
        }
        ExternalMediaType::Album => {
            let external_album = service.fetch_album(&external_id).await?;
            let external_artist = service.fetch_artist(&external_album.artist).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;
            let album = find_or_create_album(db, storage, &external_album, artist.id).await?;

            for track_external_id in external_album.tracks {
                let track_request = ExternalMediaRequest {
                    external_ids: vec![track_external_id.clone()],
                    ..Default::default()
                };
                let download = Box::pin(download(db, services, storage, user_id, track_request));
                if let Err(err) = download.await {
                    tracing::error!(
                        "failed to download track {} for album {}/{}: {}",
                        track_external_id,
                        artist.name,
                        album.name,
                        err
                    );
                }
            }
        }
        ExternalMediaType::Track => {
            let external_track = service.fetch_track(&external_id).await?;
            let external_album = service.fetch_album(&external_track.album).await?;
            let external_artist = service.fetch_artist(&external_track.artist).await?;
            let artist = find_or_create_artist(db, &external_artist).await?;
            let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
            let track = find_or_create_track(db, &external_track, album.id).await?;
            download_audio(db, service, storage, &external_id, track.id).await?;
        }
        ExternalMediaType::Playlist => {
            let external_playlist = service.fetch_playlist(&external_id).await?;
            let mut tracks = Vec::new();

            for track in external_playlist.tracks {
                let track_request = ExternalMediaRequest {
                    external_ids: vec![track],
                    media_type: Some(ExternalMediaType::Track),
                    ..Default::default()
                };

                match download_track_request(db, services, storage, track_request.clone()).await {
                    Ok(track_id) => tracks.push(track_id),
                    Err(err) => {
                        tracing::warn!(
                            "failed to download {:#?} for playlist {:#?}: {}",
                            track_request,
                            external_playlist.name,
                            err
                        );
                    }
                }
            }

            let playlist = find_or_create_playlist(
                db,
                user_id,
                &external_playlist.name,
                &external_playlist.properties,
            )
            .await?;

            let mut tx = db.begin().await?;
            playlist::clear_tracks(&mut tx, playlist.id).await?;
            playlist::insert_tracks(&mut tx, playlist.id, &tracks).await?;
            tx.commit().await?;
        }
        ExternalMediaType::Compilation => {
            let external_compilation = service.fetch_compilation(&external_id).await?;
            let mut tracks = Vec::new();

            for track in external_compilation.tracks {
                let track_request = ExternalMediaRequest {
                    artist: Some(track.artist),
                    album: Some(track.album),
                    track: Some(track.track),
                    media_type: Some(ExternalMediaType::Track),
                    ..Default::default()
                };

                match download_track_request(db, services, storage, track_request.clone()).await {
                    Ok(track_id) => tracks.push(track_id),
                    Err(err) => {
                        tracing::warn!(
                            "failed to download {:#?} for compilation {:#?}: {}",
                            track_request,
                            external_compilation.name,
                            err
                        );
                    }
                }
            }

            let playlist = find_or_create_playlist(
                db,
                user_id,
                &external_compilation.name,
                &external_compilation.properties,
            )
            .await?;

            let mut tx = db.begin().await?;
            playlist::clear_tracks(&mut tx, playlist.id).await?;
            playlist::insert_tracks(&mut tx, playlist.id, &tracks).await?;
            tx.commit().await?;
        }
        ExternalMediaType::Group => {
            let external_group = service.fetch_group(&external_id).await?;
            tracing::debug!("expanded group {external_id} to {external_group:#?}");
            for group_item in external_group {
                let item_request = ExternalMediaRequest {
                    external_ids: vec![group_item],
                    ..Default::default()
                };
                if let Err(err) =
                    Box::pin(download(db, services, storage, user_id, item_request)).await
                {
                    tracing::warn!("failed to download group item: {}", err);
                }
            }
        }
    }
    Ok(())
}

// type Sender<T> = tokio::sync::mpsc::Sender<T>;
// type Receiver<T> = tokio::sync::mpsc::Receiver<T>;
// type SharedList = Arc<Mutex<Vec<Download>>>;
//
// const DOWNLOAD_MANAGER_CAPACITY: usize = 8;
// const MAX_DOWNLOAD_RETRIES: u8 = 5;
// const DOWNLOAD_RETRY_DELAY: Duration = Duration::from_secs(30);
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum DownloadStatus {
//     Pending,
//     Downloading,
//     Complete,
//     Failed,
// }
//
// #[derive(Debug, Clone)]
// pub struct Download {
//     pub user_id: UserId,
//     pub external_id: ExternalMediaId,
//     pub status: DownloadStatus,
//     pub description: String,
// }
//
// #[derive(Debug, Clone)]
// pub struct DownloadCreate {
//     pub user_id: UserId,
//     pub external_id: ExternalMediaId,
// }
//
// #[derive(Debug, Clone)]
// pub struct DownloadDelete {
//     pub user_id: UserId,
//     pub external_id: ExternalMediaId,
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// struct DownloadKey {
//     user_id: UserId,
//     external_id: ExternalMediaId,
// }
//
// impl DownloadKey {
//     fn new(user_id: UserId, external_id: ExternalMediaId) -> Self {
//         Self {
//             user_id,
//             external_id,
//         }
//     }
// }
//
// #[derive(Debug)]
// enum Message {
//     Request {
//         user_id: UserId,
//         external_id: ExternalMediaId,
//     },
//     Delete {
//         user_id: UserId,
//         external_id: ExternalMediaId,
//     },
//     Started {
//         user_id: UserId,
//         external_id: ExternalMediaId,
//         description: String,
//     },
//     Complete {
//         user_id: UserId,
//         external_id: ExternalMediaId,
//     },
//     Failed {
//         user_id: UserId,
//         external_id: ExternalMediaId,
//         error: String,
//     },
//     Tick,
// }
//
// #[derive(Debug)]
// struct Inner {
//     sender: Sender<Message>,
//     list: SharedList,
//     handle: tokio::task::AbortHandle,
// }
//
// impl Drop for Inner {
//     fn drop(&mut self) {
//         self.handle.abort();
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct DownloadController(Arc<Inner>);
//
// impl DownloadController {
//     pub fn new(
//         db: Db,
//         storage: Arc<dyn BlobStorage>,
//         mut services: Vec<SonarExternalService>,
//     ) -> Self {
//         let (sender, receiver) = tokio::sync::mpsc::channel(DOWNLOAD_MANAGER_CAPACITY);
//         let list = Arc::new(Mutex::new(Vec::new()));
//         services.sort_by_key(|s| s.priority());
//         let mut process = Process {
//             db,
//             storage,
//             sender: sender.clone(),
//             receiver,
//             list: list.clone(),
//             downloads: Default::default(),
//             external_services: services.into(),
//         };
//         let handle = tokio::spawn(async move { process.run().await }).abort_handle();
//         Self(Arc::new(Inner {
//             sender,
//             list,
//             handle,
//         }))
//     }
//
//     pub fn list(&self, user_id: UserId) -> Vec<Download> {
//         self.0
//             .list
//             .lock()
//             .unwrap()
//             .iter()
//             .filter(|d| d.user_id == user_id)
//             .cloned()
//             .collect()
//     }
//
//     pub async fn request(&self, user_id: UserId, external_id: ExternalMediaId) {
//         self.0
//             .sender
//             .send(Message::Request {
//                 user_id,
//                 external_id,
//             })
//             .await
//             .unwrap();
//     }
//
//     pub async fn delete(&self, user_id: UserId, external_id: ExternalMediaId) {
//         self.0
//             .sender
//             .send(Message::Delete {
//                 user_id,
//                 external_id,
//             })
//             .await
//             .unwrap();
//     }
// }
//
// struct ProcessDownload {
//     user_id: UserId,
//     external_id: ExternalMediaId,
//     status: DownloadStatus,
//     description: String,
//     handle: tokio::task::AbortHandle,
//     num_retries: u8,
//     last_download: Instant,
// }
//
// struct Process {
//     db: Db,
//     storage: Arc<dyn BlobStorage>,
//     sender: Sender<Message>,
//     receiver: Receiver<Message>,
//     list: SharedList,
//     downloads: HashMap<DownloadKey, ProcessDownload>,
//     external_services: Arc<[SonarExternalService]>,
// }
//
// impl Drop for Process {
//     fn drop(&mut self) {
//         for download in self.downloads.values() {
//             download.handle.abort();
//         }
//     }
// }
//
// impl Process {
//     async fn run(&mut self) {
//         tokio::spawn({
//             let sender = self.sender.clone();
//             async move {
//                 let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
//                 while sender.send(Message::Tick).await.is_ok() {
//                     interval.tick().await;
//                 }
//             }
//         });
//
//         while let Some(message) = self.receiver.recv().await {
//             match message {
//                 Message::Request {
//                     user_id,
//                     external_id,
//                 } => {
//                     let download_id = DownloadKey::new(user_id, external_id);
//                     if let Some(download) = self.downloads.get(&download_id) {
//                         if download.status == DownloadStatus::Downloading {
//                             continue;
//                         }
//                     }
//
//                     let handle = tokio::spawn({
//                         let db = self.db.clone();
//                         let storage = self.storage.clone();
//                         let sender = self.sender.clone();
//                         let services = self.external_services.clone();
//                         let external_id = download_id.external_id.clone();
//                         async move {
//                             download_task(db, storage, sender, &services, user_id, external_id)
//                                 .await;
//                         }
//                     });
//
//                     let download = ProcessDownload {
//                         user_id,
//                         external_id: download_id.external_id.clone(),
//                         status: DownloadStatus::Pending,
//                         description: String::new(),
//                         handle: handle.abort_handle(),
//                         num_retries: 0,
//                         last_download: Instant::now(),
//                     };
//
//                     self.downloads.insert(download_id, download);
//                 }
//                 Message::Started {
//                     user_id,
//                     external_id,
//                     description,
//                 } => {
//                     let download_id = DownloadKey::new(user_id, external_id);
//                     if let Some(download) = self.downloads.get_mut(&download_id) {
//                         download.status = DownloadStatus::Downloading;
//                         download.description = description;
//                     }
//                 }
//                 Message::Delete {
//                     user_id,
//                     external_id,
//                 } => {
//                     let download_id = DownloadKey::new(user_id, external_id);
//                     if let Some(download) = self.downloads.remove(&download_id) {
//                         download.handle.abort();
//                     }
//                 }
//                 Message::Complete {
//                     user_id,
//                     external_id,
//                 } => {
//                     let download_id = DownloadKey::new(user_id, external_id);
//                     if let Some(download) = self.downloads.get_mut(&download_id) {
//                         download.status = DownloadStatus::Complete;
//                     }
//                 }
//                 Message::Failed {
//                     user_id,
//                     external_id,
//                     error,
//                 } => {
//                     let download_id = DownloadKey::new(user_id, external_id);
//                     if let Some(download) = self.downloads.get_mut(&download_id) {
//                         download.status = DownloadStatus::Failed;
//                         download.description = error;
//                     }
//                 }
//                 Message::Tick => {
//                     for (_key, download) in self.downloads.iter_mut() {
//                         if download.status == DownloadStatus::Failed
//                             && download.last_download.elapsed() > DOWNLOAD_RETRY_DELAY
//                             && download.num_retries < MAX_DOWNLOAD_RETRIES
//                         {
//                             let handle = tokio::spawn({
//                                 let db = self.db.clone();
//                                 let storage = self.storage.clone();
//                                 let sender = self.sender.clone();
//                                 let services = self.external_services.clone();
//                                 let external_id = download.external_id.clone();
//                                 let user_id = download.user_id;
//                                 async move {
//                                     download_task(
//                                         db,
//                                         storage,
//                                         sender,
//                                         &services,
//                                         user_id,
//                                         external_id,
//                                     )
//                                     .await;
//                                 }
//                             });
//
//                             download.status = DownloadStatus::Downloading;
//                             download.last_download = Instant::now();
//                             download.num_retries += 1;
//                             download.handle = handle.abort_handle();
//                         }
//                     }
//                 }
//             };
//             self.update_list();
//         }
//     }
//
//     fn update_list(&mut self) {
//         let list = self.downloads.values().map(|d| Download {
//             user_id: d.user_id,
//             external_id: d.external_id.clone(),
//             status: d.status,
//             description: d.description.clone(),
//         });
//         *self.list.lock().unwrap() = list.collect();
//     }
// }
//
// async fn download_task(
//     db: Db,
//     storage: Arc<dyn BlobStorage>,
//     sender: Sender<Message>,
//     services: &[SonarExternalService],
//     user_id: UserId,
//     external_id: ExternalMediaId,
// ) {
//     let message = match download(&db, &*storage, services, &sender, user_id, &external_id).await {
//         Ok(_) => {
//             tracing::info!("download complete: {}", external_id);
//             Message::Complete {
//                 user_id,
//                 external_id,
//             }
//         }
//         Err(err) => {
//             tracing::error!("failed to download: {}: {}", external_id, err);
//             Message::Failed {
//                 user_id,
//                 external_id,
//                 error: err.to_string(),
//             }
//         }
//     };
//     sender.send(message).await.unwrap();
// }

// async fn download(
//     db: &Db,
//     storage: &dyn BlobStorage,
//     services: &[SonarExternalService],
//     sender: &Sender<Message>,
//     user_id: UserId,
//     origin_external_id: &ExternalMediaId,
// ) -> Result<()> {
//     todo!()
//
//     // tracing::debug!("enriching {origin_external_id}");
//     // let mut request = ExternalMediaRequest::from(origin_external_id.clone());
//     // enrich(services, &mut request).await?;
//     // tracing::debug!("enriched {origin_external_id} to {request:#?}");
//     //
//     // tracing::debug!("extracting {request:#?}");
//     // let (service, media_type, external_id) = extract(services, &request).await?;
//     //
//     // match media_type {
//     //     ExternalMediaType::Artist => {
//     //         let external_artist = service.fetch_artist(&external_id).await?;
//     //         let artist = find_or_create_artist(db, &external_artist).await?;
//     //
//     //         let description = format!("Artist: {}", external_artist.name);
//     //         sender
//     //             .send(Message::Started {
//     //                 user_id,
//     //                 external_id: external_id.clone(),
//     //                 description,
//     //             })
//     //             .await
//     //             .unwrap();
//     //
//     //         let mut result = Ok(());
//     //         for external_id in external_artist.albums.iter() {
//     //             let (album_service, album_external_id) =
//     //                 expand_and_find_service_for(services, external_id, ExternalMediaType::Album)
//     //                     .await?;
//     //             let external_album = album_service.fetch_album(&album_external_id).await?;
//     //             let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
//     //             for external_id in external_album.tracks.iter() {
//     //                 let (track_service, track_external_id) = expand_and_find_service_for(
//     //                     services,
//     //                     external_id,
//     //                     ExternalMediaType::Track,
//     //                 )
//     //                 .await?;
//     //                 let external_track = track_service.fetch_track(&track_external_id).await?;
//     //                 let track = find_or_create_track(db, &external_track, album.id).await?;
//     //                 let download_result =
//     //                     download_track(db, storage, track_service, &track_external_id, track.id)
//     //                         .await;
//     //                 if result.is_ok() && download_result.is_err() {
//     //                     result = download_result;
//     //                 }
//     //             }
//     //         }
//     //         result
//     //     }
//     //     ExternalMediaType::Album => {
//     //         let external_album = service.fetch_album(&external_id).await?;
//     //         let (artist_service, artist_external_id) = expand_and_find_service_for(
//     //             services,
//     //             &external_album.artist,
//     //             ExternalMediaType::Artist,
//     //         )
//     //         .await?;
//     //         let external_artist = artist_service.fetch_artist(&artist_external_id).await?;
//     //         let artist = find_or_create_artist(db, &external_artist).await?;
//     //         let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
//     //
//     //         let description = format!("Album: {}/{}", external_album.artist, external_album.name);
//     //         sender
//     //             .send(Message::Started {
//     //                 user_id,
//     //                 external_id: external_id.clone(),
//     //                 description,
//     //             })
//     //             .await
//     //             .unwrap();
//     //
//     //         let mut result = Ok(());
//     //         for external_id in external_album.tracks.iter() {
//     //             let (track_service, track_external_id) =
//     //                 expand_and_find_service_for(services, external_id, ExternalMediaType::Track)
//     //                     .await?;
//     //             let external_track = track_service.fetch_track(&track_external_id).await?;
//     //             let track = find_or_create_track(db, &external_track, album.id).await?;
//     //             let download_result =
//     //                 download_track(db, storage, track_service, &track_external_id, track.id).await;
//     //             if result.is_ok() && download_result.is_err() {
//     //                 result = download_result;
//     //             }
//     //         }
//     //         result
//     //     }
//     //     ExternalMediaType::Track => {
//     //         let external_track = service.fetch_track(&external_id).await?;
//     //         let (album_service, album_external_id) = expand_and_find_service_for(
//     //             services,
//     //             &external_track.album,
//     //             ExternalMediaType::Album,
//     //         )
//     //         .await?;
//     //         let external_album = album_service.fetch_album(&album_external_id).await?;
//     //         let (artist_service, artist_external_id) = expand_and_find_service_for(
//     //             services,
//     //             &external_album.artist,
//     //             ExternalMediaType::Artist,
//     //         )
//     //         .await?;
//     //         let external_artist = artist_service.fetch_artist(&artist_external_id).await?;
//     //
//     //         let description = format!(
//     //             "Track: {}/{}/{}",
//     //             external_artist.name, external_album.name, external_track.name
//     //         );
//     //         sender
//     //             .send(Message::Started {
//     //                 user_id,
//     //                 external_id: external_id.clone(),
//     //                 description,
//     //             })
//     //             .await
//     //             .unwrap();
//     //
//     //         let artist = find_or_create_artist(db, &external_artist).await?;
//     //         let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
//     //         let track = find_or_create_track(db, &external_track, album.id).await?;
//     //         download_track(db, storage, service, &external_id, track.id).await?;
//     //         Ok(())
//     //     }
//     //     ExternalMediaType::Playlist => {
//     //         let external_playlist = service.fetch_playlist(&external_id).await?;
//     //
//     //         let description = format!("Playlist: {}", external_playlist.name);
//     //         sender
//     //             .send(Message::Started {
//     //                 user_id,
//     //                 external_id: external_id.clone(),
//     //                 description,
//     //             })
//     //             .await
//     //             .unwrap();
//     //
//     //         let mut playlist_tracks = Vec::new();
//     //         for external_id in external_playlist.tracks.iter() {
//     //             let (track_service, track_external_id) =
//     //                 expand_and_find_service_for(services, external_id, ExternalMediaType::Track)
//     //                     .await?;
//     //             let external_track = track_service.fetch_track(&track_external_id).await?;
//     //
//     //             let (album_service, album_external_id) = expand_and_find_service_for(
//     //                 services,
//     //                 &external_track.album,
//     //                 ExternalMediaType::Album,
//     //             )
//     //             .await?;
//     //             let external_album = album_service.fetch_album(&album_external_id).await?;
//     //
//     //             let (artist_service, artist_external_id) = expand_and_find_service_for(
//     //                 services,
//     //                 &external_album.artist,
//     //                 ExternalMediaType::Artist,
//     //             )
//     //             .await?;
//     //             let external_artist = artist_service.fetch_artist(&artist_external_id).await?;
//     //
//     //             let artist = find_or_create_artist(db, &external_artist).await?;
//     //             let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
//     //             let track = find_or_create_track(db, &external_track, album.id).await?;
//     //             download_track(db, storage, track_service, &track_external_id, track.id).await?;
//     //             playlist_tracks.push(track.id);
//     //         }
//     //         let playlist = find_or_create_playlist(db, &external_playlist, user_id).await?;
//     //
//     //         let mut tx = db.begin().await?;
//     //         playlist::clear_tracks(&mut tx, playlist.id).await?;
//     //         playlist::insert_tracks(&mut tx, playlist.id, &playlist_tracks).await?;
//     //         tx.commit().await?;
//     //
//     //         Ok(())
//     //     }
//     // }
// }
//
async fn find_or_create_artist(db: &Db, external_artist: &ExternalArtist) -> Result<Artist> {
    let create = ArtistCreate {
        name: external_artist.name.clone(),
        cover_art: None,
        genres: external_artist.genres.clone(),
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
        genres: external_album.genres.clone(),
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
            let update = AlbumUpdate {
                cover_art: ValueUpdate::Set(image_id),
                ..Default::default()
            };
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
        lyrics: external_track.lyrics.clone(),
        audio: None,
        properties: external_track.properties.clone(),
    };
    let track = track::find_or_create_by_name_tx(db, create).await?;
    Ok(track)
}

async fn find_or_create_playlist(
    db: &Db,
    user_id: UserId,
    name: &str,
    properties: &Properties,
) -> Result<Playlist> {
    let create = PlaylistCreate {
        name: name.to_string(),
        owner: user_id,
        tracks: Vec::new(),
        properties: properties.clone(),
    };
    let playlist = playlist::find_or_create_by_name_tx(db, create).await?;
    Ok(playlist)
}

async fn download_track_request(
    db: &Db,
    services: &ExternalServices,
    storage: &dyn BlobStorage,
    mut request: ExternalMediaRequest,
) -> Result<TrackId> {
    request.media_type = Some(ExternalMediaType::Track);
    services.enrich(&mut request).await?;
    let (service, track_media_type, track_external_id) = services.extract(&request).await?;
    assert_eq!(track_media_type, ExternalMediaType::Track);
    download_track(db, service, storage, &track_external_id).await
}

async fn download_track(
    db: &Db,
    service: &dyn ExternalService,
    storage: &dyn BlobStorage,
    external_id: &ExternalMediaId,
) -> Result<TrackId> {
    let external_track = service.fetch_track(&external_id).await?;
    let external_album = service.fetch_album(&external_track.album).await?;
    let external_artist = service.fetch_artist(&external_track.artist).await?;
    let artist = find_or_create_artist(db, &external_artist).await?;
    let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
    let track = find_or_create_track(db, &external_track, album.id).await?;
    download_audio(db, service, storage, &external_id, track.id).await?;
    Ok(track.id)
}

async fn download_audio(
    db: &Db,
    service: &dyn ExternalService,
    storage: &dyn BlobStorage,
    external_id: &ExternalMediaId,
    track_id: TrackId,
) -> Result<()> {
    {
        // don't hold the connection while  downloading the track
        let mut conn = db.acquire().await?;
        if !audio::list_by_track(&mut conn, track_id).await?.is_empty() {
            tracing::info!("audio already exists for track: {}", track_id);
            return Ok(());
        }
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
//
// async fn find_service<'s, 'i>(
//     services: &'s [SonarExternalService],
//     external_ids: &'i MultiExternalMediaId,
// ) -> Result<(
//     &'s SonarExternalService,
//     &'i ExternalMediaId,
//     ExternalMediaType,
// )> {
//     todo!()
//     // for service in services {
//     //     for external_id in external_ids {
//     //         match service.probe(external_id).await {
//     //             Ok(id_type) => return Ok((service, external_id, id_type)),
//     //             Err(err) => {
//     //                 if err.kind() != ErrorKind::Invalid {
//     //                     tracing::warn!("failed to validate id: {}: {}", service.identifier(), err);
//     //                 }
//     //             }
//     //         }
//     //     }
//     // }
//     // Err(Error::internal(format!(
//     //     "no service found for ids: {:?}",
//     //     external_ids
//     // )))
// }
//
// async fn find_service_for<'s, 'i>(
//     services: &'s [SonarExternalService],
//     external_ids: &'i MultiExternalMediaId,
//     media_type: ExternalMediaType,
// ) -> Result<(&'s SonarExternalService, &'i ExternalMediaId)> {
//     let (service, external_id, found_media_type) = find_service(services, external_ids).await?;
//     if media_type != found_media_type {
//         return Err(Error::internal(format!(
//             "invalid id type for {}: {}",
//             media_type, external_id
//         )));
//     }
//     Ok((service, external_id))
// }
//
// async fn enrich(
//     services: &[SonarExternalService],
//     request: &mut ExternalMediaRequest,
// ) -> Result<()> {
//     let mut status = ExternalMediaEnrichStatus::Modified;
//     while status == ExternalMediaEnrichStatus::Modified {
//         status = ExternalMediaEnrichStatus::NotModified;
//         for service in services {
//             if service.enrich(request).await? == ExternalMediaEnrichStatus::Modified {
//                 status = ExternalMediaEnrichStatus::Modified;
//             }
//         }
//     }
//     Ok(())
// }
//
// async fn extract<'s>(
//     services: &'s [SonarExternalService],
//     request: &'_ ExternalMediaRequest,
// ) -> Result<(&'s SonarExternalService, ExternalMediaType, ExternalMediaId)> {
//     for service in services {
//         if let Ok((media_type, external_id)) = service.extract(request).await {
//             return Ok((service, media_type, external_id));
//         }
//     }
//     tracing::warn!("failed to extract request: {request:#?}");
//     Err(Error::new(ErrorKind::Invalid, "failed to extract request"))
// }

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
