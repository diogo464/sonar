use crate::{
    db::DbC, Error, ListParams, PlaylistId, Properties, Result, Timestamp, TrackId, UserId,
    ValueUpdate, PropertyUpdate,
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
    properties: Option<Vec<u8>>,
    track_count: i64,
    created_at: i64,
}

impl PlaylistView {
    fn into_playlist(self) -> Playlist {
        Playlist {
            id: PlaylistId::from_db(self.id),
            name: self.name,
            owner: UserId::from_db(self.owner),
            track_count: self.track_count as u32,
            properties: Properties::deserialize_unchecked(&self.properties.unwrap_or_default()),
            created_at: Timestamp::from_seconds(self.created_at as u64),
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Playlist>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        PlaylistView,
        "SELECT * FROM playlist_view ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(views.into_iter().map(PlaylistView::into_playlist).collect())
}

pub async fn get(db: &mut DbC, playlist_id: PlaylistId) -> Result<Playlist> {
    let playlist_view = sqlx::query_as!(
        PlaylistView,
        "SELECT * FROM playlist_view WHERE id = ?",
        playlist_id
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(playlist_view.into_playlist())
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
    let owner = create.owner.to_db();
    let properties = create.properties.serialize();
    let playlist_id = sqlx::query!(
        "INSERT INTO playlist(name, owner, properties) VALUES (?, ?, ?) RETURNING id",
        name,
        owner,
        properties
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    let playlist_id = PlaylistId::from_db(playlist_id);
    insert_tracks(db, playlist_id, &create.tracks).await?;

    get(db, playlist_id).await
}

pub async fn update(
    db: &mut DbC,
    playlist_id: PlaylistId,
    update: PlaylistUpdate,
) -> Result<Playlist> {
    let db_id = playlist_id.to_db();
    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!("UPDATE playlist SET name = ? WHERE id = ?", new_name, db_id)
            .execute(&mut *db)
            .await?;
    }

    if update.properties.len() > 0 {
        let properties = sqlx::query_scalar!("SELECT properties FROM playlist WHERE id = ?", db_id)
            .fetch_one(&mut *db)
            .await?
            .unwrap_or_default();
        let mut properties = Properties::deserialize_unchecked(&properties);
        properties.apply_updates(&update.properties);
        let properties = properties.serialize();
        sqlx::query!(
            "UPDATE playlist SET properties = ? WHERE id = ?",
            properties,
            db_id
        )
        .execute(&mut *db)
        .await?;
    }

    get(db, playlist_id).await
}

pub async fn delete(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    let playlist_id = playlist_id.to_db();
    sqlx::query!("DELETE FROM playlist WHERE id = ?", playlist_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn list_tracks(
    db: &mut DbC,
    playlist_id: PlaylistId,
    params: ListParams,
) -> Result<Vec<PlaylistTrack>> {
    let playlist_id = playlist_id.to_db();
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

pub async fn clear_tracks(db: &mut DbC, playlist_id: PlaylistId) -> Result<()> {
    let playlist_id = playlist_id.to_db();
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
    let playlist_id = playlist_id.to_db();
    for track_id in tracks {
        let track_id = track_id.to_db();
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
    let playlist_id = playlist_id.to_db();
    for track_id in tracks {
        let track_id = track_id.to_db();
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
