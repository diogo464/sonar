use crate::{
    genre, property, Artist, ArtistCreate, ArtistId, ArtistUpdate, DbC, Genres, ImageId,
    ListParams, Properties, Result,
};

#[derive(sqlx::FromRow)]
struct ArtistView {
    id: i64,
    name: String,
    listen_count: i64,
    cover_art: Option<i64>,
    album_count: i64,
}

impl ArtistView {
    fn into_artist(self, genres: Genres, properties: Properties) -> Artist {
        Artist {
            id: ArtistId::from_db(self.id),
            name: self.name,
            listen_count: self.listen_count as u32,
            genres,
            cover_art: self.cover_art.map(ImageId::from_db),
            properties,
            album_count: self.album_count as u32,
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

    let mut artists = Vec::with_capacity(views.len());
    for view in views {
        let genres = genre::get(&mut *db, genre::Namespace::Artist, view.id).await?;
        let properties = property::get(&mut *db, property::Namespace::Artist, view.id).await?;
        artists.push(view.into_artist(genres, properties));
    }

    Ok(artists)
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

    let genres = genre::get(&mut *db, genre::Namespace::Artist, artist_id).await?;
    let properties = property::get(&mut *db, property::Namespace::Artist, artist_id).await?;

    Ok(artist_view.into_artist(genres, properties))
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
        cover_art
    )
    .fetch_one(&mut *db)
    .await?
    .id;
    tracing::info!("created artist {}", artist_id);

    property::set(
        &mut *db,
        property::Namespace::Artist,
        artist_id,
        &create.properties,
    )
    .await?;

    genre::set(
        &mut *db,
        genre::Namespace::Artist,
        artist_id,
        &create.genres,
    )
    .await?;

    let artist_view = sqlx::query_as!(
        ArtistView,
        "SELECT * FROM artist_view WHERE id = ?",
        artist_id
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(artist_view.into_artist(create.genres, create.properties))
}

pub async fn update(db: &mut DbC, artist_id: ArtistId, update: ArtistUpdate) -> Result<Artist> {
    let db_id = artist_id.to_db();
    let new_name = match update.name {
        crate::ValueUpdate::Set(name) => Some(name),
        crate::ValueUpdate::Unset => Some("".to_string()),
        crate::ValueUpdate::Unchanged => None,
    };
    if let Some(name) = new_name {
        sqlx::query!("UPDATE artist SET name = ? WHERE id = ?", name, db_id)
            .execute(&mut *db)
            .await?;
    }

    genre::update(&mut *db, genre::Namespace::Artist, db_id, &update.genres).await?;
    property::update(
        &mut *db,
        property::Namespace::Artist,
        db_id,
        &update.properties,
    )
    .await?;

    Ok(get(db, artist_id).await?)
}

pub async fn delete(db: &mut DbC, artist_id: ArtistId) -> Result<()> {
    let artist_id = artist_id.to_db();
    sqlx::query!("DELETE FROM artist WHERE id = ?", artist_id)
        .execute(&mut *db)
        .await?;
    property::clear(&mut *db, property::Namespace::Artist, artist_id).await?;
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
