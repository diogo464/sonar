use std::time::Duration;

use sqlx::Row;

use crate::{
    audio::{self, AudioDownload},
    blob::BlobStorage,
    db::{self, Db, DbC},
    property, AlbumId, ArtistId, AudioId, ByteRange, Error, ErrorKind, ImageId, ListParams,
    Properties, PropertyUpdate, Result, Timestamp, TrackId, ValueUpdate,
};

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub artist: ArtistId,
    pub album: AlbumId,
    pub duration: Duration,
    pub listen_count: u32,
    pub audio: Option<AudioId>,
    pub cover_art: Option<ImageId>,
    pub properties: Properties,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackLyrics {
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

pub struct TrackCreate {
    pub name: String,
    pub album: AlbumId,
    pub cover_art: Option<ImageId>,
    pub lyrics: Option<TrackLyrics>,
    pub audio: Option<AudioId>,
    pub properties: Properties,
}

impl std::fmt::Debug for TrackCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackCreate")
            .field("name", &self.name)
            .field("album", &self.album)
            .field("cover_art", &self.cover_art)
            .field("lyrics", &self.lyrics)
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TrackUpdate {
    pub name: ValueUpdate<String>,
    pub album: ValueUpdate<AlbumId>,
    pub cover_art: ValueUpdate<ImageId>,
    pub lyrics: ValueUpdate<TrackLyrics>,
    pub properties: Vec<PropertyUpdate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LyricsKind {
    Synced,
    Unsynced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    duration_ms: Option<i64>,
    audio: Option<i64>,
    listen_count: Option<i64>,
    cover_art: Option<i64>,
    created_at: i64,
}

impl From<(TrackView, Properties)> for Track {
    fn from((value, properties): (TrackView, Properties)) -> Self {
        Self {
            id: TrackId::from_db(value.id),
            name: value.name,
            artist: ArtistId::from_db(value.artist),
            album: AlbumId::from_db(value.album),
            duration: Duration::from_millis(value.duration_ms.unwrap_or_default() as u64),
            audio: value.audio.map(AudioId::from_db),
            listen_count: value.listen_count.unwrap_or_default() as u32,
            cover_art: value.cover_art.map(ImageId::from_db),
            properties,
            created_at: Timestamp::from_seconds(value.created_at as u64),
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<Track>> {
    let views = db::list::<TrackView>(db, "sqlx_track", params).await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| TrackId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Track::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_by_album(
    db: &mut DbC,
    album_id: AlbumId,
    params: ListParams,
) -> Result<Vec<Track>> {
    let views =
        db::list_where_field_eq::<TrackView, _>(db, "sqlx_track", "album", album_id, params)
            .await?;
    let properties =
        property::get_bulk(db, views.iter().map(|view| TrackId::from_db(view.id))).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Track::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn list_album_id_pairs(db: &mut DbC) -> Result<Vec<(AlbumId, TrackId)>> {
    let rows = sqlx::query("SELECT album, id FROM track")
        .fetch_all(db)
        .await?;
    let mut pairs = Vec::with_capacity(rows.len());
    for row in rows {
        let album_id = AlbumId::from_db(row.get(0));
        let track_id = TrackId::from_db(row.get(1));
        pairs.push((album_id, track_id));
    }
    Ok(pairs)
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, track_id: TrackId) -> Result<Track> {
    let track_view = sqlx::query_as::<_, TrackView>("SELECT * FROM sqlx_track WHERE id = ?")
        .bind(track_id)
        .fetch_one(&mut *db)
        .await?;
    let properties = property::get(db, track_id).await?;
    Ok(Track::from((track_view, properties)))
}

#[tracing::instrument(skip(db))]
pub async fn get_bulk(db: &mut DbC, track_ids: &[TrackId]) -> Result<Vec<Track>> {
    let views = db::list_bulk::<TrackView, _>(db, "sqlx_track", track_ids).await?;
    let properties = property::get_bulk(db, track_ids.iter().copied()).await?;
    Ok(views
        .into_iter()
        .zip(properties.into_iter())
        .map(Track::from)
        .collect())
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: TrackCreate) -> Result<Track> {
    let cover_art = create.cover_art.map(|id| id.to_db());
    let track_id = sqlx::query_scalar(
        "INSERT INTO track (name, album, cover_art)
        VALUES (?, ?, ?) RETURNING id",
    )
    .bind(create.name)
    .bind(create.album)
    .bind(cover_art)
    .fetch_one(&mut *db)
    .await?;

    let track_id = TrackId::from_db(track_id);
    property::set(db, track_id, &create.properties).await?;
    if let Some(audio_id) = create.audio {
        audio::set_preferred(db, audio_id, track_id).await?;
    }

    if let Some(lyrics) = create.lyrics {
        set_lyrics(db, track_id, lyrics).await?;
    }

    get(db, track_id).await
}

#[tracing::instrument(skip(db))]
pub async fn update(db: &mut DbC, track_id: TrackId, update: TrackUpdate) -> Result<Track> {
    tracing::info!("updating track {} with {:#?}", track_id, update);

    db::value_update_string_non_null(db, "track", "name", track_id, update.name).await?;
    db::value_update_id_non_null(db, "track", "album", track_id, update.album).await?;
    db::value_update_id_nullable(db, "track", "cover_art", track_id, update.cover_art).await?;
    match update.lyrics {
        ValueUpdate::Set(lyrics) => set_lyrics(db, track_id, lyrics).await?,
        ValueUpdate::Unset => clear_lyrics(db, track_id).await?,
        ValueUpdate::Unchanged => {}
    }
    property::update(db, track_id, &update.properties).await?;

    get(db, track_id).await
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, track_id: TrackId) -> Result<()> {
    sqlx::query("DELETE FROM track WHERE id = ?")
        .bind(track_id)
        .execute(&mut *db)
        .await?;
    property::clear(db, track_id).await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name(db: &mut DbC, create_: TrackCreate) -> Result<Track> {
    let track_id = sqlx::query_scalar("SELECT id FROM track WHERE name = ? AND album = ?")
        .bind(&create_.name)
        .bind(create_.album)
        .fetch_optional(&mut *db)
        .await?;
    if let Some(track_id) = track_id {
        return get(db, TrackId::from_db(track_id)).await;
    }
    create(db, create_).await
}

#[tracing::instrument(skip(db))]
pub async fn find_or_create_by_name_tx(db: &Db, create_: TrackCreate) -> Result<Track> {
    let mut tx = db.begin().await?;
    let result = find_or_create_by_name(&mut tx, create_).await;
    if result.is_ok() {
        tx.commit().await?;
    }
    result
}

#[tracing::instrument(skip(db))]
pub async fn download(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    track_id: TrackId,
    range: ByteRange,
) -> Result<AudioDownload> {
    let audio_id = sqlx::query_scalar("SELECT audio FROM sqlx_track WHERE id = ?")
        .bind(track_id)
        .fetch_one(&mut *db)
        .await?;
    if let Some(audio_id) = audio_id {
        let audio_id = AudioId::from_db(audio_id);
        audio::download(db, storage, audio_id, range).await
    } else {
        Err(Error::new(ErrorKind::NotFound, "no audio for track"))
    }
}

#[tracing::instrument(skip(db))]
pub async fn get_lyrics(db: &mut DbC, track_id: TrackId) -> Result<Lyrics> {
    let lyrics_kind =
        sqlx::query_scalar::<_, Option<String>>("SELECT lyrics_kind FROM track WHERE id = ?")
            .bind(track_id)
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

    let line_rows = sqlx::query(
        "SELECT offset, text FROM track_lyrics_line WHERE track = ? ORDER BY offset ASC",
    )
    .bind(track_id)
    .fetch_all(&mut *db)
    .await?;
    let mut lines = Vec::with_capacity(line_rows.len());

    for row in line_rows {
        lines.push(LyricsLine {
            offset: Duration::from_secs(row.get::<i64, _>(0) as u64),
            text: row.get(1),
        });
    }

    Ok(Lyrics {
        track: track_id,
        kind: lyrics_kind,
        lines,
    })
}

#[tracing::instrument(skip(db))]
async fn set_lyrics(db: &mut DbC, track_id: TrackId, lyrics: TrackLyrics) -> Result<()> {
    let kind = match lyrics.kind {
        LyricsKind::Synced => "S",
        LyricsKind::Unsynced => "U",
    };

    clear_lyrics(db, track_id).await?;

    sqlx::query("UPDATE track SET lyrics_kind = ? WHERE id = ?")
        .bind(kind)
        .bind(track_id)
        .execute(&mut *db)
        .await?;

    for line in lyrics.lines {
        let offset = line.offset.as_secs() as i64;
        sqlx::query("INSERT INTO track_lyrics_line (track, offset, text) VALUES (?, ?, ?)")
            .bind(track_id)
            .bind(offset)
            .bind(line.text)
            .execute(&mut *db)
            .await?;
    }

    Ok(())
}

#[tracing::instrument(skip(db))]
async fn clear_lyrics(db: &mut DbC, track_id: TrackId) -> Result<()> {
    sqlx::query("UPDATE track SET lyrics_kind = NULL WHERE id = ?")
        .bind(track_id)
        .execute(&mut *db)
        .await?;
    sqlx::query("DELETE FROM track_lyrics_line WHERE track = ?")
        .bind(track_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
