use std::io::Result;

use crate::{async_trait, bytestream, ByteRange};
use bytes::Bytes;

mod memory;
pub use memory::MemoryBlobStorage;

mod filesystem;
pub use filesystem::FilesystemBlobStorage;

use crate::bytestream::ByteStream;

// TODO: remove this in favor of ByteRange
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobRange {
    pub start: Option<u64>,
    pub length: Option<u64>,
}

impl From<ByteRange> for BlobRange {
    fn from(value: ByteRange) -> Self {
        Self {
            start: value.offset,
            length: value.length,
        }
    }
}

impl BlobRange {
    pub fn new(start: u64, length: u64) -> Self {
        Self {
            start: Some(start),
            length: Some(length),
        }
    }

    pub fn with_start(mut self, start: u64) -> Self {
        self.start = Some(start);
        self
    }

    pub fn with_length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }
}

#[async_trait]
pub trait BlobStorage: std::fmt::Debug + Send + Sync + 'static {
    async fn get(&self, key: &str, range: BlobRange) -> Result<Bytes> {
        let stream = self.read(key, range).await?;
        bytestream::to_bytes(stream).await
    }
    async fn read(&self, key: &str, range: BlobRange) -> Result<ByteStream>;
    async fn put(&self, key: &str, bytes: Bytes) -> Result<()>;
    async fn write(&self, key: &str, reader: ByteStream) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub fn random_key() -> String {
    ulid::Ulid::new().to_string()
}

pub fn random_key_with_prefix(prefix: &str) -> String {
    format!("{}/{}", prefix, ulid::Ulid::new())
}

#[cfg(test)]
mod test {
    use crate::bytestream;

    use super::*;

    #[tokio::test]
    async fn memory() {
        let storage = MemoryBlobStorage::default();
        generic(&storage).await;
    }

    #[tokio::test]
    async fn filesystem() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = FilesystemBlobStorage::new(tempdir.path());
        generic(&storage).await;
    }

    async fn generic(storage: &dyn BlobStorage) {
        generic_get_missing(storage).await;
        generic_read_missing(storage).await;
        generic_delete_missing(storage).await;
        generic_put(storage).await;
        generic_put_delete(storage).await;
        generic_write(storage).await;
        generic_write_delete(storage).await;
        generic_read_range(storage).await;
    }

    async fn generic_get_missing(storage: &dyn BlobStorage) {
        let key = random_key();
        let result = storage.get(&key, Default::default()).await;
        assert!(result.is_err());
    }

    async fn generic_read_missing(storage: &dyn BlobStorage) {
        let key = random_key();
        let result = storage.read(&key, Default::default()).await;
        assert!(result.is_err());
    }

    async fn generic_delete_missing(storage: &dyn BlobStorage) {
        // we should ignore missing keys
        let key = random_key();
        let result = storage.delete(&key).await;
        assert!(!result.is_err());
    }

    async fn generic_put(storage: &dyn BlobStorage) {
        let key = random_key();
        let bytes = Bytes::from_static(b"hello world");
        storage.put(&key, bytes.clone()).await.unwrap();
        let result = storage.get(&key, Default::default()).await.unwrap();
        assert_eq!(result, bytes);
    }

    async fn generic_put_delete(storage: &dyn BlobStorage) {
        let key = random_key();
        let bytes = Bytes::from_static(b"hello world");
        storage.put(&key, bytes.clone()).await.unwrap();
        storage.delete(&key).await.unwrap();
        let result = storage.get(&key, Default::default()).await;
        assert!(result.is_err());
    }

    async fn generic_write(storage: &dyn BlobStorage) {
        let key = random_key();
        let bytes = Bytes::from_static(b"hello world");
        let reader = bytestream::from_bytes(bytes.clone());
        storage.write(&key, reader).await.unwrap();
        let result = storage.get(&key, Default::default()).await.unwrap();
        assert_eq!(result, bytes);
    }

    async fn generic_write_delete(storage: &dyn BlobStorage) {
        let key = random_key();
        let bytes = Bytes::from_static(b"hello world");
        let reader = bytestream::from_bytes(bytes);
        storage.write(&key, reader).await.unwrap();
        storage.delete(&key).await.unwrap();
        let result = storage.get(&key, Default::default()).await;
        assert!(result.is_err());
    }

    async fn generic_read_range(storage: &dyn BlobStorage) {
        let key = random_key();
        let bytes = Bytes::from_static(b"hello world");
        storage.put(&key, bytes.clone()).await.unwrap();
        let result = storage.read(&key, BlobRange::new(1, 3)).await.unwrap();
        let result = bytestream::to_bytes(result).await.unwrap();
        assert_eq!(result, Bytes::from_static(b"ell"));
    }
}
