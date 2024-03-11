use tokio::io::{AsyncRead, AsyncSeek};

use super::AudioSource;

pub struct FileAudioSource {
    file: tokio::fs::File,
    size: u64,
}

impl FileAudioSource {
    pub async fn new(file: tokio::fs::File) -> std::io::Result<Self> {
        let metadata = file.metadata().await?;
        Ok(Self {
            file,
            size: metadata.len(),
        })
    }
}

impl AsyncRead for FileAudioSource {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncRead::poll_read(std::pin::Pin::new(&mut self.get_mut().file), cx, buf)
    }
}

impl AsyncSeek for FileAudioSource {
    fn start_seek(
        self: std::pin::Pin<&mut Self>,
        position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        AsyncSeek::start_seek(std::pin::Pin::new(&mut self.get_mut().file), position)
    }

    fn poll_complete(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        AsyncSeek::poll_complete(std::pin::Pin::new(&mut self.get_mut().file), cx)
    }
}

impl AudioSource for FileAudioSource {
    fn size(&self) -> u64 {
        self.size
    }
}
