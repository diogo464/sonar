use std::time::Duration;

use lofty::AudioFile;

use crate::{
    blob::{self, BlobStorage},
    bytestream::{self, ByteStream},
    db::DbC,
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

pub struct AudioDownload {
    pub mime_type: String,
    pub stream: ByteStream,
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

    let blob_id = sqlx::query_scalar!(
        "INSERT INTO blob (key, size, sha256) VALUES (?, ?, ?) RETURNING id",
        blob_key,
        filesize,
        blob_sha256
    )
    .fetch_one(&mut *db)
    .await?;

    let audio_id = sqlx::query!(
        "INSERT INTO audio (bitrate, duration_ms, num_channels, sample_freq, mime_type, blob, filename) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        bitrate,
        duration_ms,
        num_channels,
        sample_freq,
        mime_type,
        blob_id,
        filename,
    )
    .fetch_one(&mut *db)
    .await?;

    get(db, AudioId::from_db(audio_id.id)).await
}

pub async fn get(db: &mut DbC, audio_id: AudioId) -> Result<Audio> {
    let row = sqlx::query_as!(AudioView, "SELECT * FROM sqlx_audio WHERE id = ?", audio_id)
        .fetch_one(&mut *db)
        .await?;

    Ok(Audio {
        id: AudioId::from_db(row.id),
        bitrate: row.bitrate as u32,
        duration: Duration::from_millis(row.duration_ms as u64),
        num_channels: row.num_channels as u32,
        sample_freq: row.sample_freq as u32,
        size: row.blob_size as u32,
        mime_type: row.mime_type,
    })
}

pub async fn delete(db: &mut DbC, audio_id: AudioId) -> Result<()> {
    let audio_id = audio_id.to_db();
    sqlx::query!("DELETE FROM audio WHERE id = ?", audio_id)
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
    let row = sqlx::query!(
        "SELECT mime_type, blob_key FROM sqlx_audio WHERE id = ?",
        audio_id
    )
    .fetch_one(&mut *db)
    .await?;
    let stream = storage.read(&row.blob_key, From::from(range)).await?;
    Ok(AudioDownload {
        mime_type: row.mime_type,
        stream,
    })
}

pub async fn link(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    sqlx::query!(
        "INSERT OR IGNORE INTO track_audio (track, audio) VALUES (?, ?)",
        track_id,
        audio_id
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

pub async fn unlink(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    sqlx::query!(
        "DELETE FROM track_audio WHERE track = ? AND audio = ?",
        track_id,
        audio_id
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

pub async fn set_preferred(db: &mut DbC, audio_id: AudioId, track_id: TrackId) -> Result<()> {
    link(db, audio_id, track_id).await?;
    sqlx::query!(
        "UPDATE track_audio SET preferred = NULL WHERE track = ?",
        track_id
    )
    .execute(&mut *db)
    .await?;
    sqlx::query!(
        "UPDATE track_audio SET preferred = TRUE WHERE track = ? AND audio = ?",
        track_id,
        audio_id
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}
