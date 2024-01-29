#[tokio::test]
async fn album_list_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let albums = sonar::album_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(albums.len(), 0);
}

#[tokio::test]
async fn album_create_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let release_date = "2020-01-01T00:00:00Z".parse().unwrap();
    let create = sonar::AlbumCreate {
        name: "Album".to_string(),
        artist: artist.id,
        cover_art: None,
        release_date,
        properties: sonar::test::create_simple_properties(),
    };
    let album = sonar::album_create(&ctx, create).await.unwrap();
    assert_eq!(album.name, "Album");
    assert_eq!(album.properties.len(), 2);
}

#[tokio::test]
async fn album_list_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let albums = sonar::album_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].id, album.id);
}

#[tokio::test]
async fn album_list_two() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album1 = sonar::test::create_album(&ctx, artist.id, "album1").await;
    let album2 = sonar::test::create_album(&ctx, artist.id, "album2").await;

    let albums = sonar::album_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(albums.len(), 2);
    assert_eq!(albums[0].id, album1.id);
    assert_eq!(albums[1].id, album2.id);
}

#[tokio::test]
async fn album_list_offset() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let _album1 = sonar::test::create_album(&ctx, artist.id, "album1").await;
    let _album2 = sonar::test::create_album(&ctx, artist.id, "album2").await;
    let albums = sonar::album_list(&ctx, sonar::ListParams::default().with_offset(1))
        .await
        .unwrap();
    assert_eq!(albums.len(), 1);
}

#[tokio::test]
async fn album_list_limit() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let _album1 = sonar::test::create_album(&ctx, artist.id, "album1").await;
    let _album2 = sonar::test::create_album(&ctx, artist.id, "album2").await;
    let albums = sonar::album_list(&ctx, sonar::ListParams::default().with_limit(1))
        .await
        .unwrap();
    assert_eq!(albums.len(), 1);
}

#[tokio::test]
async fn album_delete_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album1 = sonar::test::create_album(&ctx, artist.id, "album1").await;
    let album2 = sonar::test::create_album(&ctx, artist.id, "album2").await;
    sonar::album_delete(&ctx, album1.id).await.unwrap();
    let albums = sonar::album_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].id, album2.id);
}
