use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tokio::net::TcpListener;

use crate::{
    common::Version,
    request::{
        annotation::{Scrobble, SetRating, Star, Unstar},
        bookmark::{CreateBookmark, DeleteBookmark, GetBookmarks, GetPlayQueue, SavePlayQueue},
        browsing::{
            GetAlbum, GetAlbumInfo, GetAlbumInfo2, GetArtist, GetArtistInfo, GetArtistInfo2,
            GetArtists, GetGenres, GetIndexes, GetMusicDirectory, GetMusicFolders, GetSimilarSongs,
            GetSimilarSongs2, GetTopSongs, GetVideoInfo, GetVideos,
        },
        chat::{AddChatMessage, GetChatMessages},
        jukebox::JukeboxControl,
        lists::{
            GetAlbumList, GetAlbumList2, GetNowPlaying, GetRandomSongs, GetStarred, GetStarred2,
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
        retrieval::{Download, GetAvatar, GetCaptions, GetCoverArt, GetLyrics, Hls},
        scan::{GetScanStatus, StartScan},
        search::{Search, Search2, Search3},
        sharing::{CreateShare, DeleteShare, GetShares, UpdateShare},
        system::{GetLicense, Ping},
        user::{ChangePassword, CreateUser, DeleteUser, GetUser, GetUsers, UpdateUser},
        Request, SubsonicRequest,
    },
    response::{
        AlbumInfo, AlbumList, AlbumList2, AlbumWithSongsID3, Artist, ArtistInfo, ArtistInfo2,
        ArtistsID3, Bookmarks, ChatMessages, Directory, Error, ErrorCode, Genres, Indexes,
        InternetRadioStations, JukeboxControlResponse, License, Lyrics, MusicFolders,
        NewestPodcasts, NowPlaying, PlayQueue, PlaylistWithSongs, Playlists, Podcasts, Response,
        ResponseBody, ResponseObject, ScanStatus, SearchResult, SearchResult2, SearchResult3,
        Shares, SimilarSongs, SimilarSongs2, Songs, Starred, Starred2, TopSongs, User, Users,
        VideoInfo, Videos,
    },
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, self.to_string()).into_response()
    }
}

