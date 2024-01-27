use std::time::Duration;

use crate::{
    blob::{self, BlobStorage},
    genre, property, AlbumId, ArtistId, DbC, Error, ErrorKind, Genres, ImageId, ListParams, Lyrics,
    LyricsKind, LyricsLine, Properties, Result, Track, TrackCreate, TrackId, TrackLyrics,
    TrackUpdate, ValueUpdate,
};

#[derive(sqlx::FromRow)]
struct TrackView {
    id: i64,
    name: String,
    artist: i64,
    album: i64,
    disc_number: i64,
    track_number: i64,
    duration_ms: i64,
    listen_count: i64,
    cover_art: Option<i64>,
}

impl TrackView {
    fn into_track(self, genres: Genres, properties: Properties) -> Track {
        Track {
            id: TrackId::from_db(self.id),
            name: self.name,
            artist: ArtistId::from_db(self.artist),
            album: AlbumId::from_db(self.album),
            disc_number: self.disc_number as u32,
            track_number: self.track_number as u32,
            duration: Duration::from_millis(self.duration_ms as u64),
            listen_count: self.listen_count as u32,
            cover_art: self.cover_art.map(ImageId::from_db),
            genres,
            properties,
        }
    }
}

pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Track>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        TrackView,
        "SELECT * FROM track_view ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;

    let mut tracks = Vec::with_capacity(views.len());
    for view in views {
        let genres = crate::genre::get(&mut *db, crate::genre::Namespace::Track, view.id).await?;
        let properties =
            crate::property::get(&mut *db, crate::property::Namespace::Track, view.id).await?;
        tracks.push(view.into_track(genres, properties));
    }

    Ok(tracks)
}

pub async fn get(db: &mut DbC, track_id: TrackId) -> Result<Track> {
    let track_id = track_id.to_db();
    let track_view = sqlx::query_as!(TrackView, "SELECT * FROM track_view WHERE id = ?", track_id)
        .fetch_one(&mut *db)
        .await?;

    let genres = genre::get(&mut *db, crate::genre::Namespace::Track, track_id).await?;
    let properties = property::get(&mut *db, crate::property::Namespace::Track, track_id).await?;
    Ok(track_view.into_track(genres, properties))
}

pub async fn create(db: &mut DbC, storage: &dyn BlobStorage, create: TrackCreate) -> Result<Track> {
    let blob_key = blob::random_key_with_prefix("audio");
    storage.write(&blob_key, create.audio_stream).await?;

    let album_id = create.album.to_db();
    let cover_art = create.cover_art.map(|id| id.to_db());
    let track_id = sqlx::query!(
        "INSERT INTO track (name, album, cover_art, audio_blob_key, audio_filename)
        VALUES (?, ?, ?, ?, ?) RETURNING id",
        create.name,
        album_id,
        cover_art,
        blob_key,
        create.audio_filename,
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    genre::set(&mut *db, genre::Namespace::Track, track_id, &create.genres).await?;
    property::set(
        &mut *db,
        property::Namespace::Track,
        track_id,
        &create.properties,
    )
    .await?;

    let track_id = TrackId::from_db(track_id);
    if let Some(lyrics) = create.lyrics {
        set_lyrics(db, track_id, lyrics).await?;
    }

    get(db, track_id).await
}

pub async fn update(db: &mut DbC, track_id: TrackId, update: TrackUpdate) -> Result<Track> {
    let db_id = track_id.to_db();
    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!("UPDATE track SET name = ? WHERE id = ?", new_name, db_id)
            .execute(&mut *db)
            .await?;
    }

    match update.album {
        ValueUpdate::Set(album_id) => {
            let album_id = album_id.to_db();
            sqlx::query!("UPDATE track SET album = ? WHERE id = ?", album_id, db_id)
                .execute(&mut *db)
                .await?;
        }
        ValueUpdate::Unset => {
            return Err(Error::new(
                ErrorKind::Invalid,
                "cannot unset album on track update",
            ));
        }
        ValueUpdate::Unchanged => {}
    }

    match update.cover_art {
        ValueUpdate::Set(cover_art_id) => {
            let cover_art_id = cover_art_id.to_db();
            sqlx::query!(
                "UPDATE track SET cover_art = ? WHERE id = ?",
                cover_art_id,
                db_id
            )
            .execute(&mut *db)
            .await?;
        }
        ValueUpdate::Unset => {
            sqlx::query!("UPDATE track SET cover_art = NULL WHERE id = ?", db_id)
                .execute(&mut *db)
                .await?;
        }
        ValueUpdate::Unchanged => {}
    }

    match update.lyrics {
        ValueUpdate::Set(lyrics) => set_lyrics(db, track_id, lyrics).await?,
        ValueUpdate::Unset => clear_lyrics(db, track_id).await?,
        ValueUpdate::Unchanged => {}
    }

    genre::update(&mut *db, genre::Namespace::Track, db_id, &update.genres).await?;
    property::update(
        &mut *db,
        property::Namespace::Track,
        db_id,
        &update.properties,
    )
    .await?;

    get(db, track_id).await
}

