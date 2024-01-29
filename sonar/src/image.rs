use crate::{
    blob::{self, BlobStorage},
    DbC, ImageCreate, ImageDownload, ImageId, Result,
};

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
