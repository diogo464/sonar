use std::time::Duration;

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
    let _data = sonar::test::create_stream(b"track data");
    let create = sonar::TrackCreate {
        name: "Track".to_string(),
        album: album.id,
        cover_art: None,
        audio: None,
        lyrics: None,
        properties: sonar::test::create_simple_properties(),
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
async fn track_get_bulk() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track1 = sonar::test::create_track(&ctx, album.id, "track1").await;
    let track2 = sonar::test::create_track(&ctx, album.id, "track2").await;
    let tracks = sonar::track_get_bulk(&ctx, &[track1.id, track2.id])
        .await
        .unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].id, track1.id);
    assert_eq!(tracks[1].id, track2.id);
}

#[tokio::test]
async fn track_get_bulk_repeated() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let track1 = sonar::test::create_track(&ctx, album.id, "track1").await;
    let track2 = sonar::test::create_track(&ctx, album.id, "track2").await;
    let track3 = sonar::test::create_track(&ctx, album.id, "track3").await;
    let tracks = sonar::track_get_bulk(&ctx, &[track1.id, track2.id, track3.id, track2.id])
        .await
        .unwrap();
    assert_eq!(tracks.len(), 4);
    assert_eq!(tracks[0].id, track1.id);
    assert_eq!(tracks[1].id, track2.id);
    assert_eq!(tracks[2].id, track3.id);
    assert_eq!(tracks[3].id, track2.id);
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
    let audio = sonar::test::create_audio(&ctx, sonar::test::SMALL_AUDIO_MP3).await;
    let track = sonar::test::create_track_with_audio(&ctx, album.id, "track", audio.id).await;
    let download = sonar::track_download(&ctx, track.id, sonar::ByteRange::default())
        .await
        .unwrap();
    let downloaded = sonar::bytestream::to_bytes(download.stream).await.unwrap();
    assert_eq!(downloaded, sonar::test::SMALL_AUDIO_MP3);
}

#[tokio::test]
async fn track_with_lyrics() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let lyrics = sonar::TrackLyrics {
        kind: sonar::LyricsKind::Synced,
        lines: vec![sonar::LyricsLine {
            offset: Duration::from_secs(1),
            text: "Lyrics".to_string(),
            duration: Duration::from_secs(1),
        }],
    };

    let track = sonar::track_create(
        &ctx,
        sonar::TrackCreate {
            name: "Track".to_string(),
            album: album.id,
            cover_art: None,
            audio: None,
            lyrics: Some(lyrics.clone()),
            properties: sonar::test::create_simple_properties(),
        },
    )
    .await
    .unwrap();

    let fetched_lyrics = sonar::track_get_lyrics(&ctx, track.id).await.unwrap();
    assert_eq!(fetched_lyrics.kind, lyrics.kind);
    assert_eq!(fetched_lyrics.lines, lyrics.lines);
}

#[tokio::test]
async fn track_list_properties() {
    let ctx = sonar::test::create_context_memory().await;
    let artist = sonar::test::create_artist(&ctx, "artist").await;
    let album = sonar::test::create_album(&ctx, artist.id, "album").await;
    let _data = sonar::test::create_stream(b"track data");
    let create = sonar::TrackCreate {
        name: "Track".to_string(),
        album: album.id,
        cover_art: None,
        audio: None,
        lyrics: None,
        properties: sonar::test::create_simple_properties(),
    };
    sonar::track_create(&ctx, create).await.unwrap();
    let tracks = sonar::track_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].name, "Track");
    assert_eq!(tracks[0].properties.len(), 2);
}
