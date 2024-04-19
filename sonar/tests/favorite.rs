use sonar::{Album, Artist, Track};

async fn create_artist_album_track(ctx: &sonar::Context) -> (Artist, Album, Track) {
    let artist = sonar::test::create_artist(ctx, "artist").await;
    let album = sonar::test::create_album(ctx, artist.id, "album").await;
    let track = sonar::test::create_track(ctx, album.id, "track").await;
    (artist, album, track)
}

#[tokio::test]
async fn favorite_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let favorites = sonar::favorite_list(&ctx, user.id).await.unwrap();
    assert!(favorites.is_empty());
}

#[tokio::test]
async fn favorite_add_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, _album, _track) = create_artist_album_track(&ctx).await;

    sonar::favorite_add(&ctx, user.id, From::from(artist.id))
        .await
        .unwrap();
    let favorites = sonar::favorite_list(&ctx, user.id).await.unwrap();
    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, From::from(artist.id));
}

#[tokio::test]
async fn favorite_add_three() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, album, track) = create_artist_album_track(&ctx).await;

    sonar::favorite_add(&ctx, user.id, From::from(artist.id))
        .await
        .unwrap();
    sonar::favorite_add(&ctx, user.id, From::from(album.id))
        .await
        .unwrap();
    sonar::favorite_add(&ctx, user.id, From::from(track.id))
        .await
        .unwrap();

    let favorites = sonar::favorite_list(&ctx, user.id).await.unwrap();
    let favorites_ids = favorites.iter().map(|f| f.id).collect::<Vec<_>>();
    assert_eq!(favorites.len(), 3);
    assert!(favorites_ids.contains(&From::from(artist.id)));
    assert!(favorites_ids.contains(&From::from(album.id)));
    assert!(favorites_ids.contains(&From::from(track.id)));
}

#[tokio::test]
async fn favorite_add_repeated() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, _album, _track) = create_artist_album_track(&ctx).await;

    sonar::favorite_add(&ctx, user.id, From::from(artist.id))
        .await
        .unwrap();
    sonar::favorite_add(&ctx, user.id, From::from(artist.id))
        .await
        .unwrap();
    let favorites = sonar::favorite_list(&ctx, user.id).await.unwrap();
    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, From::from(artist.id));
}

#[tokio::test]
async fn favorite_remove() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, album, track) = create_artist_album_track(&ctx).await;

    sonar::favorite_add(&ctx, user.id, From::from(artist.id))
        .await
        .unwrap();
    sonar::favorite_add(&ctx, user.id, From::from(album.id))
        .await
        .unwrap();
    sonar::favorite_add(&ctx, user.id, From::from(track.id))
        .await
        .unwrap();
    sonar::favorite_remove(&ctx, user.id, From::from(album.id))
        .await
        .unwrap();
    let favorites = sonar::favorite_list(&ctx, user.id).await.unwrap();
    let favorites_ids = favorites.iter().map(|f| f.id).collect::<Vec<_>>();
    assert_eq!(favorites.len(), 2);
    assert!(favorites_ids.contains(&From::from(artist.id)));
    assert!(favorites_ids.contains(&From::from(track.id)));
}
