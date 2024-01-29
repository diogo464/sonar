//! # Tower Service
//! To create a server, implement the [`OpenSubsonicServer`] trait.
//! Then create a [`tower::Service`] from it using [`OpenSubsonicService::new`].
//! The service can be used with an http server like [`axum`](https://docs.rs/axum).
//!
//! ## Example
//! ```rust
//! use opensubsonic::service::prelude::*;
//!
//! struct MyServer;
//!
//! #[opensubsonic::async_trait]
//! impl OpenSubsonicServer for MyServer {
//!     // implement methods here.  
//!     // unimplemented methods will return an error by default.
//!     // not all methods need to be implemented to have a functional server.
//!     // check the example in 'examples/filesystem-server.rs' for a basic implementation.
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = MyServer;
//!     let service = OpenSubsonicService::new("my-service", "0.0.0", server);
//!     let router = axum::Router::default().nest_service("/", service);
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
//!     # if false { // don't run this line when running tests.
//!     axum::serve(listener, router).await?;
//!     # }
//!     Ok(())
//! }
//! ```
use std::{borrow::Cow, future::Future, pin::Pin, sync::Arc};

use bytes::Bytes;

use crate::{
    common::{ByteStream, Version},
    request::{
        annotation::{Scrobble, SetRating, Star, Unstar},
        bookmark::{CreateBookmark, DeleteBookmark, GetBookmarks, GetPlayQueue, SavePlayQueue},
        browsing::{
            GetAlbum, GetAlbumInfo, GetAlbumInfo2, GetArtist, GetArtistInfo, GetArtistInfo2,
            GetArtists, GetGenres, GetIndexes, GetMusicDirectory, GetMusicFolders, GetSimilarSongs,
            GetSimilarSongs2, GetSong, GetTopSongs, GetVideoInfo, GetVideos,
        },
        chat::{AddChatMessage, GetChatMessages},
        jukebox::JukeboxControl,
        lists::{
            GetAlbumList, GetAlbumList2, GetNowPlaying, GetRandomSongs, GetSongsByGenre,
            GetStarred, GetStarred2,
        },
        playlists::{CreatePlaylist, DeletePlaylist, GetPlaylist, GetPlaylists, UpdatePlaylist},
        podcast::{
            CreatePodcastChannel, DeletePodcastChannel, DeletePodcastEpisode,
            DownloadPodcastEpisode, GetNewestPodcasts, GetPodcasts, RefreshPodcasts,
        },
        radio::{
            CreateInternetRadioStation, DeleteInternetRadioStation, GetInternetRadioStations,
            UpdateInternetRadioStation,
        },
        retrieval::{Download, GetAvatar, GetCaptions, GetCoverArt, GetLyrics, Hls, Stream},
        scan::{GetScanStatus, StartScan},
        search::{Search, Search2, Search3},
        sharing::{CreateShare, DeleteShare, GetShares, UpdateShare},
        system::{GetLicense, Ping},
        user::{ChangePassword, CreateUser, DeleteUser, GetUser, GetUsers, UpdateUser},
        Request,
    },
    response::{
        AlbumInfo, AlbumList, AlbumList2, AlbumWithSongsID3, ArtistInfo, ArtistInfo2,
        ArtistWithAlbumsID3, ArtistsID3, Bookmarks, ChatMessages, Directory, Error, ErrorCode,
        Genres, InternetRadioStations, JukeboxControlResponse, License, Lyrics, MusicFolders,
        NewestPodcasts, NowPlaying, PlayQueue, PlaylistWithSongs, Playlists, Podcasts, Response,
        ResponseBody, ResponseObject, ScanStatus, SearchResult, SearchResult2, SearchResult3,
        Shares, SimilarSongs, SimilarSongs2, Songs, Starred, Starred2, TopSongs, User, Users,
        VideoInfo, Videos,
    },
};

