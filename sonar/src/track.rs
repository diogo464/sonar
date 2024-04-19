use std::time::Duration;

use sqlx::{FromRow as _, Row};

use crate::{
    audio::{self, AudioDownload, AudioStat},
    blob::BlobStorage,
    db::{self, Db, DbC, SonarView},
    property, AlbumId, ArtistId, AudioId, ByteRange, Error, ErrorKind, Genre, ImageId, ListParams,
    Properties, PropertyUpdate, Result, SonarId, Timestamp, TrackId, ValueUpdate,
    ID_NAMESPACE_ARTIST,
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
    pub duration: Duration,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Lyrics {
    pub track: TrackId,
    pub kind: LyricsKind,
    pub lines: Vec<LyricsLine>,
}

#[derive(Debug, Clone)]
pub struct TrackListRandom {
    pub limit: Option<u32>,
    pub genre: Option<Genre>,
}

#[derive(Clone, sqlx::FromRow)]
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

impl SonarView for TrackView {
    fn sonar_id(&self) -> SonarId {
        TrackId::from_db(self.id).into()
    }
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
pub async fn list_ids(db: &mut DbC) -> Result<Vec<TrackId>> {
    let ids = sqlx::query_scalar("SELECT id FROM track")
        .fetch_all(&mut *db)
        .await?;
    Ok(ids.into_iter().map(TrackId::from_db).collect())
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
pub async fn list_top_from_artist(
    db: &mut DbC,
    artist_id: ArtistId,
    params: ListParams,
) -> Result<Vec<Track>> {
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("");
    query.push("SELECT * FROM sqlx_track");
    query.push(" WHERE artist = ");
    query.push_bind(artist_id);
    query.push(" ORDER BY listen_count");
    query.push(params.sql_display());
    tracks_from_views_query(db, query).await
}

#[tracing::instrument(skip(db))]
pub async fn list_random(db: &mut DbC, params: TrackListRandom) -> Result<Vec<Track>> {
    let mut query =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT sqlx_track.id FROM sqlx_track ");

    if let Some(genre) = params.genre {
        // TODO: add album genres to?
        query.push("JOIN genre ON (genre.namespace = ");
        query.push(ID_NAMESPACE_ARTIST);
        query.push(" AND genre.identifier = sqlx_track.artist)");
        query.push(" WHERE genre.genre = '");
        query.push(genre);
        query.push("' ");
    }

    query.push("ORDER BY RANDOM() ");
    if let Some(limit) = params.limit {
        query.push("LIMIT ");
        query.push(limit);
    }

    tracks_from_views_query(db, query).await
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
    let expanded = db::expand_views(views, track_ids);
    let properties = property::get_bulk(db, track_ids.iter().copied()).await?;
    Ok(db::merge_view_properties(expanded, properties))
}

#[tracing::instrument(skip(db))]
pub async fn get_by_name(db: &mut DbC, name: &str) -> Result<Track> {
    let ids = sqlx::query_scalar("SELECT id FROM track WHERE name = ?")
        .bind(name)
        .fetch_all(&mut *db)
        .await?;
    if ids.is_empty() {
        return Err(Error::new(ErrorKind::NotFound, "track not found"));
    } else if ids.len() > 1 {
        return Err(Error::new(ErrorKind::Invalid, "ambiguous track name"));
    }
    get(db, TrackId::from_db(ids[0])).await
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
pub async fn stat(db: &mut DbC, track_id: TrackId) -> Result<AudioStat> {
    let audio_id = sqlx::query_scalar("SELECT audio FROM sqlx_track WHERE id = ?")
        .bind(track_id)
        .fetch_one(&mut *db)
        .await?;
    if let Some(audio_id) = audio_id {
        let audio_id = AudioId::from_db(audio_id);
        audio::stat(db, audio_id).await
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
        "SELECT offset, duration, text FROM track_lyrics_line WHERE track = ? ORDER BY offset ASC",
    )
    .bind(track_id)
    .fetch_all(&mut *db)
    .await?;
    let mut lines = Vec::with_capacity(line_rows.len());

    for row in line_rows {
        lines.push(LyricsLine {
            offset: Duration::from_millis(row.get::<i64, _>(0) as u64),
            duration: Duration::from_millis(row.get::<i64, _>(1) as u64),
            text: row.get(2),
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
        sqlx::query(
            "INSERT INTO track_lyrics_line (track, offset, duration, text) VALUES (?, ?, ?, ?)",
        )
        .bind(track_id)
        .bind(line.offset.as_millis() as i64)
        .bind(line.duration.as_millis() as i64)
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

async fn tracks_from_views_query(
    db: &mut DbC,
    mut query: sqlx::QueryBuilder<'_, sqlx::Sqlite>,
) -> Result<Vec<Track>> {
    let rows = query.build().fetch_all(&mut *db).await?;
    let mut ids = Vec::with_capacity(rows.len());
    let mut views = Vec::with_capacity(rows.len());
    for row in rows {
        let view = TrackView::from_row(&row)?;
        let track_id = TrackId::from_db(view.id);
        ids.push(track_id);
        views.push(view);
    }
    let expanded = db::expand_views(views, &ids);
    let properties = property::get_bulk(db, ids.iter().copied()).await?;
    Ok(db::merge_view_properties(expanded, properties))
}