pub struct ByteStream;

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
    async fn get_artist(&self, request: Request<GetArtist>) -> Result<Artist> {
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
    async fn get_indexes(&self, request: Request<GetIndexes>) -> Result<Indexes> {
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
    // TODO
    // async fn get_song(&self, request: Request<GetSong>) -> Result<Songs>{unsupported()}
    // async fn get_songs_by_genre(&self, request:Request<GetSongsByGenre>) ->
    // Result<Songs>{unsupported()}
    async fn get_starreed(&self, request: Request<GetStarred>) -> Result<Starred> {
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
    // TODO: stream
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

fn unsupported<T>() -> Result<T> {
    Err(Error::with_message(
        ErrorCode::Generic,
        "unsupported method",
    ))
}

#[axum::async_trait]
impl<S, T> FromRequestParts<S> for Request<T>
where
    T: SubsonicRequest,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let query = match parts.uri.query() {
            Some(query) => query,
            None => return Err((StatusCode::BAD_REQUEST, "missing query").into_response()),
        };
        let request = match Self::from_query(query) {
            Ok(request) => request,
            Err(err) => return Err((StatusCode::BAD_REQUEST, err.to_string()).into_response()),
        };
        Ok(request)
    }
}

fn wrap_body(body: Option<ResponseBody>) -> Result<Json<ResponseObject>> {
    const SERVER_VERSION: &str = "1.0.1";
    const SERVER_TYPE: &str = "mads";
    let response = match body {
        Some(body) => Response::ok(Version::LATEST, body, SERVER_TYPE, SERVER_VERSION),
        None => Response::ok_empty(Version::LATEST, SERVER_TYPE, SERVER_VERSION),
    };
    Ok(Json(ResponseObject::from(response)))
}

macro_rules! route {
    ($name:ident, $req:ty) => {
        async fn $name(
            State(state): State<Arc<dyn OpenSubsonicServer>>,
            request: Request<$req>,
        ) -> Result<Json<ResponseObject>> {
            tracing::trace!("request: {:#?}", request);
            #[allow(unused)]
            let result = match state.$name(request).await {
                Ok(result) => result,
                Err(err) => {
                    tracing::warn!("error: {:#?}", err);
                    return Err(err);
                }
            };
            let response = wrap_body(None);
            tracing::trace!("response: {:#?}", response);
            response
        }
    };
    ($name:ident, $req:ty, $body:tt) => {
        async fn $name(
            State(state): State<Arc<dyn OpenSubsonicServer>>,
            request: Request<$req>,
        ) -> Result<Json<ResponseObject>> {
            tracing::trace!("request: {:#?}", request);
            #[allow(unused)]
            let result = match state.$name(request).await {
                Ok(result) => result,
                Err(err) => {
                    tracing::warn!("error: {:#?}", err);
                    return Err(err);
                }
            };
            let response = wrap_body(Some(ResponseBody::$body(result)));
            tracing::trace!("response: {:#?}", response);
            response
        }
    };
    ($router:expr => $name:ident, $req:ty) => {{
        route!($name, $req);
        route!(@ $router => $name, $req)
    }};
    ($router:expr => $name:ident, $req:ty, $body:tt) => {{
        route!($name, $req, $body);
        route!(@ $router => $name, $req)
    }};
    (@ $router:expr => $handler:ident, $req:ty) => {{
        $router
            .route(<$req as crate::request::SubsonicRequest>::PATH, get($handler))
            .route(
                &format!("{}.view", <$req as crate::request::SubsonicRequest>::PATH),
                get($handler),
            )
    }};
}

pub async fn serve(address: SocketAddr, server: impl OpenSubsonicServer) -> std::io::Result<()> {
    let mut router = Router::default();
    router = route!(router => add_chat_message, AddChatMessage);
    router = route!(router => change_password, ChangePassword);
    router = route!(router => create_bookmark, CreateBookmark);
    router = route!(
        router =>
        create_internet_radio_station,
        CreateInternetRadioStation
    );
    router = route!(router => create_playlist, CreatePlaylist, Playlist);
    router = route!(router => create_podcast_channel, CreatePodcastChannel);
    router = route!(router => create_share, CreateShare, Shares);
    router = route!(router => create_user, CreateUser);
    router = route!(router => delete_bookmark, DeleteBookmark);
    router = route!(
        router =>
        delete_internet_radio_station,
        DeleteInternetRadioStation
    );
    router = route!(router => delete_playlist, DeletePlaylist);
    router = route!(router => delete_podcast_channel, DeletePodcastChannel);
    router = route!(router => delete_podcast_episode, DeletePodcastEpisode);
    router = route!(router => delete_share, DeleteShare);
    router = route!(router => delete_user, DeleteUser);
    router = route!(router => download, Download); // TODO: fix
    router = route!(
        router =>
        download_podcast_episode,
        DownloadPodcastEpisode
    ); // TODO: fix
    router = route!(router => get_album, GetAlbum, Album);
    router = route!(router => get_album_info, GetAlbumInfo, AlbumInfo);
    router = route!(router => get_album_info2, GetAlbumInfo2, AlbumInfo);
    router = route!(router => get_album_list, GetAlbumList, AlbumList);
    router = route!(router => get_album_list2, GetAlbumList2, AlbumList2);
    router = route!(router => get_artist, GetArtist, Artist);
    router = route!(router => get_artist_info, GetArtistInfo, ArtistInfo);
    router = route!(router => get_artist_info2, GetArtistInfo2, ArtistInfo2);
    router = route!(router => get_artists, GetArtists, Artists);
    router = route!(router => get_avatar, GetAvatar); // TODO: fix
    router = route!(router => get_bookmarks, GetBookmarks, Bookmarks);
    router = route!(router => get_captions, GetCaptions); // TODO: fix
    router = route!(router => get_chat_message, GetChatMessages, ChatMessages);
    router = route!(router => get_cover_art, GetCoverArt); // TODO: fix
    router = route!(router => get_genres, GetGenres, Genres);
    router = route!(router => get_indexes, GetIndexes, Indexes);
    router = route!(router =>
        get_internet_radio_stations,
        GetInternetRadioStations,
        InternetRadioStations
    );
    router = route!(router => get_license, GetLicense, License);
    router = route!(router => get_lyrics, GetLyrics, Lyrics);
    router = route!(router => get_music_directory, GetMusicDirectory, Directory);
    router = route!(router => get_music_folders, GetMusicFolders, MusicFolders);
    router = route!(router => get_newest_podcasts, GetNewestPodcasts, NewestPodcasts);
    router = route!(router => get_now_playing, GetNowPlaying, NowPlaying);
    router = route!(router => get_playlist, GetPlaylist, Playlist);
    router = route!(router => get_playlists, GetPlaylists, Playlists);
    router = route!(router => get_play_queue, GetPlayQueue, PlayQueue);
    router = route!(router => get_podcasts, GetPodcasts, Podcasts);
    router = route!(router => get_random_songs, GetRandomSongs, RandomSongs);
    router = route!(router => get_scan_status, GetScanStatus, ScanStatus);
    router = route!(router => get_shares, GetShares, Shares);
    router = route!(router => get_similar_songs, GetSimilarSongs, SimilarSongs);
    router = route!(router => get_similar_songs2, GetSimilarSongs2, SimilarSongs2);
    router = route!(router => get_starreed, GetStarred, Starred);
    router = route!(router => get_starred2, GetStarred2, Starred2);
    router = route!(router => get_top_songs, GetTopSongs, TopSongs);
    router = route!(router => get_user, GetUser, User);
    router = route!(router => get_users, GetUsers, Users);
    router = route!(router => get_video_info, GetVideoInfo, VideoInfo);
    router = route!(router => get_videos, GetVideos, Videos);
    router = route!(router => hls, Hls); // TODO: fix
    router = route!(router => jukebox_control, JukeboxControl, JukeboxControlResponse);
    router = route!(router => ping, Ping);
    router = route!(router => refresh_podcasts, RefreshPodcasts);
    router = route!(router => save_play_queue, SavePlayQueue);
    router = route!(router => scrobble, Scrobble);
    router = route!(router => search, Search, SearchResult);
    router = route!(router => search2, Search2, SearchResult2);
    router = route!(router => search3, Search3, SearchResult3);
    router = route!(router => set_rating, SetRating);
    router = route!(router => star, Star);
    router = route!(router => start_scan, StartScan, ScanStatus);
    router = route!(router => unstar, Unstar);
    router = route!(router => update_internet_radio_station, UpdateInternetRadioStation);
    router = route!(router => update_playlist, UpdatePlaylist);
    router = route!(router => update_share, UpdateShare);
    router = route!(router => update_user, UpdateUser);

    router = router.layer(tower_http::trace::TraceLayer::new_for_http());
    let router = router.with_state(Arc::new(server));

    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
