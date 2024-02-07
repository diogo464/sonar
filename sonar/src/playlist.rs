use crate::{
    db::{Db, DbC},
    property, Error, ListParams, PlaylistId, Properties, PropertyUpdate, Result, Timestamp,
    TrackId, UserId, ValueUpdate,
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
    pub properties: Properties,
    pub created_at: Timestamp,
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
    created_at: i64,
}

impl From<(PlaylistView, Properties)> for Playlist {
    fn from((value, properties): (PlaylistView, Properties)) -> Self {
        Self {
            id: PlaylistId::from_db(value.id),
            name: value.name,
            owner: UserId::from_db(value.owner),
            track_count: value.track_count as u32,
            properties,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Playlist>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        PlaylistView,
        "SELECT * FROM sqlx_playlist ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|v| PlaylistId::from_db(v.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Playlist::from)
        .collect())
}

pub async fn get(db: &mut DbC, playlist_id: PlaylistId) -> Result<Playlist> {
    let playlist_view = sqlx::query_as!(
        PlaylistView,
        "SELECT * FROM sqlx_playlist WHERE id = ?",
        playlist_id
    )
    .fetch_one(&mut *db)
    .await?;
    let properties = property::get(db, playlist_id).await?;
    Ok(Playlist::from((playlist_view, properties)))
}

pub async fn get_bulk(db: &mut DbC, playlist_ids: &[PlaylistId]) -> Result<Vec<Playlist>> {
    let mut playlists = Vec::with_capacity(playlist_ids.len());
    for playlist_id in playlist_ids {
        playlists.push(get(db, *playlist_id).await?);
    }
    Ok(playlists)
}

pub async fn get_by_name(db: &mut DbC, user_id: UserId, name: &str) -> Result<Playlist> {
    match find_by_name(db, user_id, name).await? {
        Some(playlist) => Ok(playlist),
        None => Err(Error::new(crate::ErrorKind::NotFound, "playlist not found")),
    }
}

pub async fn find_by_name(db: &mut DbC, user_id: UserId, name: &str) -> Result<Option<Playlist>> {
    let playlist_id = sqlx::query_scalar!(
        "SELECT id FROM playlist WHERE owner = ? AND name = ?",
        user_id,
        name
    )
    .fetch_optional(&mut *db)
    .await?;
    match playlist_id {
        Some(playlist_id) => get(db, PlaylistId::from_db(playlist_id)).await.map(Some),
        None => Ok(None),
    }
}

pub async fn create(db: &mut DbC, create: PlaylistCreate) -> Result<Playlist> {
    let name = create.name.as_str();
    let playlist_id = sqlx::query!(
        "INSERT INTO playlist(name, owner) VALUES (?, ?) RETURNING id",
        name,
        create.owner,
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    let playlist_id = PlaylistId::from_db(playlist_id);
    insert_tracks(db, playlist_id, &create.tracks).await?;
    property::set(db, playlist_id, &create.properties).await?;

    get(db, playlist_id).await
}

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

pub async fn find_or_create_by_name_tx(db: &Db, create_: PlaylistCreate) -> Result<Playlist> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_.owner, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}

pub async fn update(
    db: &mut DbC,
    playlist_id: PlaylistId,
    update: PlaylistUpdate,
) -> Result<Playlist> {
    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!(
            "UPDATE playlist SET name = ? WHERE id = ?",
            new_name,
            playlist_id
        )
        .execute(&mut *db)
        .await?;
    }
    property::update(db, playlist_id, &update.properties).await?;

    get(db, playlist_id).await
}

pub async fn delete(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    sqlx::query!("DELETE FROM playlist WHERE id = ?", playlist_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, playlist_id).await?;
    Ok(())
}

pub async fn list_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    params: ListParams,
) -> Result<Vec<PlaylistTrack>> {
    let (offset, limit) = params.to_db_offset_limit();
    let tracks = sqlx::query_as!(
        PlaylistTrackView,
        "SELECT playlist, track, created_at FROM playlist_track WHERE playlist = ? ORDER BY rowid ASC LIMIT ? OFFSET ?",
        playlist_id,
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(tracks
        .into_iter()
        .map(|t| t.into_playlist_track())
        .collect())
}

pub async fn list_tracks_in_all_playlists(db: &mut DbC) -> Result<Vec<TrackId>> {
    let tracks = sqlx::query_scalar!("SELECT track FROM playlist_track")
        .fetch_all(&mut *db)
        .await?;
    Ok(tracks.into_iter().map(TrackId::from_db).collect())
}

pub async fn clear_tracks(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    sqlx::query!("DELETE FROM playlist_track WHERE playlist = ?", playlist_id)
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
        sqlx::query!(
            "INSERT INTO playlist_track (playlist, track) VALUES (?, ?)",
            playlist_id,
            track_id
        )
        .execute(&mut *db)
        .await?;
    }
    Ok(())
}

pub async fn remove_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    tracks: &[TrackId],
) -> Result<()> {
    for track_id in tracks {
        sqlx::query!(
            "DELETE FROM playlist_track WHERE playlist = ? AND track = ?",
            playlist_id,
            track_id
        )
        .execute(&mut *db)
        .await?;
    }
    Ok(())
}
