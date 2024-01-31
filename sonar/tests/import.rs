use std::time::Duration;

use sonar::{extractor::ExtractedMetadata, Genres};

#[tokio::test]
async fn import_simple() {
    let metadata = ExtractedMetadata {
        title: Some("title".to_string()),
        album: Some("album".to_string()),
        artist: Some("artist".to_string()),
        track_number: Some(4),
        disc_number: Some(2),
        duration: None,
        release_date: None,
        cover_art: None,
        genres: Genres::new(vec!["edm"]).unwrap(),
    };
    let extractor = sonar::test::StaticMetadataExtractor::new(metadata.clone());
    let mut config = sonar::test::create_config_memory();
    config.register_extractor("extractor", extractor).unwrap();
    let ctx = sonar::test::create_context(config).await;

    sonar::import(
        &ctx,
        sonar::Import {
            artist: None,
            album: None,
            filepath: Some("test.mp3".to_string()),
            stream: sonar::test::create_stream(sonar::test::SMALL_AUDIO_MP3),
        },
    )
    .await
    .unwrap();

    let artists = sonar::artist_list(&ctx, Default::default()).await.unwrap();
    let albums = sonar::album_list(&ctx, Default::default()).await.unwrap();
    let tracks = sonar::track_list(&ctx, Default::default()).await.unwrap();

    assert_eq!(artists.len(), 1);
    assert_eq!(albums.len(), 1);
    assert_eq!(tracks.len(), 1);

    let artist = &artists[0];
    assert_eq!(artist.name, metadata.artist.unwrap());

    let album = &albums[0];
    assert_eq!(album.name, metadata.album.unwrap());
    assert_eq!(album.artist, artist.id);

    let track = &tracks[0];
    assert_eq!(track.name, metadata.title.unwrap());
    assert_eq!(track.album, album.id);
    assert_eq!(track.duration, sonar::test::SMALL_AUDIO_MP3_DURATION);
}

#[tokio::test]
async fn import_merge_metadata() {
    let metadata1 = ExtractedMetadata {
        album: Some("album".to_string()),
        artist: Some("artist".to_string()),
        disc_number: Some(2),
        duration: None,
        release_date: None,
        cover_art: None,
        title: Default::default(),
        track_number: Default::default(),
        genres: Default::default(),
    };
    let metadata2 = ExtractedMetadata {
        title: Some("title".to_string()),
        track_number: Some(4),
        release_date: None,
        cover_art: None,
        genres: Genres::new(vec!["edm"]).unwrap(),
        album: Default::default(),
        artist: Default::default(),
        disc_number: Default::default(),
        duration: Default::default(),
    };
    let extractor1 = sonar::test::StaticMetadataExtractor::new(metadata1.clone());
    let extractor2 = sonar::test::StaticMetadataExtractor::new(metadata2.clone());
    let mut config = sonar::test::create_config_memory();
    config.register_extractor("extractor1", extractor1).unwrap();
    config.register_extractor("extractor2", extractor2).unwrap();
    let ctx = sonar::test::create_context(config).await;

    sonar::import(
        &ctx,
        sonar::Import {
            artist: None,
            album: None,
            filepath: Some("test.mp3".to_string()),
            stream: sonar::test::create_stream(sonar::test::SMALL_AUDIO_MP3),
        },
    )
    .await
    .unwrap();

    let artists = sonar::artist_list(&ctx, Default::default()).await.unwrap();
    let albums = sonar::album_list(&ctx, Default::default()).await.unwrap();
    let tracks = sonar::track_list(&ctx, Default::default()).await.unwrap();

    assert_eq!(artists.len(), 1);
    assert_eq!(albums.len(), 1);
    assert_eq!(tracks.len(), 1);

    let artist = &artists[0];
    assert_eq!(artist.name, metadata1.artist.unwrap());

    let album = &albums[0];
    assert_eq!(album.name, metadata1.album.unwrap());
    assert_eq!(album.artist, artist.id);

    let track = &tracks[0];
    assert_eq!(track.name, metadata2.title.unwrap());
    assert_eq!(track.album, album.id);
    assert_eq!(track.duration, sonar::test::SMALL_AUDIO_MP3_DURATION);
}
