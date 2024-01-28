use std::net::SocketAddr;

use eyre::Context;

tonic::include_proto!("sonar");

pub type Client = sonar_service_client::SonarServiceClient<tonic::transport::Channel>;

#[derive(Clone)]
struct Server {
    context: sonar::Context,
}

impl Server {
    fn new(context: sonar::Context) -> Self {
        Self { context }
    }
}

#[tonic::async_trait]
impl sonar_service_server::SonarService for Server {
    async fn image_create(
        &self,
        request: tonic::Request<ImageCreateRequest>,
    ) -> std::result::Result<tonic::Response<ImageCreateResponse>, tonic::Status> {
        let req = request.into_inner();
        let image_id = sonar::image_create(
            &self.context,
            sonar::ImageCreate {
                data: sonar::bytestream::from_bytes(req.content),
            },
        )
        .await
        .m()?;
        Ok(tonic::Response::new(ImageCreateResponse {
            image_id: From::from(image_id),
        }))
    }
    async fn image_delete(
        &self,
        request: tonic::Request<ImageDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }
    async fn image_download(
        &self,
        request: tonic::Request<ImageDownloadRequest>,
    ) -> std::result::Result<tonic::Response<ImageDownloadResponse>, tonic::Status> {
        let req = request.into_inner();
        let image_id = sonar::ImageId::try_from(req.image_id).m()?;
        let image_download = sonar::image_download(&self.context, image_id).await.m()?;
        let content = sonar::bytestream::to_bytes(image_download).await?;
        Ok(tonic::Response::new(ImageDownloadResponse {
            image_id: From::from(image_id),
            content: content.to_vec(),
        }))
    }
    async fn artist_list(
        &self,
        request: tonic::Request<ArtistListRequest>,
    ) -> std::result::Result<tonic::Response<ArtistListResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = sonar::ListParams::from((req.offset, req.count));
        let artists = sonar::artist_list(&self.context, params).await.m()?;
        let artists = artists.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(ArtistListResponse { artists }))
    }
    async fn artist_create(
        &self,
        request: tonic::Request<ArtistCreateRequest>,
    ) -> std::result::Result<tonic::Response<Artist>, tonic::Status> {
        let req = request.into_inner();
        let artist = sonar::artist_create(
            &self.context,
            sonar::ArtistCreate {
                name: req.name,
                cover_art: req.coverart.map(sonar::ImageId::try_from).transpose().m()?,
                genres: sonar::Genres::try_from(req.genres).m()?,
                properties: convert_properties_from_pb(req.properties)?,
            },
        )
        .await
        .m()?;
        Ok(tonic::Response::new(artist.into()))
    }
    async fn artist_update(
        &self,
        request: tonic::Request<ArtistUpdateRequest>,
    ) -> std::result::Result<tonic::Response<Artist>, tonic::Status> {
        todo!()
    }
    async fn artist_delete(
        &self,
        request: tonic::Request<ArtistDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }
}

pub async fn client(endpoint: &str) -> eyre::Result<Client> {
    tracing::info!("connecting to grpc server on {}", endpoint);
    Ok(
        sonar_service_client::SonarServiceClient::connect(endpoint.to_string())
            .await
            .context("connecting to grpc server")?,
    )
}

pub async fn start_server(context: sonar::Context, address: SocketAddr) -> eyre::Result<()> {
    tracing::info!("starting grpc server on {}", address);
    tonic::transport::Server::builder()
        .add_service(sonar_service_server::SonarServiceServer::new(Server::new(
            context,
        )))
        .serve(address)
        .await?;
    Ok(())
}

trait ResultExt<T> {
    fn m(self) -> Result<T, tonic::Status>;
}

impl<T> ResultExt<T> for sonar::Result<T> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| match e.kind() {
            sonar::ErrorKind::NotFound => tonic::Status::not_found(e.to_string()),
            sonar::ErrorKind::Invalid => tonic::Status::invalid_argument(e.to_string()),
            sonar::ErrorKind::Internal => tonic::Status::internal(e.to_string()),
        })
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidIdError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidPropertyKeyError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidPropertyValueError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidGenreError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl From<sonar::Artist> for Artist {
    fn from(value: sonar::Artist) -> Self {
        Self {
            id: From::from(value.id),
            name: value.name,
            album_count: value.album_count as u32,
            listen_count: value.listen_count as u32,
            genres: From::from(value.genres),
            coverart: value.cover_art.map(From::from),
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

fn convert_properties_to_pb(properties: sonar::Properties) -> Vec<Property> {
    let mut props = Vec::with_capacity(properties.len());
    for (key, value) in properties {
        props.push(Property {
            key: From::from(key),
            value: From::from(value),
        });
    }
    props
}

fn convert_properties_from_pb(
    properties: Vec<Property>,
) -> Result<sonar::Properties, tonic::Status> {
    let mut props = sonar::Properties::new();
    for prop in properties {
        let key = prop.key.parse::<sonar::PropertyKey>().m()?;
        let value = prop.value.parse::<sonar::PropertyValue>().m()?;
        props.insert(key, value);
    }
    Ok(props)
}
