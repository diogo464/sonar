use std::{collections::HashMap, net::SocketAddr};

use eyre::Context;
use opensubsonic::service::prelude::*;
use tower_http::cors::Any;

struct Server {
    context: sonar::Context,
}

impl Server {
    fn new(context: sonar::Context) -> Self {
        Self { context }
    }
}

#[opensubsonic::async_trait]
impl OpenSubsonicServer for Server {
    async fn ping(&self, _request: Request<Ping>) -> Result<()> {
        Ok(())
    }
    async fn get_artists(&self, _request: Request<GetArtists>) -> Result<ArtistsID3> {
        let artists = sonar::artist_list(&self.context, Default::default())
            .await
            .m()?;

        let mut index: HashMap<char, Vec<ArtistID3>> = HashMap::new();
        for artist in artists {
            index
                .entry(artist.name.chars().next().unwrap_or('#'))
                .or_default()
                .push(artistid3_from_artist(artist));
        }

        Ok(ArtistsID3 {
            index: index
                .into_iter()
                .map(|(key, value)| IndexID3 {
                    name: key.to_string(),
                    artist: value,
                })
                .collect(),
            ignored_articles: Default::default(),
        })
    }
    async fn get_album_list2(&self, _request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        Ok(Default::default())
    }
    async fn get_starred2(&self, _request: Request<GetStarred2>) -> Result<Starred2> {
        Ok(Default::default())
    }
    async fn get_playlists(&self, _request: Request<GetPlaylists>) -> Result<Playlists> {
        Ok(Default::default())
    }
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<ByteStream> {
        let image_id = request.body.id.parse::<sonar::ImageId>().m()?;
        let download = sonar::image_download(&self.context, image_id).await.m()?;
        Ok(opensubsonic::common::ByteStream::new("image/png", download))
    }
}

fn artistid3_from_artist(artist: sonar::Artist) -> ArtistID3 {
    ArtistID3 {
        id: artist.id.to_string(),
        name: artist.name,
        cover_art: artist.cover_art.map(|id| id.to_string()),
        artist_image_url: None,
        album_count: artist.album_count,
        starred: None,
    }
}

pub async fn start_server(address: SocketAddr, context: sonar::Context) -> eyre::Result<()> {
    tracing::info!("starting opensubsonic server on {}", address);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("creating tcp listener")?;
    let cors = tower_http::cors::CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        // allow requests from any origin
        .allow_origin(Any);
    let service = OpenSubsonicService::new("0.0.0", "sonar", Server::new(context));
    let router = axum::Router::default()
        .nest_service("/", service)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);
    axum::serve(listener, router)
        .await
        .context("running opensubsonic http server")?;
    Ok(())
}

trait ResultExt<T> {
    fn m(self) -> Result<T, opensubsonic::response::Error>;
}

impl<T> ResultExt<T> for sonar::Result<T> {
    fn m(self) -> Result<T, opensubsonic::response::Error> {
        self.map_err(|err| {
            opensubsonic::response::Error::with_message(
                opensubsonic::response::ErrorCode::Generic,
                err.to_string(),
            )
        })
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidIdError> {
    fn m(self) -> Result<T, opensubsonic::response::Error> {
        self.map_err(|err| {
            opensubsonic::response::Error::with_message(
                opensubsonic::response::ErrorCode::Generic,
                err.to_string(),
            )
        })
    }
}
