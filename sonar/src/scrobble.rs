use std::time::Duration;

use crate::{
    db::DbC, property, ListParams, Properties, PropertyUpdate, Result, ScrobbleId, Timestamp,
    TrackId, UserId,
};

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

#[derive(Debug, sqlx::FromRow)]
struct ScrobbleView {
    id: i64,
    user: i64,
    track: i64,
    listen_at: i64,
    listen_secs: i64,
    listen_device: String,
    created_at: i64,
}

impl From<(ScrobbleView, Properties)> for Scrobble {
    fn from((value, properties): (ScrobbleView, Properties)) -> Self {
        Self {
            id: ScrobbleId::from_db(value.id),
            user: UserId::from_db(value.user),
            track: TrackId::from_db(value.track),
            listen_at: Timestamp::from_seconds(value.listen_at as u64),
            listen_duration: Duration::from_secs(value.listen_secs as u64),
            listen_device: value.listen_device,
            properties,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Scrobble>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        ScrobbleView,
        "SELECT * FROM scrobble ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| ScrobbleId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Scrobble::from)
        .collect())
}

pub async fn get(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<Scrobble> {
    let scrobble_view = sqlx::query_as!(
        ScrobbleView,
        "SELECT * FROM scrobble WHERE id = ?",
        scrobble_id
    )
    .fetch_one(&mut *db)
    .await?;
    let properties = property::get(db, scrobble_id).await?;
    Ok(Scrobble::from((scrobble_view, properties)))
}

pub async fn create(db: &mut DbC, create: ScrobbleCreate) -> Result<Scrobble> {
    let track_id = create.track.to_db();
    let user_id = create.user.to_db();
    let listen_at = create.listen_at.seconds() as i64;
    let listen_secs = create.listen_duration.as_secs() as i64;
    let scrobble_id = sqlx::query!(
        "INSERT INTO scrobble (user, track, listen_at, listen_secs, listen_device) VALUES (?, ?, ?, ?, ?) RETURNING id",
        user_id,
        track_id,
        listen_at,
        listen_secs,
        create.listen_device,
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    get(db, ScrobbleId::from_db(scrobble_id)).await
}

pub async fn update(
    db: &mut DbC,
    scrobble_id: ScrobbleId,
    update: ScrobbleUpdate,
) -> Result<Scrobble> {
    property::update(db, scrobble_id, &update.properties).await?;
    get(db, scrobble_id).await
}

pub async fn delete(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<()> {
    let scrobble_id = scrobble_id.to_db();
    sqlx::query!("DELETE FROM scrobble WHERE id = ?", scrobble_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
