use crate::{
    db::{self, Db, DbC},
    property, ArtistId, ImageId, ListParams, Properties, PropertyUpdate, Result, Timestamp,
    ValueUpdate,
};

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub album_count: u32,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct ArtistCreate {
    pub name: String,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
}

#[derive(Debug, Default, Clone)]
pub struct ArtistUpdate {
    pub name: ValueUpdate<String>,
    pub cover_art: ValueUpdate<ImageId>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(sqlx::FromRow)]
struct ArtistView {
    id: i64,
    name: String,
    listen_count: i64,
    cover_art: Option<i64>,
    album_count: i64,
    created_at: i64,
}

impl From<(ArtistView, Properties)> for Artist {
    fn from((value, properties): (ArtistView, Properties)) -> Self {
        Artist {
            id: ArtistId::from_db(value.id),
            name: value.name,
            listen_count: value.listen_count as u32,
            cover_art: value.cover_art.map(ImageId::from_db),
            properties,
            album_count: value.album_count as u32,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Artist>> {
    let views = db::list::<ArtistView>(db, "sqlx_artist", params).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| ArtistId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Artist::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_ids(db: &mut DbC) -> Result<Vec<ArtistId>> {
    let ids = sqlx::query_scalar("SELECT id FROM artist")
        .fetch_all(&mut *db)
        .await?;
    Ok(ids.into_iter().map(ArtistId::from_db).collect())
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, artist_id: ArtistId) -> Result<Artist> {
    let view = sqlx::query_as::<_, ArtistView>("SELECT * FROM sqlx_artist WHERE id = ?")
        .bind(artist_id)
        .fetch_one(&mut *db)
        .await?;
    let properties = property::get(db, ArtistId::from_db(view.id)).await?;
    Ok(From::from((view, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, artist_ids: &[ArtistId]) -> Result<Vec<Artist>> {
    let views = db::list_bulk::<ArtistView, _>(db, "sqlx_artist", artist_ids).await?;
    let properties = property::get_bulk(db, artist_ids.iter().copied()).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(From::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: ArtistCreate) -> Result<Artist> {
    let cover_art = create.cover_art.map(|id| id.to_db());
    let id = sqlx::query_scalar("INSERT INTO artist (name, cover_art) VALUES (?, ?) RETURNING id")
        .bind(&create.name)
        .bind(cover_art)
        .fetch_one(&mut *db)
        .await?;
    let artist_id = ArtistId::from_db(id);
    property::set(db, artist_id, &create.properties).await?;
    get(db, artist_id).await
}

#[tracing::instrument(skip(db))]
pub async fn update(db: &mut DbC, artist_id: ArtistId, update: ArtistUpdate) -> Result<Artist> {
    tracing::info!("updating artist {} with {:#?}", artist_id, update);
    db::value_update_string_non_null(db, "artist", "name", artist_id, update.name).await?;
    db::value_update_id_nullable(db, "artist", "cover_art", artist_id, update.cover_art).await?;
    property::update(db, artist_id, &update.properties).await?;
    get(db, artist_id).await
}

pub async fn delete(db: &mut DbC, artist_id: ArtistId) -> Result<()> {
    sqlx::query("DELETE FROM artist WHERE id = ?")
        .bind(artist_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, artist_id).await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name(db: &mut DbC, create_: ArtistCreate) -> Result<Artist> {
    let artist_id = sqlx::query_scalar("SELECT id FROM artist WHERE name = ?")
        .bind(&create_.name)
        .fetch_optional(&mut *db)
        .await?;
    if let Some(artist_id) = artist_id {
        return get(db, ArtistId::from_db(artist_id)).await;
    }
    create(db, create_).await
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name_tx(db: &Db, create_: ArtistCreate) -> Result<Artist> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}
