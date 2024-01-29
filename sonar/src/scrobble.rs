use std::time::Duration;

use crate::{
    DbC, ListParams, Properties, Result, Scrobble, ScrobbleCreate, ScrobbleId, ScrobbleUpdate,
    Timestamp, TrackId, UserId,
};

#[derive(Debug, sqlx::FromRow)]
struct ScrobbleView {
    id: i64,
    user: i64,
    track: i64,
    listen_at: i64,
    listen_secs: i64,
    listen_device: String,
    properties: Option<Vec<u8>>,
    created_at: i64,
}

impl ScrobbleView {
    fn into_scrobble(self) -> Scrobble {
        Scrobble {
            id: ScrobbleId::from_db(self.id),
            user: UserId::from_db(self.user),
            track: TrackId::from_db(self.track),
            listen_at: Timestamp::from_seconds(self.listen_at as u64),
            listen_duration: Duration::from_secs(self.listen_secs as u64),
            listen_device: self.listen_device,
            properties: Properties::deserialize_unchecked(&self.properties.unwrap_or_default()),
            created_at: Timestamp::from_seconds(self.created_at as u64),
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
    Ok(views.into_iter().map(ScrobbleView::into_scrobble).collect())
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
    Ok(scrobble_view.into_scrobble())
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
    if update.properties.len() > 0 {
        let properties =
            sqlx::query_scalar!("SELECT properties FROM scrobble WHERE id = ?", scrobble_id)
                .fetch_one(&mut *db)
                .await?;
        let mut properties = Properties::deserialize_unchecked(&properties.unwrap_or_default());
        properties.apply_updates(&update.properties);
        let properties = properties.serialize();
        sqlx::query!(
            "UPDATE scrobble SET properties = ? WHERE id = ?",
            properties,
            scrobble_id
        )
        .execute(&mut *db)
        .await?;
    }
    get(db, scrobble_id).await
}

pub async fn delete(db: &mut DbC, scrobble_id: ScrobbleId) -> Result<()> {
    let scrobble_id = scrobble_id.to_db();
    sqlx::query!("DELETE FROM scrobble WHERE id = ?", scrobble_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
