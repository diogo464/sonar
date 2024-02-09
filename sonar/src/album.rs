use std::time::Duration;

use crate::{
    db::{Db, DbC},
    property, AlbumId, ArtistId, Error, ErrorKind, ImageId, ListParams, Properties, PropertyUpdate,
    Result, Timestamp, ValueUpdate,
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
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumUpdate {
    pub name: ValueUpdate<String>,
    pub artist: ValueUpdate<ArtistId>,
    pub cover_art: ValueUpdate<ImageId>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(sqlx::FromRow)]
struct AlbumView {
    id: i64,
    name: String,
    duration_ms: Option<i64>,
    artist: i64,
    listen_count: Option<i64>,
    cover_art: Option<i64>,
    track_count: Option<i64>,
    created_at: i64,
}

impl From<(AlbumView, Properties)> for Album {
    fn from((value, properties): (AlbumView, Properties)) -> Self {
        Album {
            id: AlbumId::from_db(value.id),
            name: value.name,
            duration: Duration::from_millis(value.duration_ms.unwrap_or_default() as u64),
            artist: ArtistId::from_db(value.artist),
            listen_count: value.listen_count.unwrap_or_default() as u32,
            cover_art: value.cover_art.map(ImageId::from_db),
            properties,
            track_count: value.track_count.unwrap_or_default() as u32,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Album>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        AlbumView,
        "SELECT * FROM sqlx_album ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Album::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_by_artist(
    db: &mut DbC,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Album>> {
    let (offset, limit) = params.to_db_offset_limit();
    let artist_id = artist_id.to_db();
    let views = sqlx::query_as!(
        AlbumView,
        "SELECT * FROM sqlx_album WHERE artist = ? ORDER BY id ASC LIMIT ? OFFSET ?",
        artist_id,
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Album::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_artist_id_pairs(db: &mut DbC) -> Result<Vec<(AlbumId, ArtistId)>> {
    let rows = sqlx::query!("SELECT id, artist FROM album")
        .fetch_all(&mut *db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (AlbumId::from_db(row.id), ArtistId::from_db(row.artist)))
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, album_id: AlbumId) -> Result<Album> {
    let album_view = sqlx::query_as!(AlbumView, "SELECT * FROM sqlx_album WHERE id = ?", album_id)
        .fetch_one(&mut *db)
        .await?;
    let properties = property::get(db, AlbumId::from_db(album_view.id)).await?;
    Ok(From::from((album_view, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, album_ids: &[AlbumId]) -> Result<Vec<Album>> {
    let mut albums = Vec::with_capacity(album_ids.len());
    for album_id in album_ids {
        albums.push(get(db, *album_id).await?);
    }
    Ok(albums)
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: AlbumCreate) -> Result<Album> {
    let name = create.name;
    let cover_art = create.cover_art.map(|id| id.to_db());

    let album_id = sqlx::query!(
        "INSERT INTO album (artist, name, cover_art) VALUES (?, ?, ?) RETURNING id",
        create.artist,
        name,
        cover_art,
    )
    .fetch_one(&mut *db)
    .await?
    .id;
    let album_id = AlbumId::from_db(album_id);
    property::set(db, album_id, &create.properties).await?;

    get(db, album_id).await
}

#[tracing::instrument(skip(db))]
pub async fn update(db: &mut DbC, album_id: AlbumId, update: AlbumUpdate) -> Result<Album> {
    tracing::info!("updating album {} with {:?}", album_id, update);
    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!("UPDATE album SET name = ? WHERE id = ?", new_name, album_id)
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
            album_id
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
                album_id
            )
            .execute(&mut *db)
            .await?;
        }
        ValueUpdate::Unset => {
            sqlx::query!("UPDATE album SET cover_art = NULL WHERE id = ?", album_id)
                .execute(&mut *db)
                .await?;
        }
        ValueUpdate::Unchanged => {}
    }
    property::update(db, album_id, &update.properties).await?;

    get(db, album_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, album_id: AlbumId) -> Result<()> {
    sqlx::query!("DELETE FROM album WHERE id = ?", album_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, album_id).await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name(db: &mut DbC, create_: AlbumCreate) -> Result<Album> {
    let name = &create_.name;
    let album_id = sqlx::query!("SELECT id FROM album WHERE name = ?", name)
        .fetch_optional(&mut *db)
        .await?
        .map(|row| AlbumId::from_db(row.id));

    if let Some(album_id) = album_id {
        return get(db, album_id).await;
    }

    create(db, create_).await
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name_tx(db: &Db, create_: AlbumCreate) -> Result<Album> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}
