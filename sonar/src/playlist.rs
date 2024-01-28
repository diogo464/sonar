use crate::{
    property, DbC, ListParams, Playlist, PlaylistCreate, PlaylistId, PlaylistTrack, PlaylistUpdate,
    Properties, Result, Timestamp, TrackId, UserId, ValueUpdate,
};

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
    owner: i64,
    track_count: i64,
}

impl PlaylistView {
    fn into_playlist(self, properties: Properties) -> Playlist {
        Playlist {
            id: PlaylistId::from_db(self.id),
            name: self.name,
            owner: UserId::from_db(self.owner),
            track_count: self.track_count as u32,
            properties,
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

    let mut playlists = Vec::with_capacity(views.len());
    for view in views {
        let properties =
            crate::property::get(&mut *db, crate::property::Namespace::Playlist, view.id).await?;
        playlists.push(view.into_playlist(properties));
    }

    Ok(playlists)
}

pub async fn get(db: &mut DbC, playlist_id: PlaylistId) -> Result<Playlist> {
    let playlist_id = playlist_id.to_db();
    let playlist_view = sqlx::query_as!(
        PlaylistView,
        "SELECT * FROM playlist_view WHERE id = ?",
        playlist_id
    )
    .fetch_one(&mut *db)
    .await?;

    let properties =
        crate::property::get(&mut *db, crate::property::Namespace::Playlist, playlist_id).await?;
    Ok(playlist_view.into_playlist(properties))
}

pub async fn create(db: &mut DbC, create: PlaylistCreate) -> Result<Playlist> {
    let name = create.name.as_str();
    let owner = create.owner.to_db();
    let playlist_id = sqlx::query!(
        "INSERT INTO playlist(name, owner) VALUES (?, ?) RETURNING id",
        name,
        owner
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    property::set(
        &mut *db,
        property::Namespace::Playlist,
        playlist_id,
        &create.properties,
    )
    .await?;

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

    property::update(
        &mut *db,
        property::Namespace::Playlist,
        db_id,
        &update.properties,
    )
    .await?;

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
