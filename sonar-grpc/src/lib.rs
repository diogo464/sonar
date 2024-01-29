use std::net::SocketAddr;

use bytes::Bytes;
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
    async fn user_list(
        &self,
        request: tonic::Request<UserListRequest>,
    ) -> std::result::Result<tonic::Response<UserListResponse>, tonic::Status> {
        let request = request.into_inner();
        let params = sonar::ListParams::from((request.offset, request.count));
        let users = sonar::user_list(&self.context, params).await.m()?;
        let users = users.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(UserListResponse { users }))
    }
    async fn user_create(
        &self,
        request: tonic::Request<UserCreateRequest>,
    ) -> std::result::Result<tonic::Response<User>, tonic::Status> {
        let req = request.into_inner();
        let username = req.username.parse::<sonar::Username>().m()?;
        let avatar = req.avatar.map(sonar::ImageId::try_from).transpose().m()?;
        let user = sonar::user_create(
            &self.context,
            sonar::UserCreate {
                username,
                password: req.password,
                avatar,
            },
        )
        .await
        .m()?;
        Ok(tonic::Response::new(user.into()))
    }
    async fn user_update(
        &self,
        request: tonic::Request<UserUpdateRequest>,
    ) -> std::result::Result<tonic::Response<User>, tonic::Status> {
        todo!()
    }
    async fn user_delete(
        &self,
        request: tonic::Request<UserDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let req = request.into_inner();
        let user_id = sonar::UserId::try_from(req.user_id).m()?;
        sonar::user_delete(&self.context, user_id).await.m()?;
        Ok(tonic::Response::new(()))
    }
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
                description: None,
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
    async fn import(
        &self,
        request: tonic::Request<tonic::Streaming<ImportRequest>>,
    ) -> std::result::Result<tonic::Response<Track>, tonic::Status> {
        let mut stream = request.into_inner();
        let first_message = match stream.message().await? {
            Some(message) => message,
            None => return Err(tonic::Status::invalid_argument("empty stream")),
        };
        let filepath = first_message.filepath;
        let artist = first_message
            .artist_id
            .map(sonar::ArtistId::try_from)
            .transpose()
            .m()?;
        let album = first_message
            .album_id
            .map(sonar::AlbumId::try_from)
            .transpose()
            .m()?;
        let track = sonar::import(
            &self.context,
            sonar::Import {
                artist,
                album,
                filepath,
                stream: Box::new(ImportStream {
                    first_chunk: Some(Bytes::from(first_message.chunk)),
                    stream,
                }),
            },
        )
        .await
        .m()?;
        Ok(tonic::Response::new(track.into()))
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
            sonar::ErrorKind::Unauthorized => tonic::Status::unauthenticated(e.to_string()),
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

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidUsernameError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl From<sonar::User> for User {
    fn from(value: sonar::User) -> Self {
        Self {
            user_id: From::from(value.id),
            username: From::from(value.username),
            avatar: value.avatar.map(From::from),
        }
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

impl From<sonar::Track> for Track {
    fn from(value: sonar::Track) -> Self {
        Self {
            id: From::from(value.id),
            name: value.name,
            artist_id: From::from(value.artist),
            album_id: From::from(value.album),
            disc_number: value.disc_number as u32,
            track_number: value.track_number as u32,
            duration: Some(TryFrom::try_from(value.duration).expect("failed to convert duration")),
            listen_count: value.listen_count as u32,
            cover_art_id: value.cover_art.map(From::from),
            genres: From::from(value.genres),
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

struct ImportStream {
    first_chunk: Option<Bytes>,
    stream: tonic::Streaming<ImportRequest>,
}

impl tokio_stream::Stream for ImportStream {
    type Item = std::io::Result<Bytes>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.first_chunk.take() {
            Some(chunk) => std::task::Poll::Ready(Some(Ok(chunk))),
            None => {
                let stream = std::pin::Pin::new(&mut self.get_mut().stream);
                match stream.poll_next(cx) {
                    std::task::Poll::Ready(Some(Ok(message))) => {
                        std::task::Poll::Ready(Some(Ok(Bytes::from(message.chunk))))
                    }
                    std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                    ))),
                    std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }
    }
}
