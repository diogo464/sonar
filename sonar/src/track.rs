use std::time::Duration;

use crate::{
    blob::{self, BlobStorage},
    bytestream::ByteStream,
    db::DbC,
    AlbumId, ArtistId, ByteRange, Error, ErrorKind, ImageId, ListParams, Properties,
    PropertyUpdate, Result, Timestamp, TrackId, ValueUpdate,
};

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub artist: ArtistId,
    pub album: AlbumId,
    pub duration: Duration,
    pub listen_count: u32,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct TrackLyrics {
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

pub struct TrackCreate {
    pub name: String,
    pub album: AlbumId,
    pub duration: Duration,
    pub cover_art: Option<ImageId>,
    pub lyrics: Option<TrackLyrics>,
    pub properties: Properties,
    pub audio_stream: ByteStream,
    pub audio_filename: String,
}

impl std::fmt::Debug for TrackCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackCreate")
            .field("name", &self.name)
            .field("album", &self.album)
            .field("duration", &self.duration)
            .field("cover_art", &self.cover_art)
            .field("lyrics", &self.lyrics)
            .field("properties", &self.properties)
            .field("audio_filename", &self.audio_filename)
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TrackUpdate {
    pub name: ValueUpdate<String>,
    pub album: ValueUpdate<AlbumId>,
    pub disc_number: ValueUpdate<u32>,
    pub track_number: ValueUpdate<u32>,
    pub cover_art: ValueUpdate<ImageId>,
    pub lyrics: ValueUpdate<TrackLyrics>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LyricsKind {
    Synced,
    Unsynced,
}

#[derive(Debug, Clone)]
pub struct LyricsLine {
    pub offset: Duration,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Lyrics {
    pub track: TrackId,
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

#[derive(sqlx::FromRow)]
struct TrackView {
    id: i64,
    name: String,
    artist: i64,
    album: i64,
    duration_ms: i64,
    listen_count: i64,
    cover_art: Option<i64>,
    properties: Option<Vec<u8>>,
    created_at: i64,
}

impl TrackView {
    fn into_track(self) -> Track {
        Track {
            id: TrackId::from_db(self.id),
            name: self.name,
            artist: ArtistId::from_db(self.artist),
            album: AlbumId::from_db(self.album),
            duration: Duration::from_millis(self.duration_ms as u64),
            listen_count: self.listen_count as u32,
            cover_art: self.cover_art.map(ImageId::from_db),
            properties: Properties::deserialize_unchecked(&self.properties.unwrap_or_default()),
            created_at: Timestamp::from_seconds(self.created_at as u64),
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
    Ok(views.into_iter().map(TrackView::into_track).collect())
}

pub async fn list_by_album(
    db: &mut DbC,
    album_id: AlbumId,
    params: ListParams,
) -> Result<Vec<Track>> {
    let album_id = album_id.to_db();
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        TrackView,
        "SELECT * FROM track_view WHERE album = ? ORDER BY id ASC LIMIT ? OFFSET ?",
        album_id,
        limit,
        offset
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(views.into_iter().map(TrackView::into_track).collect())
}

pub async fn get(db: &mut DbC, track_id: TrackId) -> Result<Track> {
    let track_id = track_id.to_db();
    let track_view = sqlx::query_as!(TrackView, "SELECT * FROM track_view WHERE id = ?", track_id)
        .fetch_one(&mut *db)
        .await?;
    Ok(track_view.into_track())
}

pub async fn get_bulk(db: &mut DbC, track_ids: &[TrackId]) -> Result<Vec<Track>> {
    // NOTE: sqlite doesn't support binding arrays. the alternative would be to generate a
    // query string with all the ids or create a temporary table with those ids and then use a
    // select
    let mut tracks = Vec::with_capacity(track_ids.len());
    for track_id in track_ids {
        tracks.push(get(db, *track_id).await?);
    }
    Ok(tracks)
}

pub async fn create(db: &mut DbC, storage: &dyn BlobStorage, create: TrackCreate) -> Result<Track> {
    let blob_key = blob::random_key_with_prefix("audio");
    storage.write(&blob_key, create.audio_stream).await?;

    let album_id = create.album.to_db();
    let cover_art = create.cover_art.map(|id| id.to_db());
    let duration_ms = create.duration.as_millis() as i64;
    let properties = create.properties.serialize();
    let track_id = sqlx::query!(
        "INSERT INTO track (name, album, cover_art, duration_ms, audio_blob_key, audio_filename, properties)
        VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        create.name,
        album_id,
        cover_art,
        duration_ms,
        blob_key,
        create.audio_filename,
        properties
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    get(db, TrackId::from_db(track_id)).await
}

pub async fn update(db: &mut DbC, track_id: TrackId, update: TrackUpdate) -> Result<Track> {
    tracing::info!("updating track {} with {:?}", track_id, update);
    if let Some(new_name) = match update.name {
        ValueUpdate::Set(name) => Some(name),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::query!("UPDATE track SET name = ? WHERE id = ?", new_name, track_id)
            .execute(&mut *db)
            .await?;
    }

    match update.album {
        ValueUpdate::Set(album_id) => {
            let album_id = album_id.to_db();
            sqlx::query!(
                "UPDATE track SET album = ? WHERE id = ?",
                album_id,
                track_id
            )
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
                track_id
            )
            .execute(&mut *db)
            .await?;
        }
        ValueUpdate::Unset => {
            sqlx::query!("UPDATE track SET cover_art = NULL WHERE id = ?", track_id)
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

    if update.properties.len() > 0 {
        let properties = sqlx::query_scalar!("SELECT properties FROM track WHERE id = ?", track_id)
            .fetch_one(&mut *db)
            .await?
            .unwrap_or_default();
        let mut properties = Properties::deserialize_unchecked(&properties);
        properties.apply_updates(&update.properties);
        let properties = properties.serialize();
        sqlx::query!(
            "UPDATE track SET properties = ? WHERE id = ?",
            properties,
            track_id
        )
        .execute(&mut *db)
        .await?;
    }

    get(db, track_id).await
}

pub async fn delete(db: &mut DbC, track_id: TrackId) -> Result<()> {
    let track_id = track_id.to_db();
    sqlx::query!("DELETE FROM track WHERE id = ?", track_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn download(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    track_id: TrackId,
    range: ByteRange,
) -> Result<ByteStream> {
    let track_id = track_id.to_db();
    let blob_key = sqlx::query_scalar!("SELECT audio_blob_key FROM track WHERE id = ?", track_id)
        .fetch_one(&mut *db)
        .await?;
    Ok(storage
        .read(&blob_key, blob::BlobRange::from(range))
        .await?)
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
