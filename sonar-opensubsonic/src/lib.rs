use std::{collections::HashMap, net::SocketAddr};

use eyre::Context;
use opensubsonic::service::prelude::*;
use tower_http::cors::Any;

const FAVORITES_PLAYLIST_NAME: &str = "favorites";

struct Server {
    context: sonar::Context,
}

impl Server {
    fn new(context: sonar::Context) -> Self {
        Self { context }
    }

    async fn authenticate<R: SubsonicRequest>(
        &self,
        request: &Request<R>,
    ) -> Result<sonar::UserId> {
        let username = request.username.parse::<sonar::Username>().m()?;
        let password = match &request.authentication {
            Authentication::Password(password) => password,
            _ => {
                return Err(opensubsonic::response::Error::with_message(
                    opensubsonic::response::ErrorCode::TokenAuthenticationNotSupported,
                    "password authentication required".to_string(),
                ))
            }
        };
        sonar::user_authenticate(&self.context, &username, &password)
            .await
            .m()
    }
}

#[opensubsonic::async_trait]
impl OpenSubsonicServer for Server {
    async fn ping(&self, request: Request<Ping>) -> Result<()> {
        let _user_id = self.authenticate(&request).await?;
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
    async fn get_artist_info2(&self, _request: Request<GetArtistInfo2>) -> Result<ArtistInfo2> {
        Ok(ArtistInfo2 {
            info: ArtistInfoBase {
                biography: "this is a biography\n".to_string(),
                music_brainz_id: "this is a music brainz id".to_string(),
                last_fm_url: "this is a last fm url".to_string(),
                ..Default::default()
            },
            similar_artist: Default::default(),
        })
    }
    async fn get_artist(&self, request: Request<GetArtist>) -> Result<ArtistWithAlbumsID3> {
        let artist_id = request.body.id.parse::<sonar::ArtistId>().m()?;
        let artist = sonar::artist_get(&self.context, artist_id).await.m()?;
        let albums = sonar::album_list_by_artist(&self.context, artist_id, Default::default())
            .await
            .m()?;
        Ok(ArtistWithAlbumsID3 {
            artist: ArtistID3 {
                id: artist.id.to_string(),
                name: artist.name.clone(),
                artist_image_url: None,
                starred: None,
                album_count: artist.album_count,
                cover_art: artist.cover_art.map(|id| id.to_string()),
            },
            album: albums
                .into_iter()
                .map(|album| albumid3_from_album_and_artist(&artist, album))
                .collect(),
        })
    }
    async fn get_top_songs(&self, _request: Request<GetTopSongs>) -> Result<TopSongs> {
        Ok(Default::default())
    }
    async fn get_album(&self, request: Request<GetAlbum>) -> Result<AlbumWithSongsID3> {
        let album_id = request.body.id.parse::<sonar::AlbumId>().m()?;
        let album = sonar::album_get(&self.context, album_id).await.m()?;
        let artist = sonar::artist_get(&self.context, album.artist).await.m()?;
        let tracks = sonar::track_list_by_album(&self.context, album_id, Default::default())
            .await
            .m()?;
        let mut song: Vec<Child> = tracks
            .into_iter()
            .map(|track| child_from_track_and_album_and_artist(&artist, &album, track))
            .collect();
        song.sort_by_key(|child| child.track.unwrap_or_default());
        Ok(AlbumWithSongsID3 {
            album: albumid3_from_album_and_artist(&artist, album),
            song,
        })
    }
    async fn get_album_list2(&self, request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        // TODO: implement list types
        let params = sonar::ListParams::from((request.body.offset, request.body.size));
        let albums = sonar::album_list(&self.context, params).await.m()?;
        let albums = sonar::ext::albums_map(albums);
        let artists = sonar::ext::artist_bulk_map(&self.context, albums.values().map(|v| v.artist))
            .await
            .m()?;
        let album = multi_albumid3_from_album_and_artist(&artists, &albums);
        Ok(AlbumList2 { album })
    }
    async fn star(&self, request: Request<Star>) -> Result<()> {
        let user_id = self.authenticate(&request).await?;
        let playlist =
            match sonar::playlist_find_by_name(&self.context, user_id, FAVORITES_PLAYLIST_NAME)
                .await
                .m()?
            {
                Some(playlist) => playlist,
                None => sonar::playlist_create(
                    &self.context,
                    sonar::PlaylistCreate {
                        name: FAVORITES_PLAYLIST_NAME.to_string(),
                        owner: user_id,
                        tracks: Default::default(),
                        properties: Default::default(),
                    },
                )
                .await
                .m()?,
            };
        let track_ids = request
            .body
            .id
            .into_iter()
            .map(|id| id.parse::<sonar::TrackId>().m())
            .collect::<Result<Vec<_>, _>>()?;
        sonar::playlist_insert_tracks(&self.context, playlist.id, &track_ids)
            .await
            .m()?;
        Ok(())
    }
    async fn get_starred2(&self, request: Request<GetStarred2>) -> Result<Starred2> {
        let user_id = self.authenticate(&request).await?;
        let song =
            match sonar::playlist_find_by_name(&self.context, user_id, FAVORITES_PLAYLIST_NAME)
                .await
                .m()?
            {
                Some(playlist) => {
                    let playlist_tracks =
                        sonar::playlist_list_tracks(&self.context, playlist.id, Default::default())
                            .await
                            .m()?;
                    let track_ids = playlist_tracks
                        .iter()
                        .map(|track| track.track)
                        .collect::<Vec<_>>();
                    let tracks = sonar::track_get_bulk(&self.context, &track_ids).await.m()?;
                    let song = child_from_playlist_tracks(&playlist_tracks, &tracks);
                    song
                }
                None => Default::default(),
            };

        Ok(Starred2 {
            song,
            album: Default::default(),
            artist: Default::default(),
        })
    }
    async fn get_playlists(&self, _request: Request<GetPlaylists>) -> Result<Playlists> {
        let playlists = sonar::playlist_list(&self.context, Default::default())
            .await
            .m()?;
        Ok(Playlists {
            playlist: playlists.into_iter().map(playlist_from_playlist).collect(),
        })
    }
    async fn get_playlist(&self, request: Request<GetPlaylist>) -> Result<PlaylistWithSongs> {
        let playlist_id = request.body.id.parse::<sonar::PlaylistId>().m()?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;
        let playlist_tracks =
            sonar::playlist_list_tracks(&self.context, playlist.id, Default::default())
                .await
                .m()?;
        let track_ids = playlist_tracks
            .iter()
            .map(|track| track.track)
            .collect::<Vec<_>>();
        let tracks = sonar::track_get_bulk(&self.context, &track_ids).await.m()?;
        Ok(PlaylistWithSongs {
            playlist: playlist_from_playlist(playlist),
            entry: child_from_playlist_tracks(&playlist_tracks, &tracks),
        })
    }
    async fn create_playlist(&self, request: Request<CreatePlaylist>) -> Result<PlaylistWithSongs> {
        let user_id = self.authenticate(&request).await?;
        let track_ids = request
            .body
            .song_id
            .into_iter()
            .map(|id| id.parse::<sonar::TrackId>().m())
            .collect::<Result<Vec<_>, _>>()?;
        let playlist_id = request
            .body
            .paylist_id
            .map(|id| id.parse::<sonar::PlaylistId>().m())
            .transpose()?;
        let playlist_name = request.body.name;

        let playlist = match (playlist_id, playlist_name) {
            (None, Some(playlist_name)) => sonar::playlist_create(
                &self.context,
                sonar::PlaylistCreate {
                    name: playlist_name,
                    owner: user_id,
                    tracks: track_ids,
                    properties: Default::default(),
                },
            )
            .await
            .m()?,
            // playlist update
            (Some(playlist_id), _) => {
                todo!()
            }
            (None, None) => {
                return Err(opensubsonic::response::Error::with_message(
                    opensubsonic::response::ErrorCode::Generic,
                    "playlist name or id required".to_string(),
                ))
            }
        };

        let playlist_tracks =
            sonar::playlist_list_tracks(&self.context, playlist.id, Default::default())
                .await
                .m()?;
        let track_ids = playlist_tracks
            .iter()
            .map(|track| track.track)
            .collect::<Vec<_>>();
        let tracks = sonar::track_get_bulk(&self.context, &track_ids).await.m()?;
        Ok(PlaylistWithSongs {
            playlist: playlist_from_playlist(playlist),
            entry: child_from_playlist_tracks(&playlist_tracks, &tracks),
        })
    }
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<ByteStream> {
        let image_id = request.body.id.parse::<sonar::ImageId>().m()?;
        let download = sonar::image_download(&self.context, image_id).await.m()?;
        Ok(opensubsonic::common::ByteStream::new(
            download.mime_type,
            download.stream,
        ))
    }
    async fn scrobble(&self, request: Request<Scrobble>) -> Result<()> {
        // TODO: implement
        Ok(())
    }
    async fn stream(&self, request: Request<Stream>) -> Result<ByteStream> {
        let track_id = request.body.id.parse::<sonar::TrackId>().m()?;
        let download = sonar::track_download(&self.context, track_id, Default::default())
            .await
            .m()?;
        Ok(opensubsonic::common::ByteStream::new(
            download.mime_type,
            download.stream,
        ))
    }
    async fn get_music_folders(&self, request: Request<GetMusicFolders>) -> Result<MusicFolders> {
        Ok(Default::default())
    }
    async fn get_podcasts(&self, _request: Request<GetPodcasts>) -> Result<Podcasts> {
        Ok(Default::default())
    }
    async fn get_internet_radio_stations(
        &self,
        _request: Request<GetInternetRadioStations>,
    ) -> Result<InternetRadioStations> {
        Ok(Default::default())
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

fn albumid3_from_album_and_artist(artist: &sonar::Artist, album: sonar::Album) -> AlbumID3 {
    AlbumID3 {
        id: album.id.to_string(),
        name: album.name,
        artist: Some(artist.name.clone()),
        artist_id: Some(artist.id.to_string()),
        cover_art: album.cover_art.map(|id| id.to_string()),
        song_count: album.track_count,
        duration: Default::default(),
        play_count: Some(album.listen_count as u64),
        created: Default::default(),
        starred: None,
        year: None,
        genre: None,
        user_rating: None,
        record_labels: Default::default(),
    }
}

fn multi_albumid3_from_album_and_artist(
    artists: &HashMap<sonar::ArtistId, sonar::Artist>,
    albums: &HashMap<sonar::AlbumId, sonar::Album>,
) -> Vec<AlbumID3> {
    let mut albumid3s = Vec::with_capacity(albums.len());
    for album in albums.values() {
        let artist = &artists[&album.artist];
        albumid3s.push(albumid3_from_album_and_artist(artist, album.clone()));
    }
    albumid3s
}

fn child_from_track_and_album_and_artist(
    artist: &sonar::Artist,
    album: &sonar::Album,
    track: sonar::Track,
) -> Child {
    Child {
        id: track.id.to_string(),
        parent: Some(track.album.to_string()),
        is_dir: false,
        title: track.name,
        album: Some(album.name.clone()),
        artist: Some(artist.name.clone()),
        track: track.properties.get_parsed(sonar::prop::TRACK_NUMBER),
        year: None,
        genre: None,
        cover_art: album.cover_art.map(|id| id.to_string()),
        duration: Some(From::from(track.duration)),
        play_count: Some(track.listen_count as u64),
        disc_number: track.properties.get_parsed(sonar::prop::DISC_NUMBER),
        created: None,
        starred: None,
        album_id: Some(album.id.to_string()),
        artist_id: Some(artist.id.to_string()),
        ..Default::default()
    }
}

fn playlist_from_playlist(playlist: sonar::Playlist) -> Playlist {
    Playlist {
        id: playlist.id.to_string(),
        name: playlist.name,
        comment: None,
        owner: Some(playlist.owner.to_string()),
        public: None,
        song_count: playlist.track_count,
        duration: Default::default(),
        created: Default::default(),
        changed: Default::default(),
        cover_art: None,
        allowed_user: Default::default(),
    }
}

fn child_from_playlist_tracks(
    playlist_tracks: &[sonar::PlaylistTrack],
    tracks: &[sonar::Track],
) -> Vec<Child> {
    let mut children = Vec::with_capacity(playlist_tracks.len());
    for track in tracks {
        children.push(Child {
            id: track.id.to_string(),
            parent: Some(track.album.to_string()),
            is_dir: false,
            title: track.name.clone(),
            album: None,
            artist: None,
            track: None,
            year: None,
            genre: None,
            cover_art: track.cover_art.map(|id| id.to_string()),
            duration: Some(From::from(track.duration)),
            play_count: Some(track.listen_count as u64),
            disc_number: None,
            album_id: Some(track.album.to_string()),
            artist_id: Some(track.artist.to_string()),
            ..Default::default()
        });
    }
    children
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

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidUsernameError> {
    fn m(self) -> Result<T, opensubsonic::response::Error> {
        self.map_err(|err| {
            opensubsonic::response::Error::with_message(
                opensubsonic::response::ErrorCode::Generic,
                err.to_string(),
            )
        })
    }
}
