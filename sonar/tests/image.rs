use bytes::Bytes;

#[tokio::test]
async fn create_image_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::ImageCreate {
        data: sonar::test::create_stream(sonar::test::SMALL_IMAGE_JPEG),
    };
    let _image_id = sonar::image_create(&ctx, create).await.unwrap();
}

#[tokio::test]
async fn read_image_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::ImageCreate {
        data: sonar::test::create_stream(sonar::test::SMALL_IMAGE_JPEG),
    };
    let image_id = sonar::image_create(&ctx, create).await.unwrap();
    let reader = sonar::image_download(&ctx, image_id).await.unwrap();
    let data = sonar::bytestream::to_bytes(reader).await.unwrap();
    assert_eq!(data, Bytes::from_static(sonar::test::SMALL_IMAGE_JPEG));
}

#[tokio::test]
async fn delete_image_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::ImageCreate {
        data: sonar::bytestream::from_bytes(Bytes::from_static(sonar::test::SMALL_IMAGE_JPEG)),
    };
    let image_id = sonar::image_create(&ctx, create).await.unwrap();
    sonar::image_delete(&ctx, image_id).await.unwrap();
    let reader = sonar::image_download(&ctx, image_id).await;
    assert!(reader.is_err());
}
