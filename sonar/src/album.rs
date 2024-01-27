use crate::{
    genre, property, Album, AlbumCreate, AlbumId, AlbumUpdate, ArtistId, DbC, Error, ErrorKind,
    Genres, ImageId, ListParams, Properties, Result, ValueUpdate,
};

#[derive(sqlx::FromRow)]
struct AlbumView {
    id: i64,
    name: String,
    artist: i64,
    release_date: String,
    listen_count: i64,
    cover_art: Option<i64>,
    track_count: i64,
}

impl AlbumView {
    fn into_album(self, genres: Genres, properties: Properties) -> Album {
        Album {
            id: AlbumId::from_db(self.id),
            name: self.name,
            artist: ArtistId::from_db(self.artist),
            release_date: self.release_date.parse().unwrap(),
            listen_count: self.listen_count as u32,
            cover_art: self.cover_art.map(ImageId::from_db),
            genres,
            properties,
            track_count: self.track_count as u32,
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

    let mut albums = Vec::with_capacity(views.len());
    for view in views {
        let genres = crate::genre::get(&mut *db, crate::genre::Namespace::Album, view.id).await?;
        let properties =
            crate::property::get(&mut *db, crate::property::Namespace::Album, view.id).await?;
        albums.push(view.into_album(genres, properties));
    }

    Ok(albums)
}

pub async fn get(db: &mut DbC, album_id: AlbumId) -> Result<Album> {
    let album_id = album_id.to_db();
    let album_view = sqlx::query_as!(AlbumView, "SELECT * FROM album_view WHERE id = ?", album_id)
        .fetch_one(&mut *db)
        .await?;

    let genres = crate::genre::get(&mut *db, crate::genre::Namespace::Album, album_id).await?;
    let properties =
        crate::property::get(&mut *db, crate::property::Namespace::Album, album_id).await?;
    Ok(album_view.into_album(genres, properties))
}

pub async fn create(db: &mut DbC, create: AlbumCreate) -> Result<Album> {
    let artist_id = create.artist.to_db();
    let name = create.name;
    let cover_art = create.cover_art.map(|id| id.to_db());
    let genres = create.genres;
    let properties = create.properties;

    let album_id = sqlx::query!(
        "INSERT INTO album (artist, name, cover_art) VALUES (?, ?, ?) RETURNING id",
        artist_id,
        name,
        cover_art
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    crate::genre::set(&mut *db, crate::genre::Namespace::Album, album_id, &genres).await?;
    crate::property::set(
        &mut *db,
        crate::property::Namespace::Album,
        album_id,
        &properties,
    )
    .await?;

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

    genre::update(&mut *db, genre::Namespace::Album, db_id, &update.genres).await?;
    property::update(
        &mut *db,
        property::Namespace::Album,
        db_id,
        &update.properties,
    )
    .await?;

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
