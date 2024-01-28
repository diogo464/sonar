use sonar::{Album, Artist, Track};

async fn create_artist_album_track(ctx: &sonar::Context) -> (Artist, Album, Track) {
    let artist = sonar::test::create_artist(ctx, "artist").await;
    let album = sonar::test::create_album(ctx, artist.id, "album").await;
    let track = sonar::test::create_track(ctx, album.id, "track").await;
    (artist, album, track)
}

#[tokio::test]
async fn playlist_list_empty() {
    let ctx = sonar::test::create_context_memory().await;
    let playlists = sonar::playlist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(playlists.len(), 0);
}

#[tokio::test]
async fn playlist_create_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let playlist = sonar::playlist_create(
        &ctx,
        sonar::PlaylistCreate {
            name: "Playlist".to_string(),
            owner: user.id,
            tracks: Default::default(),
            properties: sonar::test::create_simple_properties(),
        },
    )
    .await
    .unwrap();
    assert_eq!(playlist.name, "Playlist");
    assert_eq!(playlist.owner, user.id);
    assert_eq!(playlist.properties.len(), 2);
}

#[tokio::test]
async fn playlist_list_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let playlist = sonar::test::create_playlist(&ctx, user.id, "Playlist").await;
    let playlists = sonar::playlist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].id, playlist.id);
}

#[tokio::test]
async fn playlist_list_two() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let playlist1 = sonar::test::create_playlist(&ctx, user.id, "Playlist1").await;
    let playlist2 = sonar::test::create_playlist(&ctx, user.id, "Playlist2").await;
    let playlists = sonar::playlist_list(&ctx, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(playlists.len(), 2);
    assert_eq!(playlists[0].id, playlist1.id);
    assert_eq!(playlists[1].id, playlist2.id);
}

#[tokio::test]
async fn playlist_get_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let playlist = sonar::test::create_playlist(&ctx, user.id, "Playlist").await;
    let playlist = sonar::playlist_get(&ctx, playlist.id).await.unwrap();
    assert_eq!(playlist.name, "Playlist");
    assert_eq!(playlist.owner, user.id);
}

#[tokio::test]
async fn playlist_get_tracks_one() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let playlist = sonar::test::create_playlist(&ctx, user.id, "Playlist").await;
    let (_artist, _album, track) = create_artist_album_track(&ctx).await;
    sonar::playlist_insert_tracks(&ctx, playlist.id, &[track.id])
        .await
        .unwrap();
    let tracks = sonar::playlist_list_tracks(&ctx, playlist.id, sonar::ListParams::default())
        .await
        .unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].playlist, playlist.id);
    assert_eq!(tracks[0].track, track.id);
}
