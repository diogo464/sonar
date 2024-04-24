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
            playlist::clear_cover(&mut tx, playlist.id).await?;
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
        cover_art: None,
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
    let external_track = service.fetch_track(external_id).await?;
    let external_album = service.fetch_album(&external_track.album).await?;
    let external_artist = service.fetch_artist(&external_track.artist).await?;
    let artist = find_or_create_artist(db, &external_artist).await?;
    let album = find_or_create_album(db, storage, &external_album, artist.id).await?;
    let track = find_or_create_track(db, &external_track, album.id).await?;
    download_audio(db, service, storage, external_id, track.id).await?;
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
