use std::time::Duration;

use sqlx::Row;

use crate::{
    db::{self, Db, DbC},
    genre, property, AlbumId, ArtistId, GenreUpdate, Genres, ImageId, ListParams, Properties,
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
    pub genres: Genres,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct AlbumCreate {
    pub name: String,
    pub artist: ArtistId,
    pub cover_art: Option<ImageId>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumUpdate {
    pub name: ValueUpdate<String>,
    pub artist: ValueUpdate<ArtistId>,
    pub cover_art: ValueUpdate<ImageId>,
    pub genres: Vec<GenreUpdate>,
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

impl From<(AlbumView, Genres, Properties)> for Album {
    fn from((value, genres, properties): (AlbumView, Genres, Properties)) -> Self {
        Album {
            id: AlbumId::from_db(value.id),
            name: value.name,
            duration: Duration::from_millis(value.duration_ms.unwrap_or_default() as u64),
            artist: ArtistId::from_db(value.artist),
            listen_count: value.listen_count.unwrap_or_default() as u32,
            cover_art: value.cover_art.map(ImageId::from_db),
            genres,
            properties,
            track_count: value.track_count.unwrap_or_default() as u32,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Album>> {
    let views = db::list::<AlbumView>(db, "sqlx_album", params).await?;
    let genres = genre::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    Ok(db::merge_view_genres_properties(views, genres, properties))
}

#[tracing::instrument(skip(db))]
pub async fn list_by_artist(
    db: &mut DbC,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Album>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::QueryBuilder::new(
        "SELECT * FROM sqlx_album WHERE artist = ? ORDER BY id ASC LIMIT ? OFFSET ?",
    )
    .build_query_as::<AlbumView>()
    .bind(artist_id.to_db())
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *db)
    .await?;
    let genres = genre::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| AlbumId::from_db(view.id))).await?;
    Ok(db::merge_view_genres_properties(views, genres, properties))
}

#[tracing::instrument(skip(db))]
pub async fn list_artist_id_pairs(db: &mut DbC) -> Result<Vec<(AlbumId, ArtistId)>> {
    let rows = sqlx::query("SELECT id, artist FROM album")
        .fetch_all(&mut *db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (AlbumId::from_db(row.get(0)), ArtistId::from_db(row.get(1))))
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, album_id: AlbumId) -> Result<Album> {
    let view = sqlx::query_as::<_, AlbumView>("SELECT * FROM sqlx_album WHERE id = ?")
        .bind(album_id)
        .fetch_one(&mut *db)
        .await?;
    let genres = genre::get(db, AlbumId::from_db(view.id)).await?;
    let properties = property::get(db, AlbumId::from_db(view.id)).await?;
    Ok(From::from((view, genres, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, album_ids: &[AlbumId]) -> Result<Vec<Album>> {
    let mut query = sqlx::QueryBuilder::new("SELECT * FROM sqlx_album WHERE id IN");
    db::query_builder_push_id_tuple(&mut query, album_ids.iter().copied());
    let albums = query
        .build_query_as::<AlbumView>()
        .fetch_all(&mut *db)
        .await?;
    let album_ids = albums
        .iter()
        .map(|album| AlbumId::from_db(album.id))
        .collect::<Vec<_>>();
    let genres = genre::get_bulk(db, album_ids.iter().copied()).await?;
    let properties = property::get_bulk(db, album_ids.iter().copied()).await?;
    Ok(db::merge_view_genres_properties(albums, genres, properties))
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: AlbumCreate) -> Result<Album> {
    let cover_art = create.cover_art.map(|id| id.to_db());
    let query =
        sqlx::query("INSERT INTO album (artist, name, cover_art) VALUES (?, ?, ?) RETURNING id")
            .bind(create.artist)
            .bind(&create.name)
            .bind(cover_art)
            .fetch_one(&mut *db)
            .await?;
    let album_id = AlbumId::from_db(query.get(0));
    genre::set(db, album_id, &create.genres).await?;
    property::set(db, album_id, &create.properties).await?;
    get(db, album_id).await
}

#[tracing::instrument(skip(db))]
pub async fn update(db: &mut DbC, album_id: AlbumId, update: AlbumUpdate) -> Result<Album> {
    tracing::info!("updating album {} with {:#?}", album_id, update);
    db::value_update_string_non_null(db, "album", "name", album_id, update.name).await?;
    db::value_update_id_non_null(db, "album", "artist", album_id, update.artist).await?;
    db::value_update_id_nullable(db, "album", "cover_art", album_id, update.cover_art).await?;
    genre::update(db, album_id, &update.genres).await?;
    property::update(db, album_id, &update.properties).await?;
    get(db, album_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, album_id: AlbumId) -> Result<()> {
    sqlx::query("DELETE FROM album WHERE id = ?")
        .bind(album_id)
        .execute(&mut *db)
        .await?;
    genre::clear(db, album_id).await?;
    property::clear(db, album_id).await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name(db: &mut DbC, create_: AlbumCreate) -> Result<Album> {
    let album_id = sqlx::query("SELECT id FROM album WHERE name = ?")
        .bind(&create_.name)
        .fetch_optional(&mut *db)
        .await?
        .map(|row| AlbumId::from_db(row.get(0)));

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
