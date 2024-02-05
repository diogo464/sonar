use sonar::{
    bytestream::ByteStream,
    external::{
        ExternalAlbum, ExternalArtist, ExternalMediaId, ExternalMediaType, ExternalPlaylist,
        ExternalTrack,
    },
    Result,
};

struct Service1;

const SERVICE1_ID_ARTIST: &'static str = "service1:artist:1";
const SERVICE1_ID_ALBUM: &'static str = "service1:album:1";
const SERVICE1_ID_TRACK: &'static str = "service1:track:1";

#[sonar::async_trait]
impl sonar::external::ExternalService for Service1 {
    async fn validate_id(&self, id: &ExternalMediaId) -> Result<ExternalMediaType> {
        if id.as_str() == SERVICE1_ID_ARTIST {
            Ok(ExternalMediaType::Artist)
        } else if id.as_str() == SERVICE1_ID_ALBUM {
            Ok(ExternalMediaType::Album)
        } else if id.as_str() == SERVICE1_ID_TRACK {
            Ok(ExternalMediaType::Track)
        } else {
            Ok(ExternalMediaType::Invalid)
        }
    }
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        if id.as_str() != SERVICE1_ID_ARTIST {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid artist id",
            ));
        }

        Ok(ExternalArtist {
            name: "artist1".to_owned(),
            albums: vec![ExternalMediaId::new(SERVICE1_ID_ALBUM)],
            properties: Default::default(),
        })
    }
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        if id.as_str() != SERVICE1_ID_ALBUM {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid album id",
            ));
        }

        Ok(ExternalAlbum {
            name: "album1".to_owned(),
            artist: ExternalMediaId::new(SERVICE1_ID_ARTIST),
            tracks: vec![ExternalMediaId::new(SERVICE1_ID_TRACK)],
            properties: Default::default(),
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
            properties: Default::default(),
        })
    }
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        todo!()
    }
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        todo!()
    }
}

#[tokio::test]
async fn download_track() {
    let mut config = sonar::test::create_config_memory();
    config
        .register_external_service(1, "service1", Service1)
        .unwrap();

    let ctx = sonar::new(config).await.unwrap();
    let user = sonar::test::create_user(&ctx, "user").await;
    sonar::external_download_request(
        &ctx,
        sonar::ExternalDownloadRequest {
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

    let albums = sonar::album_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].name, "album1");

    let tracks = sonar::track_list(&ctx, Default::default()).await.unwrap();
    assert_eq!(tracks.len(), 1);
}
