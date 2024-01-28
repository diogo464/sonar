use std::{collections::HashMap, io::Result, sync::Mutex};

use bytes::{Bytes, BytesMut};
use tokio_stream::StreamExt;

use crate::{bytestream, ByteStream};

use super::{BlobRange, BlobStorage};

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
    async fn get(&self, key: &str, range: BlobRange) -> Result<Bytes> {
        let blobs = self.blobs.lock().unwrap();
        match blobs.get(key) {
            Some(bytes) => {
                let start = range.start.unwrap_or(0);
                let length = range
                    .length
                    .unwrap_or(bytes.len() as u64)
                    .min(bytes.len() as u64);
                Ok(bytes.slice(start as usize..(start + length) as usize))
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "blob not found",
            )),
        }
    }
    async fn read(&self, key: &str, range: BlobRange) -> Result<ByteStream> {
        let data = self.get(key, range).await?;
        Ok(bytestream::from_bytes(data))
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
