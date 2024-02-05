use std::sync::Arc;

use sonar::{
    bytestream::ByteStream, ExternalAlbum, ExternalArtist, ExternalMediaId, ExternalMediaType,
    ExternalPlaylist, ExternalTrack, Result,
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
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        tracing::debug!("fetching artist: {}", id);
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
        Ok(ExternalArtist {
            name: artist.name,
            albums: artist
                .albums
                .into_iter()
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            // TODO: set spotify id
            properties: Default::default(),
        })
    }
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        tracing::debug!("fetching album: {}", id);
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
        Ok(ExternalAlbum {
            name: album.name,
            artist: ExternalMediaId::new(album.artists[0].to_string()),
            tracks: album
                .discs
                .into_iter()
                .flat_map(|disc| disc.tracks)
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            // TODO: set spotify id
            properties: Default::default(),
        })
    }
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        tracing::debug!("fetching track: {}", id);
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
        Ok(ExternalTrack {
            name: track.name,
            artist: ExternalMediaId::new(track.artists[0].to_string()),
            album: ExternalMediaId::new(track.album.to_string()),
            // TODO: set spotify id
            properties: Default::default(),
        })
    }
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        todo!()
    }
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        tracing::debug!("downloading track: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Track {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }
        let track_id = resource_id.id;
        let temp_dir = tempfile::tempdir().map_err(sonar::Error::wrap)?;
        let temp_file_path = temp_dir.path().join("samples");
        let mp3_file_path = temp_dir.path().join("track.mp3");
        let sink = spotdl::download::FileDownloadSink::from_path(&temp_file_path)
            .map_err(sonar::Error::wrap)?;
        spotdl::download::download(&self.session, sink, track_id)
            .await
            .map_err(sonar::Error::wrap)?;
        convert::convert_samples_i16(
            convert::ConvertSamplesSource::Path(&temp_file_path),
            &mp3_file_path,
        )
        .await
        .map_err(sonar::Error::wrap)?;
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

// #[derive(Clone)]
// pub struct Context {
//     context: sonar::Context,
//     session: spotdl::session::Session,
//     fetcher: Arc<dyn spotdl::fetcher::MetadataFetcher>,
//     storage_dir: PathBuf,
// }
//
// impl std::fmt::Debug for Context {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Context").finish()
//     }
// }
//
// impl Context {
//     pub async fn new(
//         context: sonar::Context,
//         credentials: LoginCredentials,
//         storage_dir: PathBuf,
//     ) -> Result<Self> {
//         tokio::fs::create_dir_all(&storage_dir)
//             .await
//             .context("creating storage directory")?;
//
//         let credentials = spotdl::session::login(&credentials)
//             .await
//             .context("logging in to spotify")?;
//
//         let session = spotdl::session::Session::connect(credentials)
//             .await
//             .context("connecting to spotify")?;
//
//         let fetcher = Arc::new(spotdl::fetcher::SpotifyMetadataFetcher::new(
//             session.clone(),
//         ));
//
//         Ok(Self {
//             context,
//             session,
//             fetcher,
//             storage_dir,
//         })
//     }
//
//     pub async fn list(&self) -> Result<Vec<ResourceId>> {
//         let resources = self.read_resources().await?;
//         Ok(resources)
//     }
//
//     pub async fn add(&self, resource_id: ResourceId) -> Result<()> {
//         let mut resources = self.read_resources().await?;
//         if resources.contains(&resource_id) {
//             return Ok(());
//         }
//         resources.push(resource_id);
//         self.write_resources(&resources).await?;
//         tokio::spawn({
//             let ctx = self.clone();
//             async move {
//                 if let Err(err) = download_task(ctx, resource_id).await {
//                     tracing::error!("failed to download {}: {:?}", resource_id, err);
//                 }
//             }
//         });
//         Ok(())
//     }
//
//     pub async fn remove(&self, resource_id: ResourceId) -> Result<()> {
//         let mut resources = self.read_resources().await?;
//         resources.retain(|id| id != &resource_id);
//         self.write_resources(&resources).await?;
//         Ok(())
//     }
//
//     async fn read_resources(&self) -> Result<Vec<ResourceId>> {
//         let resources_path = self.storage_dir.join("resources.json");
//         let resources = match std::fs::read_to_string(&resources_path) {
//             Ok(contents) => serde_json::from_str(&contents)
//                 .map_err(|err| {
//                     eyre::eyre!(
//                         "parsing resources from {}: {}",
//                         resources_path.display(),
//                         err
//                     )
//                 })
//                 .context("reading resources")?,
//             Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
//             Err(err) => {
//                 return Err(eyre::eyre!(
//                     "reading resources from {}: {}",
//                     resources_path.display(),
//                     err
//                 ))
//                 .context("reading resources");
//             }
//         };
//         Ok(resources)
//     }
//
//     async fn write_resources(&self, resources: &[ResourceId]) -> Result<()> {
//         let resources_path = self.storage_dir.join("resources.json");
//         let contents = serde_json::to_string(resources)
//             .map_err(|err| {
//                 eyre::eyre!(
//                     "serializing resources to {}: {}",
//                     resources_path.display(),
//                     err
//                 )
//             })
//             .context("writing resources")?;
//         std::fs::write(&resources_path, contents)
//             .map_err(|err| {
//                 eyre::eyre!("writing resources to {}: {}", resources_path.display(), err)
//             })
//             .context("writing resources")?;
//         Ok(())
//     }
// }
//
// async fn download_task(ctx: Context, resource: ResourceId) -> Result<()> {
//     tracing::info!("downloading {}", resource);
//
//     match resource.resource {
//         Resource::Album => todo!(),
//         Resource::Track => todo!(),
//         _ => todo!(),
//     }
//
//     Ok(())
// }
//
// async fn download_track(
//     ctx: Context,
//     sonar: sonar::Context,
//     album_id: sonar::AlbumId,
//     track_id: SpotifyId,
// ) -> Result<()> {
//     let track_metadata = ctx.fetcher.get_track(track_id).await?;
//     let temp_dir = tempfile::tempdir().context("creating temporary directory")?;
//     let temp_file_path = temp_dir.path().join("samples");
//     let mp3_file_path = temp_dir.path().join("track.mp3");
//     let sink =
//         spotdl::download::FileDownloadSink::from_path(&temp_file_path).context("creating sink")?;
//     spotdl::download::download(&ctx.session, sink, track_id)
//         .await
//         .context("downloading")?;
//     convert::convert_samples_i16(
//         convert::ConvertSamplesSource::Path(&temp_file_path),
//         &mp3_file_path,
//     )
//     .await
//     .context("converting")?;
//
//     todo!()
// }
