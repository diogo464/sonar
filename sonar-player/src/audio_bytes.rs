use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncSeek};

pub struct BytesAudioSource {
    buffer: Bytes,
    position: usize,
}

impl From<Bytes> for BytesAudioSource {
    fn from(value: Bytes) -> Self {
        Self::new(value)
    }
}

impl BytesAudioSource {
    pub fn new(buffer: Bytes) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }
}

impl AsyncRead for BytesAudioSource {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let remaining = this.buffer.len() - this.position;
        if remaining == 0 {
            return std::task::Poll::Ready(Ok(()));
        }
        let to_read = std::cmp::min(remaining, buf.remaining());
        buf.put_slice(&this.buffer[this.position..this.position + to_read]);
        this.position += to_read;
        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncSeek for BytesAudioSource {
    fn start_seek(
        self: std::pin::Pin<&mut Self>,
        position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        let this = self.get_mut();
        match position {
            std::io::SeekFrom::Start(offset) => {
                this.position = (offset as usize).min(this.buffer.len())
            }
            std::io::SeekFrom::End(offset) => {
                this.position = (this.buffer.len() as i64 + offset)
                    .max(0)
                    .min(this.buffer.len() as i64) as usize
            }
            std::io::SeekFrom::Current(offset) => {
                this.position = (this.position as i64 + offset)
                    .max(0)
                    .min(this.buffer.len() as i64) as usize
            }
        }
        Ok(())
    }

    fn poll_complete(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        std::task::Poll::Ready(Ok(self.position as u64))
    }
}
