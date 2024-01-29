use crate::{
    Artist, ArtistCreate, ArtistId, ArtistUpdate, DbC, ImageId, ListParams, Properties, Result,
    Timestamp,
};

#[derive(sqlx::FromRow)]
struct ArtistView {
    id: i64,
    name: String,
    listen_count: i64,
    cover_art: Option<i64>,
    album_count: i64,
    properties: Option<Vec<u8>>,
    created_at: i64,
}

impl ArtistView {
    fn into_artist(self) -> Artist {
        Artist {
            id: ArtistId::from_db(self.id),
            name: self.name,
            listen_count: self.listen_count as u32,
            cover_art: self.cover_art.map(ImageId::from_db),
            properties: Properties::deserialize_unchecked(&self.properties.unwrap_or_default()),
            album_count: self.album_count as u32,
            created_at: Timestamp::from_seconds(self.created_at as u64),
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Artist>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        ArtistView,
        "SELECT * FROM artist_view ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(views.into_iter().map(ArtistView::into_artist).collect())
}

pub async fn get(db: &mut DbC, artist_id: ArtistId) -> Result<Artist> {
    let artist_id = artist_id.to_db();
    let artist_view = sqlx::query_as!(
        ArtistView,
        "SELECT * FROM artist_view WHERE id = ?",
        artist_id
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(artist_view.into_artist())
}

pub async fn get_bulk(db: &mut DbC, artist_ids: &[ArtistId]) -> Result<Vec<Artist>> {
    let mut artists = Vec::with_capacity(artist_ids.len());
    for artist_id in artist_ids {
        artists.push(get(db, *artist_id).await?);
    }
    Ok(artists)
}

pub async fn create(db: &mut DbC, create: ArtistCreate) -> Result<Artist> {
    let properties = create.properties.serialize();
    let cover_art = create.cover_art.map(|id| id.to_db());
    let artist_id = sqlx::query!(
        r#"INSERT INTO artist (name, cover_art, properties) VALUES (?, ?, ?) RETURNING id"#,
        create.name,
        cover_art,
        properties
    )
    .fetch_one(&mut *db)
    .await?
    .id;
    get(db, ArtistId::from_db(artist_id)).await
}

pub async fn update(db: &mut DbC, artist_id: ArtistId, update: ArtistUpdate) -> Result<Artist> {
    let new_name = match update.name {
        crate::ValueUpdate::Set(name) => Some(name),
        crate::ValueUpdate::Unset => Some("".to_string()),
        crate::ValueUpdate::Unchanged => None,
    };
    if let Some(name) = new_name {
        sqlx::query!("UPDATE artist SET name = ? WHERE id = ?", name, artist_id)
            .execute(&mut *db)
            .await?;
    }

    if update.properties.len() > 0 {
        let properties =
            sqlx::query_scalar!("SELECT properties FROM artist WHERE id = ?", artist_id)
                .fetch_one(&mut *db)
                .await?
                .unwrap_or_default();
        let mut properties = Properties::deserialize_unchecked(&properties);
        properties.apply_updates(&update.properties);
        let properties = properties.serialize();
        sqlx::query!(
            "UPDATE artist SET properties = ? WHERE id = ?",
            properties,
            artist_id
        )
        .execute(&mut *db)
        .await?;
    }

    Ok(get(db, artist_id).await?)
}

pub async fn delete(db: &mut DbC, artist_id: ArtistId) -> Result<()> {
    let artist_id = artist_id.to_db();
    sqlx::query!("DELETE FROM artist WHERE id = ?", artist_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn find_or_create(
    db: &mut DbC,
    artist_name: &str,
    create_: ArtistCreate,
) -> Result<Artist> {
    let artist_id = sqlx::query_scalar!(r#"SELECT id FROM artist WHERE name = ?"#, artist_name)
        .fetch_optional(&mut *db)
        .await?;

    if let Some(artist_id) = artist_id {
        return get(db, ArtistId::from_db(artist_id)).await;
    }

    create(db, create_).await
}
