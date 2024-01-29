#[tokio::test]
async fn track_list_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let tracks = sonar::track_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn track_create_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let data = sonar::test::create_stream(b"track data");
    let create = sonar::TrackCreate {
        name: "Track".to_string(),
        album: album.id,
        cover_art: None,
        duration: std::time::Duration::from_secs(60),
        lyrics: None,
        properties: sonar::test::create_simple_properties(),
        audio_stream: data,
        audio_filename: "track.mp3".to_string(),
    };
    let track = sonar::track_create(&ctx, create).await.unwrap();
    assert_eq!(track.name, "Track");
    assert_eq!(track.properties.len(), 2);
}

#[tokio::test]
async fn track_list_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track = sonar::test::create_track(&ctx, album.id, "track").await;
    let tracks = sonar::track_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].id, track.id);
}

#[tokio::test]
async fn track_list_two() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track1 = sonar::test::create_track(&ctx, album.id, "track2").await;
    let track2 = sonar::test::create_track(&ctx, album.id, "track2").await;
    let tracks = sonar::track_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].id, track1.id);
    assert_eq!(tracks[1].id, track2.id);
}

#[tokio::test]
async fn track_delete_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track1 = sonar::test::create_track(&ctx, album.id, "track2").await;
    let track2 = sonar::test::create_track(&ctx, album.id, "track2").await;
    sonar::track_delete(&ctx, track1.id).await.unwrap();
    let tracks = sonar::track_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].id, track2.id);
}

#[tokio::test]
async fn track_download_one() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track = sonar::test::create_track_with_data(&ctx, album.id, "track", b"track data").await;
    let reader = sonar::track_download(&ctx, track.id, sonar::ByteRange::default())
        .await
        .unwrap();
    let downloaded = sonar::bytestream::to_bytes(reader).await.unwrap();
    assert_eq!(downloaded, &b"track data"[..]);
}
