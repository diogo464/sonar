use crate::{
    db::{Db, DbC},
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

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Artist>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        ArtistView,
        "SELECT * FROM sqlx_artist ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| ArtistId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Artist::from)
        .collect())
}

pub async fn list_ids(db: &mut DbC) -> Result<Vec<ArtistId>> {
    let ids = sqlx::query_scalar!("SELECT id FROM artist")
        .fetch_all(&mut *db)
        .await?;
    Ok(ids.into_iter().map(ArtistId::from_db).collect())
}

pub async fn get(db: &mut DbC, artist_id: ArtistId) -> Result<Artist> {
    let artist_id = artist_id.to_db();
    let artist_view = sqlx::query_as!(
        ArtistView,
        "SELECT * FROM sqlx_artist WHERE id = ?",
        artist_id
    )
    .fetch_one(&mut *db)
    .await?;
    let properties = property::get(db, ArtistId::from_db(artist_id)).await?;
    Ok(From::from((artist_view, properties)))
}

pub async fn get_bulk(db: &mut DbC, artist_ids: &[ArtistId]) -> Result<Vec<Artist>> {
    let mut artists = Vec::with_capacity(artist_ids.len());
    for artist_id in artist_ids {
        artists.push(get(db, *artist_id).await?);
    }
    Ok(artists)
}

pub async fn create(db: &mut DbC, create: ArtistCreate) -> Result<Artist> {
    let cover_art = create.cover_art.map(|id| id.to_db());
    let artist_id = sqlx::query!(
        r#"INSERT INTO artist (name, cover_art) VALUES (?, ?) RETURNING id"#,
        create.name,
        cover_art,
    )
    .fetch_one(&mut *db)
    .await?
    .id;
    let artist_id = ArtistId::from_db(artist_id);
    property::set(db, artist_id, &create.properties).await?;
    get(db, artist_id).await
}

pub async fn update(db: &mut DbC, artist_id: ArtistId, update: ArtistUpdate) -> Result<Artist> {
    tracing::info!("updating artist {} with {:?}", artist_id, update);
    let new_name = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_string()),
        ValueUpdate::Unchanged => None,
    };
    if let Some(name) = new_name {
        sqlx::query!("UPDATE artist SET name = ? WHERE id = ?", name, artist_id)
            .execute(&mut *db)
            .await?;
    }
    property::update(db, artist_id, &update.properties).await?;
    get(db, artist_id).await
}

pub async fn delete(db: &mut DbC, artist_id: ArtistId) -> Result<()> {
    let artist_id = artist_id.to_db();
    sqlx::query!("DELETE FROM artist WHERE id = ?", artist_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, ArtistId::from_db(artist_id)).await?;
    Ok(())
}

pub async fn find_or_create_by_name(db: &mut DbC, create_: ArtistCreate) -> Result<Artist> {
    let artist_name = &create_.name;
    let artist_id = sqlx::query_scalar!(r#"SELECT id FROM artist WHERE name = ?"#, artist_name)
        .fetch_optional(&mut *db)
        .await?;

    if let Some(artist_id) = artist_id {
        return get(db, ArtistId::from_db(artist_id)).await;
    }

    create(db, create_).await
}

pub async fn find_or_create_by_name_tx(db: &Db, create_: ArtistCreate) -> Result<Artist> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}
