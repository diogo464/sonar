use std::time::Duration;

use sqlx::prelude::FromRow;

use crate::{
    db::{self, DbC},
    ListParams, Result, SubscriptionId, Timestamp, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionMediaType {
    Artist,
    Album,
    Track,
    Playlist,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub user: UserId,
    pub created_at: Timestamp,
    pub last_submitted: Option<Timestamp>,
    pub interval: Option<Duration>,
    pub description: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<String>,
    pub playlist: Option<String>,
    pub external_id: Option<String>,
    pub media_type: Option<SubscriptionMediaType>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreate {
    pub user: UserId,
    pub interval: Option<Duration>,
    pub description: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<String>,
    pub playlist: Option<String>,
    pub external_id: Option<String>,
    pub media_type: Option<SubscriptionMediaType>,
}

#[derive(Debug, FromRow)]
struct SubscriptionView {
    id: i64,
    user: i64,
    created_at: i64,
    last_submitted: Option<i64>,
    interval_sec: Option<i64>,
    description: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    track: Option<String>,
    playlist: Option<String>,
    external_id: Option<String>,
    media_type: Option<String>,
}

impl std::fmt::Display for SubscriptionMediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SubscriptionMediaType::Artist => "artist",
            SubscriptionMediaType::Album => "album",
            SubscriptionMediaType::Track => "track",
            SubscriptionMediaType::Playlist => "playlist",
        })
    }
}

impl From<SubscriptionView> for Subscription {
    fn from(value: SubscriptionView) -> Self {
        Self {
            id: SubscriptionId::from_db(value.id),
            user: UserId::from_db(value.user),
            created_at: Timestamp::from_seconds(value.created_at as u64),
            last_submitted: value
                .last_submitted
                .map(|v| Timestamp::from_seconds(v as u64)),
            interval: value.interval_sec.map(|v| Duration::from_secs(v as u64)),
            description: value.description,
            artist: value.artist,
            album: value.album,
            track: value.track,
            playlist: value.playlist,
            external_id: value.external_id,
            media_type: match value.media_type.as_ref().map(|v| v.as_str()) {
                Some("artist") => Some(SubscriptionMediaType::Artist),
                Some("album") => Some(SubscriptionMediaType::Album),
                Some("track") => Some(SubscriptionMediaType::Track),
                Some("playlist") => Some(SubscriptionMediaType::Playlist),
                None => None,
                _ => panic!("database contained invalid subscription media type"),
            },
        }
    }
}

pub async fn list(db: &mut DbC, user_id: UserId, params: ListParams) -> Result<Vec<Subscription>> {
    let views =
        db::list_where_field_eq::<SubscriptionView, _>(db, "subscription", "user", user_id, params)
            .await?;
    Ok(views.into_iter().map(From::from).collect())
}

pub async fn list_all(db: &mut DbC, params: ListParams) -> Result<Vec<Subscription>> {
    let views = db::list::<SubscriptionView>(db, "subscription", params).await?;
    Ok(views.into_iter().map(From::from).collect())
}

pub async fn get(db: &mut DbC, subscription_id: SubscriptionId) -> Result<Subscription> {
    let view = db::get_by_id::<SubscriptionView, _>(db, "subscription", subscription_id).await?;
    Ok(From::from(view))
}

pub async fn create(db: &mut DbC, create: SubscriptionCreate) -> Result<Subscription> {
    let row=  sqlx::query("INSERT INTO subscription(user, interval_sec, description, artist, album, track, playlist, external_id, media_type) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *")
        .bind(create.user)
        .bind(create.interval.map(|i| i.as_secs() as i64))
        .bind(create.description)
        .bind(create.artist)
        .bind(create.album)
        .bind(create.track)
        .bind(create.playlist)
        .bind(create.external_id)
        .bind(create.media_type.map(|m| m.to_string()))
        .fetch_one(db)
        .await?;
    let view = SubscriptionView::from_row(&row)?;
    Ok(From::from(view))
}

pub async fn remove(db: &mut DbC, subscription_id: SubscriptionId) -> Result<()> {
    sqlx::query("DELETE FROM subscription WHERE id = ?")
        .bind(subscription_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn update_last_submitted_to_now(
    db: &mut DbC,
    subscription_id: SubscriptionId,
) -> Result<()> {
    sqlx::query("UPDATE subscription SET last_submitted = unixepoch() WHERE id = ?")
        .bind(subscription_id)
        .execute(db)
        .await?;
    Ok(())
}
