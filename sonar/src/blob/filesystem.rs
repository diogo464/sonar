use std::{io::Result, path::PathBuf};

use bytes::Bytes;

use crate::{async_trait, bytestream, ByteStream};

use super::BlobStorage;

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
    async fn get(&self, key: &str) -> Result<Bytes> {
        let reader = self.read(key).await?;
        bytestream::to_bytes(reader).await
    }
    async fn read(&self, key: &str) -> Result<ByteStream> {
        let path = self.root.join(key);
        bytestream::from_file(&path).await
    }
    async fn put(&self, key: &str, bytes: Bytes) -> Result<()> {
        self.write(key, bytestream::from_bytes(bytes))
            .await
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
