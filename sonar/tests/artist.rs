mod common;
use common::*;

#[tokio::test]
async fn artist_list_empty() {
    let context = create_context().await;
    let artists = sonar::artist_list(&context, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 0);
}

#[tokio::test]
async fn artist_create_one() {
    tracing_subscriber::fmt::init();
    let context = create_context().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        genres: create_simple_genres(),
        properties: create_simple_properties(),
    };
    let artist = sonar::artist_create(&context, create).await.unwrap();
    assert_eq!(artist.name, "Artist");
    assert_eq!(artist.genres.len(), 2);
    assert_eq!(artist.properties.len(), 2);
}

#[tokio::test]
async fn artist_list_one() {
    let context = create_context().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        genres: create_simple_genres(),
        properties: create_simple_properties(),
    };
    let artist = sonar::artist_create(&context, create).await.unwrap();
    let artists = sonar::artist_list(&context, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].id, artist.id);
}

#[tokio::test]
async fn artist_update_one() {
    let context = create_context().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        genres: create_simple_genres(),
        properties: create_simple_properties(),
    };
    let artist = sonar::artist_create(&context, create).await.unwrap();
    let update = sonar::ArtistUpdate {
        name: sonar::ValueUpdate::Set("Artist2".to_string()),
        genres: vec![sonar::GenreUpdate::set("rock".parse().unwrap())],
        properties: vec![sonar::PropertyUpdate::set(
            sonar::PropertyKey::new_uncheked("key3"),
            sonar::PropertyValue::new_uncheked("value3"),
        )],
        ..Default::default()
    };
    let artist = sonar::artist_update(&context, artist.id, update)
        .await
        .unwrap();
    assert_eq!(artist.name, "Artist2");
    assert_eq!(artist.genres.len(), 3);
    assert_eq!(artist.properties.len(), 3);
}

#[tokio::test]
async fn artist_delete_one() {
    let context = create_context().await;
    let create = sonar::ArtistCreate {
        name: "Artist".to_string(),
        cover_art: None,
        genres: create_simple_genres(),
        properties: create_simple_properties(),
    };
    let artist = sonar::artist_create(&context, create).await.unwrap();
    sonar::artist_delete(&context, artist.id).await.unwrap();
    let artists = sonar::artist_list(&context, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(artists.len(), 0);
}
