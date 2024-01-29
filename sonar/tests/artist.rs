#[tokio::test]
async fn artist_list_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let artists = sonar::artist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 0);
}

#[tokio::test]
async fn artist_create_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        properties: sonar::test::create_simple_properties(),
    };
    let artist = sonar::artist_create(&ctx, create).await.unwrap();
    assert_eq!(artist.name, "Artist");
    assert_eq!(artist.properties.len(), 2);
}

#[tokio::test]
async fn artist_list_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let artists = sonar::artist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].id, artist.id);
}

#[tokio::test]
async fn artist_list_two() {
    let ctx = sonar::test::create_context_memory().await;
    let artist1 = sonar::test::create_artist(&ctx, "artist1").await;
    let artist2 = sonar::test::create_artist(&ctx, "artist2").await;
    let artists = sonar::artist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 2);
    assert_eq!(artists[0].id, artist1.id);
    assert_eq!(artists[1].id, artist2.id);
}

#[tokio::test]
async fn artist_update_one() {
    let ctx = sonar::test::create_context_memory().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        properties: sonar::test::create_simple_properties(),
    };
    let artist = sonar::artist_create(&ctx, create).await.unwrap();
    let update = sonar::ArtistUpdate {
        name: sonar::ValueUpdate::Set("Artist2".to_string()),
        properties: vec![sonar::PropertyUpdate::set(
            sonar::PropertyKey::new_uncheked("key3"),
            sonar::PropertyValue::new_uncheked("value3"),
        )],
        ..Default::default()
    };
    let artist = sonar::artist_update(&ctx, artist.id, update).await.unwrap();
    assert_eq!(artist.name, "Artist2");
    assert_eq!(artist.properties.len(), 3);
}

#[tokio::test]
async fn artist_delete_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist1 = sonar::test::create_artist(&ctx, "artist").await;
    let artist2 = sonar::test::create_artist(&ctx, "artist").await;
    sonar::artist_delete(&ctx, artist1.id).await.unwrap();
    let artists = sonar::artist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].id, artist2.id);
}

#[tokio::test]
async fn artist_list_offset() {
    let ctx = sonar::test::create_context_memory().await;
    let _artist1 = sonar::test::create_artist(&ctx, "artist1").await;
    let artist2 = sonar::test::create_artist(&ctx, "artist2").await;
    let artists = sonar::artist_list(&ctx, sonar::ListParams::default().with_offset(1))
        .await
        .unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].id, artist2.id);
}
