use std::time::Duration;

use crate::{
    property, DbC, ListParams, Properties, Result, Scrobble, ScrobbleCreate, ScrobbleId,
    ScrobbleUpdate, Timestamp, TrackId, UserId,
};

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

impl ScrobbleView {
    fn into_scrobble(self, properties: Properties) -> Scrobble {
        Scrobble {
            id: ScrobbleId::from_db(self.id),
            user: UserId::from_db(self.user),
            track: TrackId::from_db(self.track),
            listen_at: Timestamp::from_seconds(self.listen_at as u64),
            listen_duration: Duration::from_secs(self.listen_secs as u64),
            listen_device: self.listen_device,
            created_at: Timestamp::from_seconds(self.created_at as u64),
            properties,
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

    let mut scrobbles = Vec::with_capacity(views.len());
    for view in views {
        let properties =
            crate::property::get(&mut *db, crate::property::Namespace::Scrobble, view.id).await?;
        scrobbles.push(view.into_scrobble(properties));
    }

    Ok(scrobbles)
}

pub async fn get(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<Scrobble> {
    let scrobble_id = scrobble_id.to_db();
    let scrobble_view = sqlx::query_as!(
        ScrobbleView,
        "SELECT * FROM scrobble WHERE id = ?",
        scrobble_id
    )
    .fetch_one(&mut *db)
    .await?;

    let properties =
        crate::property::get(&mut *db, crate::property::Namespace::Scrobble, scrobble_id).await?;
    Ok(scrobble_view.into_scrobble(properties))
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
    property::update(
        db,
        property::Namespace::Scrobble,
        scrobble_id.to_db(),
        &update.properties,
    )
    .await?;
    get(db, scrobble_id).await
}

pub async fn delete(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<()> {
    let scrobble_id = scrobble_id.to_db();
    sqlx::query!("DELETE FROM scrobble WHERE id = ?", scrobble_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
