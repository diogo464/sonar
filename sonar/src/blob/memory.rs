use std::{collections::HashMap, io::Result, sync::Mutex};

use bytes::{Bytes, BytesMut};
use tokio_stream::StreamExt;

use crate::ByteStream;

use super::BlobStorage;

#[derive(Default)]
pub struct MemoryBlobStorage {
    blobs: Mutex<HashMap<String, Bytes>>,
}

impl std::fmt::Debug for MemoryBlobStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryBlobStorage").finish()
    }
}

#[async_trait::async_trait]
impl BlobStorage for MemoryBlobStorage {
    async fn get(&self, key: &str) -> Result<Bytes> {
        let blobs = self.blobs.lock().unwrap();
        match blobs.get(key) {
            Some(bytes) => Ok(bytes.clone()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "blob not found",
            )),
        }
    }
    async fn read(&self, key: &str) -> Result<ByteStream> {
        let data = self.get(key).await?;
        Ok(Box::new(tokio_stream::once(Ok(data))))
    }
    async fn put(&self, key: &str, bytes: Bytes) -> Result<()> {
        let mut blobs = self.blobs.lock().unwrap();
        blobs.insert(key.to_owned(), bytes);
        Ok(())
    }
    async fn write(&self, key: &str, mut reader: ByteStream) -> Result<()> {
        let mut bytes = BytesMut::new();
        while let Some(buf) = reader.next().await {
            bytes.extend_from_slice(&buf?);
        }
        let mut blobs = self.blobs.lock().unwrap();
        blobs.insert(key.to_owned(), bytes.freeze());
        Ok(())
    }
    async fn delete(&self, key: &str) -> Result<()> {
        let mut blobs = self.blobs.lock().unwrap();
        blobs.remove(key);
        Ok(())
    }
}
