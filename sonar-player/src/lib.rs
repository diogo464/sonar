#![feature(duration_constants)]

use bytes::Bytes;
use sonar::{AudioId, Error, ErrorKind, Result, TrackId};
use tokio::io::{AsyncRead, AsyncSeek};

mod audio_stream;
pub use audio_stream::AudioStream;

mod audio_file;
pub use audio_file::FileAudioSource;

mod audio_bytes;
pub use audio_bytes::BytesAudioSource;

mod audio_player;
pub use audio_player::{AudioPlayer, AudioPlayerEvent};

mod audio_client_grpc;
pub use audio_client_grpc::AudioClientGrpc;

#[async_trait::async_trait]
pub trait AudioClient: Send + Sync + 'static {
    async fn stat(&self, track: TrackId) -> Result<u64>;
    async fn chunk(&self, track: TrackId, offset: u64, length: u64) -> Result<AudioChunk>;
    async fn download(&self, track: TrackId) -> Result<Audio>;
}

pub trait AudioSource: Unpin + AsyncRead + AsyncSeek + Send + Sync + 'static {
    /// the total size of this audio source.
    fn size(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct Audio {
    pub data: Bytes,
}

#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub data: Bytes,
    pub offset: u64,
}
