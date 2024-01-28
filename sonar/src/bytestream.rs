use std::path::Path;

use bytes::Bytes;
use tokio::fs::File;
use tokio_stream::StreamExt;

pub type ByteStream = Box<dyn tokio_stream::Stream<Item = std::io::Result<Bytes>> + Send + Unpin>;

pub async fn to_file(stream: ByteStream, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path).await?;
    let mut reader = tokio_util::io::StreamReader::new(stream);
    tokio::io::copy(&mut reader, &mut file).await?;
    Ok(())
}

pub async fn from_file(path: &Path) -> std::io::Result<ByteStream> {
    let file = File::open(path).await?;
    Ok(Box::new(tokio_util::io::ReaderStream::new(
        tokio::io::BufReader::new(file),
    )))
}

pub async fn to_bytes<S>(mut stream: S) -> std::io::Result<bytes::Bytes>
where
    S: tokio_stream::Stream<Item = std::io::Result<bytes::Bytes>> + Unpin,
{
    let mut bytes = bytes::BytesMut::new();
    while let Some(buf) = stream.next().await {
        bytes.extend_from_slice(&buf?);
    }
    Ok(bytes.freeze())
}

pub fn from_bytes(bytes: impl Into<Bytes>) -> ByteStream {
    Box::new(tokio_stream::once(Ok(bytes.into())))
}