pub mod prelude {
    pub use super::Result;
    pub use crate::common::*;
    pub use crate::request::annotation::*;
    pub use crate::request::bookmark::*;
    pub use crate::request::browsing::*;
    pub use crate::request::chat::*;
    pub use crate::request::jukebox::*;
    pub use crate::request::lists::*;
    pub use crate::request::playlists::*;
    pub use crate::request::podcast::*;
    pub use crate::request::radio::*;
    pub use crate::request::retrieval::*;
    pub use crate::request::scan::*;
    pub use crate::request::search::*;
    pub use crate::request::sharing::*;
    pub use crate::request::system::*;
    pub use crate::request::user::*;
    pub use crate::request::*;
    pub use crate::response::*;
    pub use crate::service::{OpenSubsonicServer, OpenSubsonicService};
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(unused)]
#[async_trait::async_trait]
pub trait OpenSubsonicServer: Send + Sync + 'static {
    async fn add_chat_message(&self, request: Request<AddChatMessage>) -> Result<()> {
        unsupported()
    }
    async fn change_password(&self, request: Request<ChangePassword>) -> Result<()> {
        unsupported()
    }
    async fn create_bookmark(&self, request: Request<CreateBookmark>) -> Result<()> {
        unsupported()
    }
    async fn create_internet_radio_station(
        &self,
        request: Request<CreateInternetRadioStation>,
    ) -> Result<()> {
        unsupported()
    }
    async fn create_playlist(&self, request: Request<CreatePlaylist>) -> Result<PlaylistWithSongs> {
        unsupported()
    }
    async fn create_podcast_channel(&self, request: Request<CreatePodcastChannel>) -> Result<()> {
        unsupported()
    }
    async fn create_share(&self, request: Request<CreateShare>) -> Result<Shares> {
        unsupported()
    }
    async fn create_user(&self, request: Request<CreateUser>) -> Result<()> {
        unsupported()
    }
    async fn delete_bookmark(&self, request: Request<DeleteBookmark>) -> Result<()> {
        unsupported()
    }
    async fn delete_internet_radio_station(
        &self,
        request: Request<DeleteInternetRadioStation>,
    ) -> Result<()> {
        unsupported()
    }
    async fn delete_playlist(&self, request: Request<DeletePlaylist>) -> Result<()> {
        unsupported()
    }
    async fn delete_podcast_channel(&self, request: Request<DeletePodcastChannel>) -> Result<()> {
        unsupported()
    }
    async fn delete_podcast_episode(&self, request: Request<DeletePodcastEpisode>) -> Result<()> {
        unsupported()
    }
    async fn delete_share(&self, request: Request<DeleteShare>) -> Result<()> {
        unsupported()
    }
    async fn delete_user(&self, request: Request<DeleteUser>) -> Result<()> {
        unsupported()
    }
    async fn download(&self, request: Request<Download>) -> Result<ByteStream> {
        unsupported()
    }
    async fn download_podcast_episode(
        &self,
        request: Request<DownloadPodcastEpisode>,
    ) -> Result<ByteStream> {
        unsupported()
    }
    async fn get_album(&self, request: Request<GetAlbum>) -> Result<AlbumWithSongsID3> {
        unsupported()
    }
    async fn get_album_info(&self, request: Request<GetAlbumInfo>) -> Result<AlbumInfo> {
        unsupported()
    }
    async fn get_album_info2(&self, request: Request<GetAlbumInfo2>) -> Result<AlbumInfo> {
        unsupported()
    }
    async fn get_album_list(&self, request: Request<GetAlbumList>) -> Result<AlbumList> {
        unsupported()
    }
    async fn get_album_list2(&self, request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        unsupported()
    }
    async fn get_artist(&self, request: Request<GetArtist>) -> Result<ArtistWithAlbumsID3> {
        unsupported()
    }
    async fn get_artist_info(&self, request: Request<GetArtistInfo>) -> Result<ArtistInfo> {
        unsupported()
    }
    async fn get_artist_info2(&self, request: Request<GetArtistInfo2>) -> Result<ArtistInfo2> {
        unsupported()
    }
    async fn get_artists(&self, request: Request<GetArtists>) -> Result<ArtistsID3> {
        unsupported()
    }
    async fn get_avatar(&self, request: Request<GetAvatar>) -> Result<ByteStream> {
        unsupported()
    }
    async fn get_bookmarks(&self, request: Request<GetBookmarks>) -> Result<Bookmarks> {
        unsupported()
    }
    async fn get_captions(&self, request: Request<GetCaptions>) -> Result<ByteStream> {
        unsupported()
    }
    async fn get_chat_message(&self, request: Request<GetChatMessages>) -> Result<ChatMessages> {
        unsupported()
    }
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<ByteStream> {
        unsupported()
    }
    async fn get_genres(&self, request: Request<GetGenres>) -> Result<Genres> {
        unsupported()
    }
    async fn get_indexes(&self, request: Request<GetIndexes>) -> Result<ArtistsID3> {
        unsupported()
    }
    async fn get_internet_radio_stations(
        &self,
        request: Request<GetInternetRadioStations>,
    ) -> Result<InternetRadioStations> {
        unsupported()
    }
    async fn get_license(&self, request: Request<GetLicense>) -> Result<License> {
        unsupported()
    }
    async fn get_lyrics(&self, request: Request<GetLyrics>) -> Result<Lyrics> {
        unsupported()
    }
    // async fn get_lyrics_by_song_id(&self, request:
    async fn get_music_directory(&self, request: Request<GetMusicDirectory>) -> Result<Directory> {
        unsupported()
    }
    async fn get_music_folders(&self, request: Request<GetMusicFolders>) -> Result<MusicFolders> {
        unsupported()
    }
    async fn get_newest_podcasts(
        &self,
        request: Request<GetNewestPodcasts>,
    ) -> Result<NewestPodcasts> {
        unsupported()
    }
    async fn get_now_playing(&self, request: Request<GetNowPlaying>) -> Result<NowPlaying> {
        unsupported()
    }
    async fn get_playlist(&self, request: Request<GetPlaylist>) -> Result<PlaylistWithSongs> {
        unsupported()
    }
    async fn get_playlists(&self, request: Request<GetPlaylists>) -> Result<Playlists> {
        unsupported()
    }
    async fn get_play_queue(&self, request: Request<GetPlayQueue>) -> Result<PlayQueue> {
        unsupported()
    }
    async fn get_podcasts(&self, request: Request<GetPodcasts>) -> Result<Podcasts> {
        unsupported()
    }
    async fn get_random_songs(&self, request: Request<GetRandomSongs>) -> Result<Songs> {
        unsupported()
    }
    async fn get_scan_status(&self, request: Request<GetScanStatus>) -> Result<ScanStatus> {
        unsupported()
    }
    async fn get_shares(&self, request: Request<GetShares>) -> Result<Shares> {
        unsupported()
    }
    async fn get_similar_songs(&self, request: Request<GetSimilarSongs>) -> Result<SimilarSongs> {
        unsupported()
    }
    async fn get_similar_songs2(
        &self,
        request: Request<GetSimilarSongs2>,
    ) -> Result<SimilarSongs2> {
        unsupported()
    }
    async fn get_song(&self, request: Request<GetSong>) -> Result<Songs> {
        unsupported()
    }
    async fn get_songs_by_genre(&self, request: Request<GetSongsByGenre>) -> Result<Songs> {
        unsupported()
    }
    async fn get_starred(&self, request: Request<GetStarred>) -> Result<Starred> {
        unsupported()
    }
    async fn get_starred2(&self, request: Request<GetStarred2>) -> Result<Starred2> {
        unsupported()
    }
    async fn get_top_songs(&self, request: Request<GetTopSongs>) -> Result<TopSongs> {
        unsupported()
    }
    async fn get_user(&self, request: Request<GetUser>) -> Result<User> {
        unsupported()
    }
    async fn get_users(&self, request: Request<GetUsers>) -> Result<Users> {
        unsupported()
    }
    async fn get_video_info(&self, request: Request<GetVideoInfo>) -> Result<VideoInfo> {
        unsupported()
    }
    async fn get_videos(&self, request: Request<GetVideos>) -> Result<Videos> {
        unsupported()
    }
    async fn hls(&self, request: Request<Hls>) -> Result<ByteStream> {
        unsupported()
    }
    async fn jukebox_control(
        &self,
        request: Request<JukeboxControl>,
    ) -> Result<JukeboxControlResponse> {
        unsupported()
    }
    async fn ping(&self, request: Request<Ping>) -> Result<()> {
        unsupported()
    }
    async fn refresh_podcasts(&self, request: Request<RefreshPodcasts>) -> Result<()> {
        unsupported()
    }
    async fn save_play_queue(&self, request: Request<SavePlayQueue>) -> Result<()> {
        unsupported()
    }
    async fn scrobble(&self, request: Request<Scrobble>) -> Result<()> {
        unsupported()
    }
    async fn search(&self, request: Request<Search>) -> Result<SearchResult> {
        unsupported()
    }
    async fn search2(&self, request: Request<Search2>) -> Result<SearchResult2> {
        unsupported()
    }
    async fn search3(&self, request: Request<Search3>) -> Result<SearchResult3> {
        unsupported()
    }
    async fn set_rating(&self, request: Request<SetRating>) -> Result<()> {
        unsupported()
    }
    async fn star(&self, request: Request<Star>) -> Result<()> {
        unsupported()
    }
    async fn start_scan(&self, request: Request<StartScan>) -> Result<ScanStatus> {
        unsupported()
    }
    async fn stream(&self, request: Request<Stream>) -> Result<ByteStream> {
        unsupported()
    }
    async fn unstar(&self, request: Request<Unstar>) -> Result<()> {
        unsupported()
    }
    async fn update_internet_radio_station(
        &self,
        request: Request<UpdateInternetRadioStation>,
    ) -> Result<()> {
        unsupported()
    }
    async fn update_playlist(&self, request: Request<UpdatePlaylist>) -> Result<()> {
        unsupported()
    }
    async fn update_share(&self, request: Request<UpdateShare>) -> Result<()> {
        unsupported()
    }
    async fn update_user(&self, request: Request<UpdateUser>) -> Result<()> {
        unsupported()
    }
}

