use std::io::Cursor;
use std::time::Duration;

use image::DynamicImage;
use image::{io::Reader as ImageReader, GenericImage};

use crate::blob::BlobStorage;
use crate::{album, track, ImageCreate};
use crate::{
    db::{self, Db, DbC},
    property, Error, ImageId, ListParams, PlaylistId, Properties, PropertyUpdate, Result,
    Timestamp, TrackId, UserId, ValueUpdate,
};

#[derive(Debug, Clone)]
pub struct PlaylistTrack {
    pub playlist: PlaylistId,
    pub track: TrackId,
    pub inserted_at: Timestamp,
}

// TODO: add duration
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub owner: UserId,
    pub track_count: u32,
    pub duration: Duration,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct PlaylistCreate {
    pub name: String,
    pub owner: UserId,
    pub tracks: Vec<TrackId>,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct PlaylistUpdate {
    pub name: ValueUpdate<String>,
    pub cover_art: ValueUpdate<ImageId>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, sqlx::FromRow)]
struct PlaylistTrackView {
    playlist: i64,
    track: i64,
    created_at: i64,
}

impl PlaylistTrackView {
    fn into_playlist_track(self) -> PlaylistTrack {
        PlaylistTrack {
            playlist: PlaylistId::from_db(self.playlist),
            track: TrackId::from_db(self.track),
            inserted_at: Timestamp::from_seconds(self.created_at as u64),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PlaylistView {
    id: i64,
    name: String,
    duration_ms: i64,
    owner: i64,
    track_count: i64,
    cover_art: Option<i64>,
    created_at: i64,
}

impl From<(PlaylistView, Properties)> for Playlist {
    fn from((value, properties): (PlaylistView, Properties)) -> Self {
        Self {
            id: PlaylistId::from_db(value.id),
            name: value.name,
            owner: UserId::from_db(value.owner),
            track_count: value.track_count as u32,
            duration: Duration::from_millis(value.duration_ms as u64),
            cover_art: value.cover_art.map(ImageId::from_db),
            properties,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Playlist>> {
    let views = db::list::<PlaylistView>(db, "sqlx_playlist", params).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|v| PlaylistId::from_db(v.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Playlist::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, playlist_id: PlaylistId) -> Result<Playlist> {
    let view = db::get_by_id(db, "sqlx_playlist", playlist_id).await?;
    let properties = property::get(db, playlist_id).await?;
    Ok(Playlist::from((view, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, playlist_ids: &[PlaylistId]) -> Result<Vec<Playlist>> {
    let mut playlists = Vec::with_capacity(playlist_ids.len());
    for playlist_id in playlist_ids {
        playlists.push(get(db, *playlist_id).await?);
    }
    Ok(playlists)
}

#[tracing::instrument(skip(db))]
pub async fn get_by_name(db: &mut DbC, user_id: UserId, name: &str) -> Result<Playlist> {
    match find_by_name(db, user_id, name).await? {
        Some(playlist) => Ok(playlist),
        None => Err(Error::new(crate::ErrorKind::NotFound, "playlist not found")),
    }
}

#[tracing::instrument(skip(db))]
pub async fn find_by_name(db: &mut DbC, user_id: UserId, name: &str) -> Result<Option<Playlist>> {
    let playlist_id = sqlx::query_scalar("SELECT id FROM playlist WHERE owner = ? AND name = ?")
        .bind(user_id)
        .bind(name)
        .fetch_optional(&mut *db)
        .await?;
    match playlist_id {
        Some(playlist_id) => get(db, PlaylistId::from_db(playlist_id)).await.map(Some),
        None => Ok(None),
    }
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: PlaylistCreate) -> Result<Playlist> {
    let playlist_id =
        sqlx::query_scalar("INSERT INTO playlist(name, owner) VALUES (?, ?) RETURNING id")
            .bind(create.name.as_str())
            .bind(create.owner)
            .fetch_one(&mut *db)
            .await?;

    let playlist_id = PlaylistId::from_db(playlist_id);
    insert_tracks(db, playlist_id, &create.tracks).await?;
    property::set(db, playlist_id, &create.properties).await?;

    get(db, playlist_id).await
}

#[tracing::instrument(skip(db))]
pub async fn duplicate(db: &mut DbC, playlist_id: PlaylistId, new_name: &str) -> Result<Playlist> {
    let playlist = get(db, playlist_id).await?;
    let playlist_tracks = list_tracks(db, playlist_id, ListParams::default()).await?;
    let new_playlist = create(
        db,
        PlaylistCreate {
            name: new_name.to_owned(),
            owner: playlist.owner,
            tracks: playlist_tracks.iter().map(|t| t.track).collect(),
            cover_art: playlist.cover_art,
            properties: playlist.properties.clone(),
        },
    )
    .await?;
    Ok(new_playlist)
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name(
    db: &mut DbC,
    user_id: UserId,
    create_: PlaylistCreate,
) -> Result<Playlist> {
    match find_by_name(db, user_id, &create_.name).await? {
        Some(playlist) => Ok(playlist),
        None => create(db, create_).await,
    }
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name_tx(db: &Db, create_: PlaylistCreate) -> Result<Playlist> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_.owner, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}

#[tracing::instrument(skip(db))]
pub async fn update(
    db: &mut DbC,
    playlist_id: PlaylistId,
    update: PlaylistUpdate,
) -> Result<Playlist> {
    db::value_update_string_non_null(db, "playlist", "name", playlist_id, update.name).await?;
    db::value_update_id_nullable(db, "playlist", "cover_art", playlist_id, update.cover_art)
        .await?;
    property::update(db, playlist_id, &update.properties).await?;
    get(db, playlist_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    sqlx::query("DELETE FROM playlist WHERE id = ?")
        .bind(playlist_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, playlist_id).await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn list_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    params: ListParams,
) -> Result<Vec<PlaylistTrack>> {
    let (offset, limit) = params.to_db_offset_limit();
    let tracks = sqlx::query_as::<_,PlaylistTrackView>(
        "SELECT playlist, track, created_at FROM playlist_track WHERE playlist = ? ORDER BY rowid ASC LIMIT ? OFFSET ?")
        .bind(playlist_id)
        .bind(limit)
        .bind(offset)
    .fetch_all(&mut *db)
    .await?;
    Ok(tracks
        .into_iter()
        .map(|t| t.into_playlist_track())
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_tracks_in_all_playlists(db: &mut DbC) -> Result<Vec<TrackId>> {
    let tracks = sqlx::query_scalar("SELECT track FROM playlist_track")
        .fetch_all(&mut *db)
        .await?;
    Ok(tracks.into_iter().map(TrackId::from_db).collect())
}

#[tracing::instrument(skip(db))]
pub async fn clear_cover(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    update(
        db,
        playlist_id,
        PlaylistUpdate {
            cover_art: ValueUpdate::unset(),
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn clear_tracks(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    sqlx::query("DELETE FROM playlist_track WHERE playlist = ?")
        .bind(playlist_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn insert_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    for track_id in tracks {
        sqlx::query("INSERT INTO playlist_track (playlist, track) VALUES (?, ?)")
            .bind(playlist_id)
            .bind(track_id)
            .execute(&mut *db)
            .await?;
    }
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn remove_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    for track_id in tracks {
        sqlx::query("DELETE FROM playlist_track WHERE playlist = ? AND track = ?")
            .bind(playlist_id)
            .bind(track_id)
            .execute(&mut *db)
            .await?;
    }
    Ok(())
}

#[tracing::instrument(skip(db, storage))]
pub async fn generate_cover(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    playlist_id: PlaylistId,
) -> Result<()> {
    tracing::info!("creating cover art");
    let playlist_tracks = list_tracks(db, playlist_id, Default::default()).await?;

    if playlist_tracks.is_empty() {
        tracing::info!("playlist has no tracks, skipping cover art creation");
        return Ok(());
    }

    let mut images = Vec::with_capacity(4);
    for playlist_track in playlist_tracks.into_iter().cycle().take(4) {
        let track = track::get(db, playlist_track.track).await?;
        let album = album::get(db, track.album).await?;
        let image = match album.cover_art {
            Some(image_id) => {
                let download = crate::image::download(db, storage, image_id).await?;
                let image_data = crate::bytestream::to_bytes(download.stream).await?;
                let image = ImageReader::new(Cursor::new(&image_data))
                    .with_guessed_format()?
                    .decode()
                    .map_err(Error::wrap)?;
                image.resize_exact(300, 300, image::imageops::FilterType::Nearest)
            }
            None => {
                tracing::debug!("no image found for track {}", track.id);
                DynamicImage::new_rgb8(300, 300)
            }
        };
        images.push(image);
    }

    let mut cover_image = DynamicImage::new_rgb8(600, 600);
    cover_image
        .copy_from(&images[0], 0, 0)
        .map_err(Error::wrap)?;
    cover_image
        .copy_from(&images[1], 300, 0)
        .map_err(Error::wrap)?;
    cover_image
        .copy_from(&images[2], 0, 300)
        .map_err(Error::wrap)?;
    cover_image
        .copy_from(&images[3], 300, 300)
        .map_err(Error::wrap)?;

    let output_path = tempfile::NamedTempFile::new()?.into_temp_path();
    tokio::task::spawn_blocking({
        let output_path = output_path.to_path_buf();
        move || {
            cover_image
                .save_with_format(&output_path, image::ImageFormat::Jpeg)
                .map_err(Error::wrap)?;
            Result::<()>::Ok(())
        }
    })
    .await
    .map_err(Error::wrap)??;

    let image_stream = crate::bytestream::from_file(&output_path).await?;
    let image_id = crate::image::create(db, storage, ImageCreate { data: image_stream }).await?;

    update(
        db,
        playlist_id,
        PlaylistUpdate {
            cover_art: ValueUpdate::set(image_id),
            ..Default::default()
        },
    )
    .await?;

    tracing::info!("cover art updated");

    Ok(())
}
