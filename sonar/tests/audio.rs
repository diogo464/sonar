#[tokio::test]
async fn audio_list_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let (_, _, track) =
        sonar::test::create_artist_album_track(&ctx, "artist", "album", "track").await;
    let audios = sonar::audio_list_by_track(&ctx, track.id).await.unwrap();
    assert_eq!(audios.len(), 0);
}

#[tokio::test]
async fn audio_link_one() {
    let ctx = sonar::test::create_context_memory().await;
    let (_, _, track) =
        sonar::test::create_artist_album_track(&ctx, "artist", "album", "track").await;
    let audio = sonar::audio_create(
        &ctx,
        sonar::AudioCreate {
            stream: sonar::test::create_stream(sonar::test::SMALL_AUDIO_MP3),
            filename: None,
        },
    )
    .await
    .unwrap();
    sonar::audio_link(&ctx, audio.id, track.id).await.unwrap();

    let audios = sonar::audio_list_by_track(&ctx, track.id).await.unwrap();
    assert_eq!(audios.len(), 1);
    assert_eq!(audios[0].id, audio.id);
}

#[tokio::test]
async fn audio_unlink_one() {
    let ctx = sonar::test::create_context_memory().await;
    let (_, _, track) =
        sonar::test::create_artist_album_track(&ctx, "artist", "album", "track").await;
    let audio = sonar::audio_create(
        &ctx,
        sonar::AudioCreate {
            stream: sonar::test::create_stream(sonar::test::SMALL_AUDIO_MP3),
            filename: None,
        },
    )
    .await
    .unwrap();
    sonar::audio_link(&ctx, audio.id, track.id).await.unwrap();

    let audios = sonar::audio_list_by_track(&ctx, track.id).await.unwrap();
    assert_eq!(audios.len(), 1);
    assert_eq!(audios[0].id, audio.id);

    sonar::audio_unlink(&ctx, audio.id, track.id).await.unwrap();
    let audios = sonar::audio_list_by_track(&ctx, track.id).await.unwrap();
    assert_eq!(audios.len(), 0);
}