type HttpResponse = http::Response<http_body_util::StreamBody<OpenSubsonicBodyStream>>;

pub enum OpenSubsonicBodyStream {
    Empty,
    Bytes(Option<Bytes>),
    ByteStream(ByteStream),
}

impl tokio_stream::Stream for OpenSubsonicBodyStream {
    type Item = std::io::Result<http_body::Frame<Bytes>>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Empty => std::task::Poll::Ready(None),
            Self::Bytes(bytes) => match bytes.take() {
                Some(bytes) => {
                    let frame = http_body::Frame::data(bytes);
                    std::task::Poll::Ready(Some(Ok(frame)))
                }
                None => std::task::Poll::Ready(None),
            },
            Self::ByteStream(stream) => {
                let stream = Pin::new(&mut *stream);
                stream
                    .poll_next(cx)
                    .map(|opt| opt.map(|res| res.map(http_body::Frame::data)))
            }
        }
    }
}

#[derive(Debug)]
pub struct OpenSubsonicService<S> {
    server_version: Cow<'static, str>,
    server_type: Cow<'static, str>,
    server: Arc<S>,
}

impl<S> Clone for OpenSubsonicService<S> {
    fn clone(&self) -> Self {
        Self {
            server_version: self.server_version.clone(),
            server_type: self.server_type.clone(),
            server: self.server.clone(),
        }
    }
}

