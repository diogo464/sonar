use bytes::Bytes;

use crate::{
    blob::{self, BlobStorage},
    bytestream::ByteStream,
    db::DbC,
    ImageId, Result,
};

pub struct ImageCreate {
    pub data: ByteStream,
}

pub struct ImageDownload {
    mime_type: String,
    stream: ByteStream,
}

impl ImageDownload {
    pub(crate) fn new(mime_type: String, stream: ByteStream) -> Self {
        Self { mime_type, stream }
    }
}

impl std::fmt::Debug for ImageDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageDownload")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}

impl tokio_stream::Stream for ImageDownload {
    type Item = std::io::Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut *self.get_mut().stream).poll_next(cx)
    }
}

pub async fn download(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    image_id: ImageId,
) -> Result<ImageDownload> {
    let db_id = image_id.to_db();
    let row = sqlx::query!("SELECT mime_type, blob_key FROM image WHERE id = ?", db_id)
        .fetch_one(db)
        .await?;
    let stream = storage.read(&row.blob_key, Default::default()).await?;
    Ok(ImageDownload::new(row.mime_type, stream))
}

pub async fn create(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    create: ImageCreate,
) -> Result<ImageId> {
    let blob_key = blob::random_key_with_prefix("image");
    storage.write(&blob_key, create.data).await?;
    // TODO: infer mime type from blob
    let db_id = sqlx::query!(
        "INSERT INTO image (mime_type, blob_key) VALUES (?, ?) RETURNING id",
        "image/jpeg",
        blob_key
    )
    .fetch_one(db)
    .await?
    .id;
    Ok(ImageId::from_db(db_id))
}

pub async fn delete(db: &mut DbC, image_id: ImageId) -> Result<()> {
    let db_id = image_id.to_db();
    sqlx::query!("DELETE FROM image WHERE id = ?", db_id)
        .execute(db)
        .await?;
    Ok(())
}
