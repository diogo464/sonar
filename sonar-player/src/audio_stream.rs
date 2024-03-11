use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncSeek};

use crate::{Result, TrackId};

use super::{AudioClient, AudioSource};

type ChunkIndex = usize;

const CHUNK_SIZE: usize = 128 * 1024;

struct Shared {
    /// the audio we are downloading
    audio: TrackId,
    /// total size of the audio file in bytes
    size: u64,
    /// the current read offset
    offset: AtomicU64,
    /// a flag indicating the download has failed
    failed: AtomicBool,
    /// a waker for the reader task
    waker: futures::task::AtomicWaker,
    /// a map of chunks that have been downloaded
    /// the downloader will insert chunks into this map
    /// and the reader will use them to read the audio file
    chunks: Mutex<HashMap<ChunkIndex, Bytes>>,
}

pub struct AudioStream {
    shared: Arc<Shared>,
    abort: tokio::task::AbortHandle,
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

impl AudioStream {
    pub async fn new<C: AudioClient>(client: C, audio: TrackId) -> Result<Self> {
        let size = client.stat(audio).await?;
        let shared = Arc::new(Shared {
            audio,
            size,
            offset: AtomicU64::new(0),
            failed: AtomicBool::new(false),
            waker: Default::default(),
            chunks: Mutex::new(HashMap::new()),
        });
        let handle = tokio::spawn(download_task(client, shared.clone()));
        Ok(Self {
            shared,
            abort: handle.abort_handle(),
        })
    }
}

impl AsyncRead for AudioStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let offset = self.shared.offset.load(Ordering::Relaxed);
        let remain = self.shared.size - offset;
        if remain == 0 {
            return std::task::Poll::Ready(Err(std::io::Error::from(
                std::io::ErrorKind::UnexpectedEof,
            )));
        }
        let chunk = offset / CHUNK_SIZE as u64;
        let buffer_offset = offset % CHUNK_SIZE as u64;
        let copy_n = buf
            .remaining()
            .min(CHUNK_SIZE - buffer_offset as usize)
            .min(remain as usize);
        let chunks = self.shared.chunks.lock().unwrap();
        match chunks.get(&(chunk as ChunkIndex)) {
            Some(buffer) => {
                tracing::trace!(
                    "reading {} bytes from chunk {} (offset {})",
                    copy_n,
                    chunk,
                    offset
                );
                buf.put_slice(&buffer[buffer_offset as usize..buffer_offset as usize + copy_n]);
                self.shared
                    .offset
                    .fetch_add(copy_n as u64, Ordering::Relaxed);
                std::task::Poll::Ready(Ok(()))
            }
            None => {
                if self.shared.failed.load(Ordering::Relaxed) {
                    std::task::Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "download failed",
                    )))
                } else {
                    self.shared.waker.register(cx.waker());
                    std::task::Poll::Pending
                }
            }
        }
    }
}

impl AsyncSeek for AudioStream {
    fn start_seek(
        self: std::pin::Pin<&mut Self>,
        position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        let position = match position {
            std::io::SeekFrom::Start(off) => off.min(self.shared.size),
            std::io::SeekFrom::End(off) => (self.shared.size as i64 - off - 1).max(0) as u64,
            std::io::SeekFrom::Current(off) => {
                ((self.shared.offset.load(Ordering::Relaxed) as i64 + off).max(0) as u64)
                    .min(self.shared.size)
            }
        };
        self.shared.offset.store(position, Ordering::Relaxed);
        Ok(())
    }

    fn poll_complete(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        std::task::Poll::Ready(Ok(self.shared.offset.load(Ordering::Relaxed)))
    }
}

impl AudioSource for AudioStream {
    fn size(&self) -> u64 {
        self.shared.size
    }
}

async fn download_task<C: AudioClient>(client: C, shared: Arc<Shared>) {
    tracing::info!(
        "starting download for track {} with size {}",
        shared.audio,
        shared.size
    );
    if let Err(err) = download_task_try(&client, &shared).await {
        tracing::error!("audio download task failed: {}", err);
        shared.failed.store(true, Ordering::Relaxed);
        shared.waker.wake();
    }
}

async fn download_task_try<C: AudioClient>(client: &C, shared: &Shared) -> Result<()> {
    let total_chunks = ((shared.size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64) as usize;
    let mut missing = HashSet::new();
    for i in 0..total_chunks {
        missing.insert(i as ChunkIndex);
    }

    while !missing.is_empty() {
        let offset = shared.offset.load(Ordering::Relaxed);
        let mut chunk = (offset / CHUNK_SIZE as u64) as usize;
        while !missing.contains(&chunk) {
            chunk = (chunk + 1) % total_chunks;
        }

        let audio_offset = chunk * CHUNK_SIZE;
        let audio_size = CHUNK_SIZE.min(shared.size.saturating_sub(audio_offset as u64) as usize);
        tracing::debug!(
            "downloading chunk {} for track {} at offset {} with size {}",
            chunk,
            shared.audio,
            chunk * CHUNK_SIZE,
            audio_size,
        );
        let mut audio_chunk = client
            .chunk(shared.audio, audio_offset as u64, audio_size as u64)
            .await?;
        if audio_chunk.data.len() != audio_size {
            tracing::warn!(
                "audio chunk {} for track {} at offset {} with size {} had invalid size ({}), resizing",
                chunk,
                shared.audio,
                chunk * CHUNK_SIZE,
                audio_size,
                audio_chunk.data.len(),
            );
            let mut new_buffer = BytesMut::new();
            new_buffer.extend_from_slice(&audio_chunk.data);
            new_buffer.resize(audio_size, 0);
            audio_chunk.data = new_buffer.freeze();
        }

        let mut chunks = shared.chunks.lock().unwrap();
        chunks.insert(chunk, audio_chunk.data);
        missing.remove(&chunk);
        shared.waker.wake();
    }

    tracing::info!("download for track {} completed", shared.audio);
    Ok(())
}
