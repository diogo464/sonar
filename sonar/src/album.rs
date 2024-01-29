use std::time::Duration;

use crate::{
    db::DbC, AlbumId, ArtistId, DateTime, Error, ErrorKind, ImageId, ListParams, Properties,
    PropertyUpdate, Result, Timestamp, ValueUpdate,
};

#[derive(Debug, Clone)]
pub struct Album {
    pub id: AlbumId,
    pub name: String,
    pub duration: Duration,
    pub artist: ArtistId,
    pub track_count: u32,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct AlbumCreate {
    pub name: String,
    pub artist: ArtistId,
    pub cover_art: Option<ImageId>,
    pub release_date: DateTime,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumUpdate {
    pub name: ValueUpdate<String>,
    pub artist: ValueUpdate<ArtistId>,
    pub release_date: ValueUpdate<DateTime>,
    pub cover_art: ValueUpdate<ImageId>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(sqlx::FromRow)]
struct AlbumView {
    id: i64,
    name: String,
    duration_ms: i64,
    artist: i64,
    listen_count: i64,
    cover_art: Option<i64>,
    track_count: i64,
    properties: Option<Vec<u8>>,
    created_at: i64,
}

impl From<AlbumView> for Album {
    fn from(value: AlbumView) -> Self {
        Album {
            id: AlbumId::from_db(value.id),
            name: value.name,
            duration: Duration::from_millis(value.duration_ms as u64),
            artist: ArtistId::from_db(value.artist),
            listen_count: value.listen_count as u32,
            cover_art: value.cover_art.map(ImageId::from_db),
            properties: Properties::deserialize_unchecked(&value.properties.unwrap_or_default()),
            track_count: value.track_count as u32,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Album>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        AlbumView,
        "SELECT * FROM album_view ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(views.into_iter().map(Album::from).collect())
}

pub async fn list_by_artist(
    db: &mut DbC,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Album>> {
    let artist_id = artist_id.to_db();
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        AlbumView,
        "SELECT * FROM album_view WHERE artist = ? ORDER BY id ASC LIMIT ? OFFSET ?",
        artist_id,
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(views.into_iter().map(Album::from).collect())
}

pub async fn get(db: &mut DbC, album_id: AlbumId) -> Result<Album> {
    let album_id = album_id.to_db();
    let album_view = sqlx::query_as!(AlbumView, "SELECT * FROM album_view WHERE id = ?", album_id)
        .fetch_one(&mut *db)
        .await?;
    Ok(From::from(album_view))
}

pub async fn get_bulk(db: &mut DbC, album_ids: &[AlbumId]) -> Result<Vec<Album>> {
    let mut albums = Vec::with_capacity(album_ids.len());
    for album_id in album_ids {
        albums.push(get(db, *album_id).await?);
    }
    Ok(albums)
}

pub async fn create(db: &mut DbC, create: AlbumCreate) -> Result<Album> {
    let artist_id = create.artist.to_db();
    let name = create.name;
    let cover_art = create.cover_art.map(|id| id.to_db());
    let properties = create.properties.serialize();

    let album_id = sqlx::query!(
        "INSERT INTO album (artist, name, cover_art, properties) VALUES (?, ?, ?, ?) RETURNING id",
        artist_id,
        name,
        cover_art,
        properties,
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    get(db, AlbumId::from_db(album_id)).await
}

pub async fn update(db: &mut DbC, album_id: AlbumId, update: AlbumUpdate) -> Result<Album> {
    let db_id = album_id.to_db();

    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!("UPDATE album SET name = ? WHERE id = ?", new_name, db_id)
            .execute(&mut *db)
            .await?;
    }

    if let Some(new_artist) = match update.artist {
        ValueUpdate::Set(artist) => Some(artist.to_db()),
        ValueUpdate::Unset => {
            return Err(Error::new(
                ErrorKind::Invalid,
                "cannot unset artist on album update",
            ))
        }
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!(
            "UPDATE album SET artist = ? WHERE id = ?",
            new_artist,
            db_id
        )
        .execute(&mut *db)
        .await?;
    }

    match update.cover_art {
        ValueUpdate::Set(image_id) => {
            let image_id = image_id.to_db();
            sqlx::query!(
                "UPDATE album SET cover_art = ? WHERE id = ?",
                image_id,
                db_id
            )
            .execute(&mut *db)
            .await?;
        }
        ValueUpdate::Unset => {
            sqlx::query!("UPDATE album SET cover_art = NULL WHERE id = ?", db_id)
                .execute(&mut *db)
                .await?;
        }
        ValueUpdate::Unchanged => {}
    }

    if update.properties.len() > 0 {
        let properties = sqlx::query_scalar!("SELECT properties FROM album WHERE id = ?", db_id)
            .fetch_one(&mut *db)
            .await?
            .unwrap_or_default();
        let mut properties = Properties::deserialize_unchecked(&properties);
        properties.apply_updates(&update.properties);
        let properties = properties.serialize();
        sqlx::query!(
            "UPDATE album SET properties = ? WHERE id = ?",
            properties,
            db_id
        )
        .execute(&mut *db)
        .await?;
    }

    get(db, album_id).await
}

pub async fn delete(db: &mut DbC, album_id: AlbumId) -> Result<()> {
    let album_id = album_id.to_db();
    sqlx::query!("DELETE FROM album WHERE id = ?", album_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn find_or_create(db: &mut DbC, name: &str, create_: AlbumCreate) -> Result<Album> {
    let album_id = sqlx::query!("SELECT id FROM album WHERE name = ?", name)
        .fetch_optional(&mut *db)
        .await?
        .map(|row| AlbumId::from_db(row.id));

    if let Some(album_id) = album_id {
        return get(db, album_id).await;
    }

    create(db, create_).await
}
