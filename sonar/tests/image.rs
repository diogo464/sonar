mod common;
use bytes::Bytes;
use common::*;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn create_image_one() {
    let context = create_context().await;
    let create = sonar::ImageCreate {
        data: sonar::bytestream::from_bytes(Bytes::from_static(b"image data")),
    };
    let _image_id = sonar::image_create(&context, create).await.unwrap();
}

#[tokio::test]
async fn read_image_one() {
    let context = create_context().await;
    let create = sonar::ImageCreate {
        data: sonar::bytestream::from_bytes(Bytes::from_static(b"image data")),
    };
    let image_id = sonar::image_create(&context, create).await.unwrap();
    let reader = sonar::image_reader(&context, image_id).await.unwrap();
    let mut buf = Vec::new();
    tokio_util::io::StreamReader::new(reader)
        .read_to_end(&mut buf)
        .await
        .unwrap();
    assert_eq!(buf, b"image data");
}

#[tokio::test]
async fn delete_image_one() {
    let context = create_context().await;
    let create = sonar::ImageCreate {
        data: sonar::bytestream::from_bytes(Bytes::from_static(b"image data")),
    };
    let image_id = sonar::image_create(&context, create).await.unwrap();
    sonar::image_delete(&context, image_id).await.unwrap();
    let reader = sonar::image_reader(&context, image_id).await;
    assert!(reader.is_err());
}
