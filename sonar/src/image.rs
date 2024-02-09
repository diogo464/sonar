use bytes::Bytes;

use crate::{
    blob::{self, BlobStorage},
    bytestream::{self, ByteStream},
    db::DbC,
    ks, Error, ErrorKind, ImageId, Result,
};

pub struct ImageCreate {
    pub data: ByteStream,
}

pub struct ImageDownload {
    pub mime_type: String,
    pub stream: ByteStream,
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

#[tracing::instrument(skip(db))]
pub async fn download(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    image_id: ImageId,
) -> Result<ImageDownload> {
    let db_id = image_id.to_db();
    let row = sqlx::query!(
        "SELECT mime_type, blob_key FROM sqlx_image WHERE id = ?",
        db_id
    )
    .fetch_one(db)
    .await?;
    let stream = storage.read(&row.blob_key, Default::default()).await?;
    Ok(ImageDownload::new(row.mime_type, stream))
}

#[tracing::instrument(skip(db))]
pub async fn create(
    db: &mut DbC,
    storage: &dyn BlobStorage,
    create: ImageCreate,
) -> Result<ImageId> {
    let blob_key = blob::random_key_with_prefix("image");
    let img_file = tempfile::NamedTempFile::new()?;
    bytestream::to_file(create.data, img_file.path()).await?;
    let blob_sha256 = ks::sha256_file(img_file.path()).await?;
    let blob_size = img_file.path().metadata()?.len() as u32;

    let mime_type = match infer::get_from_path(img_file.path())? {
        Some(ty) => match ty.extension() {
            "jpg" | "jpeg" | "png" => Some(ty.mime_type()),
            _ => None,
        },
        None => None,
    };
    if mime_type.is_none() {
        return Err(Error::new(ErrorKind::Invalid, "invalid image type"));
    }

    let stream = bytestream::from_file(img_file.path()).await?;
    storage.write(&blob_key, stream).await?;
    let blob_id = sqlx::query!(
        "INSERT INTO blob (key, size, sha256) VALUES (?, ?, ?) RETURNING id",
        blob_key,
        blob_size,
        blob_sha256
    )
    .fetch_one(&mut *db)
    .await?
    .id;

    let db_id = sqlx::query!(
        "INSERT INTO image (mime_type, blob) VALUES (?, ?) RETURNING id",
        mime_type,
        blob_id
    )
    .fetch_one(db)
    .await?
    .id;

    Ok(ImageId::from_db(db_id))
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, image_id: ImageId) -> Result<()> {
    let db_id = image_id.to_db();
    sqlx::query!("DELETE FROM image WHERE id = ?", db_id)
        .execute(db)
        .await?;
    Ok(())
}
