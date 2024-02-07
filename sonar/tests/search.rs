//! NOTE: this tests assume the builtin search engine that just uses a simple 'name LIKE %?%' query.

#[tokio::test]
async fn search_basic() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, album, track) =
        sonar::test::create_artist_album_track(&ctx, "artist", "album", "track").await;

    let result = sonar::search(
        &ctx,
        user.id,
        sonar::SearchQuery {
            query: "".parse().unwrap(),
            limit: None,
            flags: sonar::SearchQuery::FLAG_ALL,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.results.len(), 3);

    let artists = result.artists().collect::<Vec<_>>();
    let albums = result.albums().collect::<Vec<_>>();
    let tracks = result.tracks().collect::<Vec<_>>();

    assert_eq!(artists.len(), 1);
    assert_eq!(albums.len(), 1);
    assert_eq!(tracks.len(), 1);

    assert_eq!(artists[0].id, artist.id);
    assert_eq!(albums[0].id, album.id);
    assert_eq!(tracks[0].id, track.id);
}

#[tokio::test]
async fn search_filter() {
    let ctx = sonar::test::create_context_memory().await;
    let user = sonar::test::create_user(&ctx, "user").await;
    let (artist, _album, _track) =
        sonar::test::create_artist_album_track(&ctx, "artist", "album", "track").await;

    let result = sonar::search(
        &ctx,
        user.id,
        sonar::SearchQuery {
            query: "artist".parse().unwrap(),
            limit: None,
            flags: sonar::SearchQuery::FLAG_ALL,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.results.len(), 1);

    let artists = result.artists().collect::<Vec<_>>();
    let albums = result.albums().collect::<Vec<_>>();
    let tracks = result.tracks().collect::<Vec<_>>();

    assert_eq!(artists.len(), 1);
    assert_eq!(albums.len(), 0);
    assert_eq!(tracks.len(), 0);

    assert_eq!(artists[0].id, artist.id);
}
