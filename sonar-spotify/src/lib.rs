use std::sync::Arc;

use sonar::{
    bytestream::ByteStream, ExternalAlbum, ExternalArtist, ExternalImage, ExternalMediaId,
    ExternalMediaType, ExternalPlaylist, ExternalTrack, Result,
};
pub use spotdl::{session::LoginCredentials, Resource, ResourceId, SpotifyId};

mod convert;

pub struct SpotifyService {
    session: spotdl::session::Session,
    fetcher: Arc<dyn spotdl::fetcher::MetadataFetcher>,
    _cache_directory: tempfile::TempDir,
}

impl SpotifyService {
    pub async fn new(credentials: LoginCredentials) -> Result<Self> {
        let credentials = spotdl::session::login(&credentials)
            .await
            .map_err(sonar::Error::wrap)?;

        let session = spotdl::session::Session::connect(credentials)
            .await
            .map_err(sonar::Error::wrap)?;

        let cache_directory = tempfile::tempdir().map_err(sonar::Error::wrap)?;
        let fetcher = spotdl::fetcher::SpotifyMetadataFetcher::new(session.clone());
        let fetcher = spotdl::fetcher::FsCacheMetadataFetcher::new(
            fetcher,
            cache_directory.path().to_owned(),
        )
        .await
        .map_err(sonar::Error::wrap)?;
        let fetcher = Arc::new(fetcher);

        Ok(Self {
            session,
            fetcher,
            _cache_directory: cache_directory,
        })
    }
}

#[sonar::async_trait]
impl sonar::ExternalService for SpotifyService {
    #[tracing::instrument(skip(self))]
    async fn validate_id(&self, id: &ExternalMediaId) -> Result<ExternalMediaType> {
        tracing::debug!("validating id: {}", id);
        let resource_id = parse_resource_id(id)?;
        let media_type = match resource_id.resource {
            Resource::Artist => ExternalMediaType::Artist,
            Resource::Album => ExternalMediaType::Album,
            Resource::Track => ExternalMediaType::Track,

            Resource::Playlist => ExternalMediaType::Playlist,
        };
        Ok(media_type)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        tracing::info!("fetching artist: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Artist {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid artist id",
            ));
        }
        let artist = self
            .fetcher
            .get_artist(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("artist: {:#?}", artist);
        Ok(ExternalArtist {
            name: artist.name,
            albums: artist
                .albums
                .into_iter()
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            cover: None,
            properties: properties_for_resource(resource_id.id),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        tracing::info!("fetching album: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Album {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid album id",
            ));
        }
        let album = self
            .fetcher
            .get_album(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("album: {:#?}", album);
        let cover = match album.cover {
            Some(url) => Some(ExternalImage {
                data: reqwest::get(url)
                    .await
                    .map_err(sonar::Error::wrap)?
                    .bytes()
                    .await
                    .map_err(sonar::Error::wrap)?
                    .to_vec(),
            }),
            None => None,
        };
        let external = ExternalAlbum {
            name: album.name,
            artist: ExternalMediaId::new(album.artists[0].to_string()),
            tracks: album
                .discs
                .into_iter()
                .flat_map(|disc| disc.tracks)
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            cover,
            properties: properties_for_resource(resource_id.id),
        };
        tracing::debug!("external album: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        tracing::info!("fetching track: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Track {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }
        let track = self
            .fetcher
            .get_track(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("track: {:#?}", track);

        let external = ExternalTrack {
            name: track.name,
            artist: ExternalMediaId::new(track.artists[0].to_string()),
            album: ExternalMediaId::new(track.album.to_string()),
            lyrics: None, // TODO: fetch lyrics
            properties: properties_for_resource(resource_id.id),
        };
        tracing::debug!("external track: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        tracing::info!("fetching playlist: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Playlist {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid playlist id",
            ));
        }

        let playlist = self
            .fetcher
            .get_playlist(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("playlist: {:#?}", playlist);

        let external = ExternalPlaylist {
            name: playlist.name,
            tracks: playlist
                .tracks
                .into_iter()
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            properties: properties_for_resource(resource_id.id),
        };
        tracing::debug!("external playlist: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        tracing::info!("downloading track: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Track {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }

        tracing::debug!("preparing to download track: {:#?}", resource_id.id);
        let track_id = resource_id.id;
        let temp_dir = tempfile::tempdir().map_err(sonar::Error::wrap)?;
        let temp_file_path = temp_dir.path().join("samples");
        let mp3_file_path = temp_dir.path().join("track.mp3");
        let sink = spotdl::download::FileDownloadSink::from_path(&temp_file_path)
            .map_err(sonar::Error::wrap)?;

        tracing::debug!("downloading track: {:#?}", track_id);
        spotdl::download::download(&self.session, sink, track_id)
            .await
            .map_err(sonar::Error::wrap)?;

        tracing::debug!("converting samples to mp3: {:#?}", mp3_file_path);
        convert::convert_samples_i16(
            convert::ConvertSamplesSource::Path(&temp_file_path),
            &mp3_file_path,
        )
        .await
        .map_err(sonar::Error::wrap)?;

        tracing::debug!("creating strem from file: {:#?}", mp3_file_path);
        Ok(sonar::bytestream::from_file(&mp3_file_path).await?)
    }
}

fn parse_resource_id(external_id: &ExternalMediaId) -> Result<ResourceId> {
    let resource_id = external_id
        .as_str()
        .parse::<ResourceId>()
        .map_err(|_| sonar::Error::new(sonar::ErrorKind::Invalid, "invalid spotify id"))?;
    Ok(resource_id)
}

fn properties_for_resource(id: SpotifyId) -> sonar::Properties {
    let mut properties = sonar::Properties::default();
    properties.insert(
        sonar::prop::EXTERNAL_SPOTIFY_ID,
        sonar::PropertyValue::new(id.to_string()).unwrap(),
    );
    properties
}