pub async fn delete(db: &mut DbC, track_id: TrackId) -> Result<()> {
    let track_id = track_id.to_db();
    sqlx::query!("DELETE FROM track WHERE id = ?", track_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn get_lyrics(db: &mut DbC, track_id: TrackId) -> Result<Lyrics> {
    let db_id = track_id.to_db();
    let lyrics_kind = sqlx::query_scalar!("SELECT lyrics_kind FROM track WHERE id = ?", db_id)
        .fetch_one(&mut *db)
        .await?;

    let lyrics_kind = match lyrics_kind.as_deref() {
        Some("S") => LyricsKind::Synced,
        Some("U") => LyricsKind::Unsynced,
        Some(_) => {
            return Err(Error::new(
                ErrorKind::Internal,
                "invalid lyrics kind in database",
            ))
        }
        None => return Err(Error::new(ErrorKind::NotFound, "no lyrics for track")),
    };

    let line_rows = sqlx::query!(
        "SELECT offset, text FROM track_lyrics_line WHERE track = ? ORDER BY offset ASC",
        db_id
    )
    .fetch_all(&mut *db)
    .await?;
    let mut lines = Vec::with_capacity(line_rows.len());

    for row in line_rows {
        lines.push(LyricsLine {
            offset: Duration::from_secs(row.offset as u64),
            text: row.text,
        });
    }

    Ok(Lyrics {
        track: track_id,
        kind: lyrics_kind,
        lines,
    })
}

async fn set_lyrics(db: &mut DbC, track_id: TrackId, lyrics: TrackLyrics) -> Result<()> {
    let db_id = track_id.to_db();
    let kind = match lyrics.kind {
        LyricsKind::Synced => "S",
        LyricsKind::Unsynced => "U",
    };

    clear_lyrics(db, track_id).await?;

    sqlx::query!("UPDATE track SET lyrics_kind = ? WHERE id = ?", kind, db_id)
        .execute(&mut *db)
        .await?;

    for line in lyrics.lines {
        let offset = line.offset.as_secs() as i64;
        sqlx::query!(
            "INSERT INTO track_lyrics_line (track, offset, text) VALUES (?, ?, ?)",
            db_id,
            offset,
            line.text,
        )
        .execute(&mut *db)
        .await?;
    }

    Ok(())
}

async fn clear_lyrics(db: &mut DbC, track_id: TrackId) -> Result<()> {
    let db_id = track_id.to_db();
    sqlx::query!("UPDATE track SET lyrics_kind = NULL WHERE id = ?", db_id)
        .execute(&mut *db)
        .await?;

    sqlx::query!("DELETE FROM track_lyrics_line WHERE track = ?", db_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
