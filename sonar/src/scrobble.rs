use std::time::Duration;

use crate::{
    db::{self, DbC},
    property, ListParams, Properties, PropertyUpdate, Result, ScrobbleId, Timestamp, TrackId,
    UserId,
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

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Scrobble>> {
    let views = db::list::<ScrobbleView>(db, "scrobble", params).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| ScrobbleId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Scrobble::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<Scrobble> {
    let view = db::get_by_id(db, "scrobble", scrobble_id).await?;
    let properties = property::get(db, scrobble_id).await?;
    Ok(Scrobble::from((view, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, scrobble_ids: &[ScrobbleId]) -> Result<Vec<Scrobble>> {
    let mut scrobbles = Vec::with_capacity(scrobble_ids.len());
    for scrobble_id in scrobble_ids {
        scrobbles.push(get(db, *scrobble_id).await?);
    }
    Ok(scrobbles)
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: ScrobbleCreate) -> Result<Scrobble> {
    let track_id = create.track.to_db();
    let user_id = create.user.to_db();
    let listen_at = create.listen_at.seconds() as i64;
    let listen_secs = create.listen_duration.as_secs() as i64;
    let scrobble_id = sqlx::query_scalar(
        "INSERT INTO scrobble (user, track, listen_at, listen_secs, listen_device) VALUES (?, ?, ?, ?, ?) RETURNING id")
        .bind(user_id)
        .bind(track_id)
        .bind(listen_at)
        .bind(listen_secs)
        .bind(create.listen_device)
    .fetch_one(&mut *db)
    .await?;

    get(db, ScrobbleId::from_db(scrobble_id)).await
}

#[tracing::instrument(skip(db))]
pub async fn update(
    db: &mut DbC,
    scrobble_id: ScrobbleId,
    update: ScrobbleUpdate,
) -> Result<Scrobble> {
    property::update(db, scrobble_id, &update.properties).await?;
    get(db, scrobble_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<()> {
    let scrobble_id = scrobble_id.to_db();
    sqlx::query("DELETE FROM scrobble WHERE id = ?")
        .bind(scrobble_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn list_unsubmitted(db: &mut DbC, scrobbler: &str) -> Result<Vec<Scrobble>> {
    let ids = sqlx::query_scalar(
        "
SELECT sc.id
FROM scrobble sc
LEFT JOIN scrobble_submission ss ON sc.id = ss.scrobble AND ss.scrobbler = ?
WHERE ss.scrobble IS NULL
LIMIT 100
",
    )
    .bind(scrobbler)
    .fetch_all(&mut *db)
    .await?;

    let ids = ids.into_iter().map(ScrobbleId::from_db).collect::<Vec<_>>();
    get_bulk(db, &ids).await
}

#[tracing::instrument(skip(db))]
pub async fn list_unsubmitted_for_user(
    db: &mut DbC,
    user_id: UserId,
    scrobbler: &str,
) -> Result<Vec<Scrobble>> {
    let ids = sqlx::query_scalar(
        "
SELECT sc.id
FROM scrobble sc
LEFT JOIN scrobble_submission ss ON sc.id = ss.scrobble AND ss.scrobbler = ?
WHERE ss.scrobble IS NULL AND sc.user = ?
LIMIT 100
",
    )
    .bind(scrobbler)
    .bind(user_id)
    .fetch_all(&mut *db)
    .await?;

    let ids = ids.into_iter().map(ScrobbleId::from_db).collect::<Vec<_>>();
    get_bulk(db, &ids).await
}

#[tracing::instrument(skip(db))]
pub async fn register_submission(
    db: &mut DbC,
    scrobble_id: ScrobbleId,
    scrobbler: &str,
) -> Result<()> {
    let scrobble_id = scrobble_id.to_db();
    sqlx::query("INSERT INTO scrobble_submission (scrobble, scrobbler) VALUES (?, ?)")
        .bind(scrobble_id)
        .bind(scrobbler)
        .execute(&mut *db)
        .await?;
    Ok(())
}
