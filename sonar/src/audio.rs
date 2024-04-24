use std::time::Duration;

use lofty::prelude::AudioFile;
use sqlx::Row;

use crate::{
    blob::{self, BlobStorage},
    bytestream::{self, ByteStream},
    db::{self, DbC},
    ks, AudioId, ByteRange, Error, ErrorKind, Result, TrackId,
};

#[derive(Debug, Clone)]
pub struct Audio {
    pub id: AudioId,
    pub bitrate: u32,
    pub duration: Duration,
    pub num_channels: u32,
    pub sample_freq: u32,
    pub size: u32,
    pub mime_type: String,
}

pub struct AudioCreate {
    pub stream: ByteStream,
    pub filename: Option<String>,
}

impl std::fmt::Debug for AudioCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioCreate").finish()
    }
}

#[derive(Debug, Clone)]
pub struct AudioStat {
    pub id: AudioId,
    pub size: u32,
}

pub struct AudioDownload {
    pub mime_type: String,
    pub stream: ByteStream,
    pub audio: Audio,
}

impl std::fmt::Debug for AudioDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDownload")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AudioView {
    id: i64,
    bitrate: i64,
    duration_ms: i64,
    num_channels: i64,
    sample_freq: i64,
    mime_type: String,
    #[allow(unused)]
    filename: Option<String>,
    #[allow(unused)]
    blob_key: String,
    blob_size: i64,
}

impl From<AudioView> for Audio {
    fn from(value: AudioView) -> Self {
        Self {
            id: AudioId::from_db(value.id),
            bitrate: value.bitrate as u32,
            duration: Duration::from_millis(value.duration_ms as u64),
            num_channels: value.num_channels as u32,
            sample_freq: value.sample_freq as u32,
            size: value.blob_size as u32,
            mime_type: value.mime_type,
        }
    }
}

pub async fn list_by_track(db: &mut DbC, track_id: TrackId) -> Result<Vec<Audio>> {
    let rows = sqlx::query_as::<_, AudioView>(
        "SELECT a.* FROM sqlx_audio a INNER JOIN track_audio ta ON a.id = ta.audio WHERE ta.track = ?"
    )
    .bind(track_id)
    .fetch_all(&mut *db)
    .await?;
    Ok(rows.into_iter().map(Audio::from).collect())
}

pub async fn create(db: &mut DbC, storage: &dyn BlobStorage, create: AudioCreate) -> Result<Audio> {
    let temp_dir = tempfile::tempdir()?;
    let temp_file_path = temp_dir.path().join("audio");
    bytestream::to_file(create.stream, &temp_file_path).await?;

    let file_type = infer::get_from_path(&temp_file_path)?.ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "failed to determine file type from audio file",
        )
    })?;
    let temp_file_path = {
        let new_path = temp_file_path.with_extension(file_type.extension());
        std::fs::rename(temp_file_path, &new_path)?;
        new_path
    };

    let filesize = temp_file_path.metadata()?.len() as u32;
    let tagged_file = lofty::read_from_path(&temp_file_path).map_err(Error::wrap)?;
    let blob_key = blob::random_key_with_prefix("audio");
    let blob_sha256 = ks::sha256_file(&temp_file_path).await?;
    let properties = tagged_file.properties();
    eprintln!("properties: {:#?}", properties);

    let filename = create.filename;
    let bitrate = properties
        .audio_bitrate()
        .ok_or_else(|| Error::new(ErrorKind::Invalid, "audio file does not have a bitrate"))?;
    let duration_ms = properties.duration().as_millis() as u32;
    let num_channels = properties.channels().ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "audio file does not have a channel count",
        )
    })? as u32;
    let sample_freq = properties
        .sample_rate()
        .ok_or_else(|| Error::new(ErrorKind::Invalid, "audio file does not have a sample rate"))?
        as u32;
    let mime_type = file_type.mime_type();

    let stream = bytestream::from_file(&temp_file_path).await?;
    storage.write(&blob_key, stream).await?;

    let blob_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO blob (key, size, sha256) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(blob_key)
    .bind(filesize)
    .bind(blob_sha256)
    .fetch_one(&mut *db)
    .await?;

    let audio_id = sqlx::query_scalar(
        "INSERT INTO audio (bitrate, duration_ms, num_channels, sample_freq, mime_type, blob, filename) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id")
    .bind(bitrate)
    .bind(duration_ms)
    .bind(num_channels)
    .bind(sample_freq)
    .bind(mime_type)
    .bind(blob_id)
    .bind(filename)
    .fetch_one(&mut *db)
    .await?;

    get(db, AudioId::from_db(audio_id)).await
}

pub async fn get(db: &mut DbC, audio_id: AudioId) -> Result<Audio> {
    let view = db::get_by_id::<AudioView, _>(db, "sqlx_audio", audio_id).await?;
    Ok(Audio::from(view))
}

pub async fn get_bulk(db: &mut DbC, audio_ids: &[AudioId]) -> Result<Vec<Audio>> {
    let views = db::list_bulk::<AudioView, _>(db, "sqlx_audio", audio_ids).await?;
    Ok(views.into_iter().map(Audio::from).collect())
}

pub async fn delete(db: &mut DbC, audio_id: AudioId) -> Result<()> {
    sqlx::query("DELETE FROM audio WHERE id = ?")
        .bind(audio_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn download(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    audio_id: AudioId,
    range: ByteRange,
) -> Result<AudioDownload> {
    let audio = get(&mut *db, audio_id).await?;
    let row = sqlx::query("SELECT mime_type, blob_key FROM sqlx_audio WHERE id = ?")
        .bind(audio_id)
        .fetch_one(&mut *db)
        .await?;
    let blob_key = row.get::<String, _>(1);
    let stream = storage.read(&blob_key, range).await?;
    Ok(AudioDownload {
        mime_type: row.get(0),
        stream,
        audio,
    })
}

pub async fn stat(db: &mut DbC, audio_id: AudioId) -> Result<AudioStat> {
    let row = sqlx::query("SELECT id, blob_size FROM sqlx_audio WHERE id = ?")
        .bind(audio_id)
        .fetch_one(&mut *db)
        .await?;
    let size: i64 = row.get(1);
    Ok(AudioStat {
        id: audio_id,
        size: size as u32,
    })
}

pub async fn link(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO track_audio (track, audio) VALUES (?, ?)")
        .bind(track_id)
        .bind(audio_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn unlink(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    sqlx::query("DELETE FROM track_audio WHERE track = ? AND audio = ?")
        .bind(track_id)
        .bind(audio_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}

pub async fn set_preferred(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    link(db, audio_id, track_id).await?;
    sqlx::query("UPDATE track_audio SET preferred = NULL WHERE track = ?")
        .bind(track_id)
        .execute(&mut *db)
        .await?;
    sqlx::query("UPDATE track_audio SET preferred = TRUE WHERE track = ? AND audio = ?")
        .bind(track_id)
        .bind(audio_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