macro_rules! case {
    ($path:literal) => {
        concat!("/rest/", $path) | concat!("/rest/", $path, ".view")
    };
    // responses that return ()
    ($self:expr, $query:expr,$method:ident) => {{
        let request = $self.parse_query($query)?;
        $self
            .server
            .$method(request)
            .await
            .map_err(|err| $self.response_from_error(err))?;
        Ok($self.response_from_body(None))
    }};
    // responses that return some concrete type
    ($self:expr, $query:expr,$method:ident, $body:ident) => {{
        let request = $self.parse_query($query)?;
        let response_body = $self
            .server
            .$method(request)
            .await
            .map_err(|err| $self.response_from_error(err))?;
        Ok($self.response_from_body(Some(ResponseBody::$body(response_body))))
    }};
    // responses that return a byte stream with mime-type
    ($self:expr, $query:expr,$method:ident ->) => {{
        let request = $self.parse_query($query)?;
        let stream = $self
            .server
            .$method(request)
            .await
            .map_err(|err| $self.response_from_error(err))?;
        Ok($self.response_from_byte_stream(stream))
    }};
}

impl<S> OpenSubsonicService<S> {
    pub fn new(
        server_version: impl Into<Cow<'static, str>>,
        server_type: impl Into<Cow<'static, str>>,
        server: S,
    ) -> Self {
        Self {
            server_version: server_version.into(),
            server_type: server_type.into(),
            server: Arc::new(server),
        }
    }
}

