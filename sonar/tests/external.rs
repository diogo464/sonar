use sonar::{
    bytestream::ByteStream, ExternalAlbum, ExternalArtist, ExternalMediaId, ExternalMediaType,
    ExternalPlaylist, ExternalTrack, Genre, Genres, Properties, PropertyKey, PropertyValue, Result,
};

struct Service1;

const SERVICE1_ID_ARTIST: &str = "service1:artist:1";
const SERVICE1_ID_ALBUM: &str = "service1:album:1";
const SERVICE1_ID_TRACK: &str = "service1:track:1";
const GENRE1: &str = "genre1";
const PROP_KEY1: &str = "key1";
const PROP_VAL1: &str = "val1";

#[sonar::async_trait]
impl sonar::ExternalService for Service1 {
    async fn probe(&self, id: &ExternalMediaId) -> Result<ExternalMediaType> {
        if id.as_str() == SERVICE1_ID_ARTIST {
            Ok(ExternalMediaType::Artist)
        } else if id.as_str() == SERVICE1_ID_ALBUM {
            Ok(ExternalMediaType::Album)
        } else if id.as_str() == SERVICE1_ID_TRACK {
            Ok(ExternalMediaType::Track)
        } else {
            Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid external id",
            ))
        }
    }
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        if id.as_str() != SERVICE1_ID_ARTIST {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid artist id",
            ));
        }
        let mut properties = Properties::default();
        properties.insert(
            PropertyKey::new_uncheked(PROP_KEY1),
            PropertyValue::new_uncheked(PROP_VAL1),
        );

        Ok(ExternalArtist {
            name: "artist1".to_owned(),
            albums: vec![ExternalMediaId::new(SERVICE1_ID_ALBUM)],
            cover: None,
            genres: Genres::from(vec![Genre::new_unchecked(GENRE1)]),
            properties,
        })
    }
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        if id.as_str() != SERVICE1_ID_ALBUM {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid album id",
            ));
        }

        let mut properties = Properties::default();
        properties.insert(
            PropertyKey::new_uncheked(PROP_KEY1),
            PropertyValue::new_uncheked(PROP_VAL1),
        );

        Ok(ExternalAlbum {
            name: "album1".to_owned(),
            artist: ExternalMediaId::new(SERVICE1_ID_ARTIST),
            tracks: vec![ExternalMediaId::new(SERVICE1_ID_TRACK)],
            cover: None,
            genres: Genres::from(vec![Genre::new_unchecked(GENRE1)]),
            properties,
        })
    }
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        if id.as_str() != SERVICE1_ID_TRACK {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }

        Ok(ExternalTrack {
            name: "track1".to_owned(),
            artist: ExternalMediaId::new(SERVICE1_ID_ARTIST),
            album: ExternalMediaId::new(SERVICE1_ID_ALBUM),
            lyrics: None,
            properties: Default::default(),
        })
    }
    async fn fetch_playlist(&self, _id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        todo!()
    }
    async fn download_track(&self, _id: &ExternalMediaId) -> Result<ByteStream> {
        todo!()
    }
}

#[tokio::test]
async fn external_download_track() {
    let mut config = sonar::test::create_config_memory();
    config
        .register_external_service(1, "service1", Service1)
        .unwrap();

    let ctx = sonar::new(config).await.unwrap();
    let user = sonar::test::create_user(&ctx, "user").await;
    sonar::download_request(
        &ctx,
        sonar::DownloadCreate {
            user_id: user.id,
            external_id: ExternalMediaId::new("service1:track:1"),
        },
    )
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let artists = sonar::artist_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].name, "artist1");
    assert_eq!(artists[0].genres.len(), 1);
    assert_eq!(artists[0].genres[0].as_str(), GENRE1);
    assert_eq!(artists[0].properties.len(), 1);
    assert_eq!(
        artists[0].properties.get(PROP_KEY1).unwrap().as_str(),
        PROP_VAL1
    );

    let albums = sonar::album_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].name, "album1");
    assert_eq!(albums[0].artist, artists[0].id);
    assert_eq!(albums[0].genres.len(), 1);
    assert_eq!(albums[0].genres[0].as_str(), GENRE1);
    assert_eq!(albums[0].properties.len(), 1);
    assert_eq!(
        albums[0].properties.get(PROP_KEY1).unwrap().as_str(),
        PROP_VAL1
    );

    let tracks = sonar::track_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(tracks.len(), 1);
}
