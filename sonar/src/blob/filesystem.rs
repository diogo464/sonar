use std::{
    io::{Result, SeekFrom},
    path::PathBuf,
};

use bytes::Bytes;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{async_trait, bytestream, ByteStream};

use super::{BlobRange, BlobStorage};

#[derive(Debug)]
pub struct FilesystemBlobStorage {
    root: PathBuf,
}

impl FilesystemBlobStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[async_trait]
impl BlobStorage for FilesystemBlobStorage {
    async fn read(&self, key: &str, range: BlobRange) -> Result<ByteStream> {
        let path = self.root.join(key);
        let mut file = File::open(path).await?;
        file.seek(SeekFrom::Start(range.start.unwrap_or(0))).await?;
        let reader = tokio::io::BufReader::new(file).take(range.length.unwrap_or(u64::MAX));
        Ok(Box::new(tokio_util::io::ReaderStream::new(reader)))
    }
    async fn put(&self, key: &str, bytes: Bytes) -> Result<()> {
        self.write(key, bytestream::from_bytes(bytes)).await
    }
    async fn write(&self, key: &str, reader: ByteStream) -> Result<()> {
        let path = self.root.join(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        bytestream::to_file(reader, &path).await
    }
    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.root.join(key);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }
}