impl<S> OpenSubsonicService<S>
where
    S: OpenSubsonicServer + Send + Sync + 'static,
{
    const VERSION: Version = Version::V1_16_1;

    async fn handle_request(&self, path: &str, query: &str) -> Result<HttpResponse, HttpResponse> {
        tracing::debug!("path: {}", path);
        tracing::debug!("query: {}", query);
        match path {
            case!("addChatMessage") => case!(self, query, add_chat_message),
            case!("changePassword") => case!(self, query, change_password),
            case!("createBookmark") => case!(self, query, create_bookmark),
            case!("createInternetRadioStation") => {
                case!(self, query, create_internet_radio_station)
            }
            case!("createPlaylist") => case!(self, query, create_playlist, Playlist),
            case!("createPodcastChannel") => case!(self, query, create_podcast_channel),
            case!("createShare") => case!(self, query, create_share, Shares),
            case!("createUser") => case!(self, query, create_user),
            case!("deleteBookmark") => case!(self, query, delete_bookmark),
            case!("deleteInternetRadioStation") => {
                case!(self, query, delete_internet_radio_station)
            }
            case!("deletePlaylist") => case!(self, query, delete_playlist),
            case!("deletePodcastChannel") => case!(self, query, delete_podcast_channel),
            case!("deletePodcastEpisode") => case!(self, query, delete_podcast_episode),
            case!("deleteShare") => case!(self, query, delete_share),
            case!("deleteUser") => case!(self, query, delete_user),
            case!("download") => case!(self, query, download ->),
            case!("downloadPodcastEpisode") => case!(self, query, download_podcast_episode ->),
            case!("getAlbum") => case!(self, query, get_album, Album),
            case!("getAlbumInfo") => case!(self, query, get_album_info, AlbumInfo),
            case!("getAlbumInfo2") => case!(self, query, get_album_info2, AlbumInfo),
            case!("getAlbumList") => case!(self, query, get_album_list, AlbumList),
            case!("getAlbumList2") => case!(self, query, get_album_list2, AlbumList2),
            case!("getArtist") => case!(self, query, get_artist, Artist),
            case!("getArtistInfo") => case!(self, query, get_artist_info),
            case!("getArtistInfo2") => case!(self, query, get_artist_info2),
            case!("getArtists") => case!(self, query, get_artists, Artists),
            case!("getAvatar") => case!(self, query, get_avatar ->),
            case!("getBookmarks") => case!(self, query, get_bookmarks, Bookmarks),
            case!("getCaptions") => case!(self, query, get_captions ->),
            case!("getChatMessages") => case!(self, query, get_chat_message, ChatMessages),
            case!("getCoverArt") => case!(self, query, get_cover_art ->),
            case!("getGenres") => case!(self, query, get_genres, Genres),
            case!("getIndexes") => case!(self, query, get_indexes, Indexes),
            case!("getInternetRadioStations") => {
                case!(
                    self,
                    query,
                    get_internet_radio_stations,
                    InternetRadioStations
                )
            }
            case!("getLicense") => case!(self, query, get_license),
            case!("getLyrics") => case!(self, query, get_lyrics),
            case!("getMusicDirectory") => case!(self, query, get_music_directory, Directory),
            case!("getMusicFolders") => case!(self, query, get_music_folders, MusicFolders),
            case!("getNewestPodcasts") => case!(self, query, get_newest_podcasts, NewestPodcasts),
            case!("getNowPlaying") => case!(self, query, get_now_playing, NowPlaying),
            case!("getPlaylist") => case!(self, query, get_playlist, Playlist),
            case!("getPlaylists") => case!(self, query, get_playlists, Playlists),
            case!("getPlayQueue") => case!(self, query, get_play_queue, PlayQueue),
            case!("getPodcasts") => case!(self, query, get_podcasts, Podcasts),
            case!("getRandomSongs") => case!(self, query, get_random_songs, RandomSongs),
            case!("getScanStatus") => case!(self, query, get_scan_status, ScanStatus),
            case!("getShares") => case!(self, query, get_shares, Shares),
            case!("getSimilarSongs") => case!(self, query, get_similar_songs, SimilarSongs),
            case!("getSimilarSongs2") => case!(self, query, get_similar_songs2, SimilarSongs2),
            case!("getStarred") => case!(self, query, get_starred, Starred),
            case!("getStarred2") => case!(self, query, get_starred2, Starred2),
            case!("getTopSongs") => case!(self, query, get_top_songs, TopSongs),
            case!("getUser") => case!(self, query, get_user, User),
            case!("getUsers") => case!(self, query, get_users, Users),
            case!("getVideoInfo") => case!(self, query, get_video_info, VideoInfo),
            case!("getVideos") => case!(self, query, get_videos, Videos),
            case!("hls") => case!(self, query, hls ->),
            case!("jukeboxControl") => case!(self, query, jukebox_control, JukeboxControlResponse),
            case!("ping") => case!(self, query, ping),
            case!("refreshPodcasts") => case!(self, query, refresh_podcasts),
            case!("savePlayQueue") => case!(self, query, save_play_queue),
            case!("scrobble") => case!(self, query, scrobble),
            case!("search") => case!(self, query, search, SearchResult),
            case!("search2") => case!(self, query, search2, SearchResult2),
            case!("search3") => case!(self, query, search3, SearchResult3),
            case!("setRating") => case!(self, query, set_rating),
            case!("star") => case!(self, query, star),
            case!("startScan") => case!(self, query, start_scan, ScanStatus),
            case!("stream") => case!(self, query, stream ->),
            case!("unstar") => case!(self, query, unstar),
            case!("updateInternetRadioStation") => {
                case!(self, query, update_internet_radio_station)
            }
            case!("updatePlaylist") => case!(self, query, update_playlist),
            case!("updateShare") => case!(self, query, update_share),
            case!("updateUser") => case!(self, query, update_user),
            _ => Err(self.response_not_found()),
        }
    }

    fn parse_query<Q>(&self, query: &str) -> Result<Q, HttpResponse>
    where
        Q: crate::query::FromQuery,
    {
        match crate::query::from_query(query) {
            Ok(request) => Ok(request),
            Err(err) => {
                Err(self
                    .response_from_error(Error::with_message(ErrorCode::Generic, err.to_string())))
            }
        }
    }

    fn response_from_body(&self, body: Option<ResponseBody>) -> HttpResponse {
        self.response_from_response(
            match body {
                Some(body) => Response::ok(
                    Self::VERSION,
                    body,
                    self.server_type.clone(),
                    self.server_version.clone(),
                ),
                None => Response::ok_empty(
                    Self::VERSION,
                    self.server_type.clone(),
                    self.server_version.clone(),
                ),
            },
            http::StatusCode::OK,
        )
    }

    fn response_from_byte_stream(&self, stream: ByteStream) -> HttpResponse {
        let mime_type = match stream.mime_type().map(|m| m.parse()) {
            Some(Ok(mime_type)) => mime_type,
            Some(Err(err)) => {
                tracing::warn!("failed to parse mime type: {}", err);
                http::HeaderValue::from_static("application/octet-stream")
            }
            None => http::HeaderValue::from_static("application/octet-stream"),
        };
        let mut response = http::Response::new(http_body_util::StreamBody::new(
            OpenSubsonicBodyStream::ByteStream(stream),
        ));
        *response.status_mut() = http::StatusCode::OK;
        response
            .headers_mut()
            .insert(http::header::CONTENT_TYPE, mime_type);
        response
    }

    fn response_from_error(&self, error: Error) -> HttpResponse {
        self.response_from_response(
            Response::failed(
                Self::VERSION,
                error,
                self.server_type.clone(),
                self.server_version.clone(),
            ),
            http::StatusCode::BAD_REQUEST,
        )
    }

    fn response_from_response(&self, response: Response, status: http::StatusCode) -> HttpResponse {
        let serialized = serde_json::to_vec(&ResponseObject::from(response))
            .expect("failed to serialize response");
        let mut response = http::Response::new(http_body_util::StreamBody::new(
            OpenSubsonicBodyStream::Bytes(Some(From::from(serialized))),
        ));
        *response.status_mut() = status;
        response
    }

    fn response_not_found(&self) -> HttpResponse {
        let mut response = http::Response::new(http_body_util::StreamBody::new(
            OpenSubsonicBodyStream::Empty,
        ));
        *response.status_mut() = http::StatusCode::NOT_FOUND;
        response
    }
}

impl<S, B> tower::Service<http::Request<B>> for OpenSubsonicService<S>
where
    S: OpenSubsonicServer + Send + Sync + 'static,
{
    type Response = HttpResponse;

    type Error = std::convert::Infallible;

    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::prelude::v1::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let svc = self.clone();
        let path = req.uri().path().to_string();
        let query = req.uri().query().unwrap_or_default().to_string();

        Box::pin(async move {
            Ok(match svc.handle_request(&path, &query).await {
                Ok(response) => response,
                Err(response) => response,
            })
        })
    }
}

fn unsupported<T>() -> Result<T> {
    Err(Error::with_message(
        ErrorCode::Generic,
        "unsupported method",
    ))
}
