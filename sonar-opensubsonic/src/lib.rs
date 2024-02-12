use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use eyre::Context;
use opensubsonic::service::prelude::*;
use sonar::PropertyKey;
use tower_http::cors::Any;

const PROPERTY_USER_STARRED: PropertyKey =
    PropertyKey::new_const("user.opensubsonic.sonar.io/starred");
const DEFAULT_MUSIC_FOLDER_ID: u32 = 1;
const DEFAULT_MUSIC_FOLDER_NAME: &str = "sonar";

#[derive(Debug)]
struct Server {
    context: sonar::Context,
    tokens: Mutex<HashMap<String, sonar::UserToken>>,
}

impl Server {
    fn new(context: sonar::Context) -> Self {
        Self {
            context,
            tokens: Default::default(),
        }
    }

    async fn authenticate<R: SubsonicRequest>(
        &self,
        request: &Request<R>,
    ) -> Result<sonar::UserId> {
        if let Some(token) = self.get_token(&request.username) {
            if let Ok(user_id) = sonar::user_validate_token(&self.context, &token).await {
                return Ok(user_id);
            } else {
                tracing::warn!("invalid token for user {}", request.username);
            }
        }

        let _username = request.username.parse::<sonar::Username>().m()?;
        let username = "admin".parse::<sonar::Username>().m()?;
        let _password = match &request.authentication {
            Authentication::Password(password) => password,
            _ => {
                return Err(opensubsonic::response::Error::with_message(
                    opensubsonic::response::ErrorCode::TokenAuthenticationNotSupported,
                    "password authentication required".to_string(),
                ))
            }
        };
        let (user_id, token) = sonar::user_login(&self.context, &username, "test12345")
            .await
            .m()?;
        self.set_token(username.as_str(), token);
        Ok(user_id)
    }

    fn get_token(&self, username: &str) -> Option<sonar::UserToken> {
        let tokens = self.tokens.lock().unwrap();
        tokens.get(username).cloned()
    }

    fn set_token(&self, username: &str, token: sonar::UserToken) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(username.to_string(), token);
    }
}

#[opensubsonic::async_trait]
impl OpenSubsonicServer for Server {
    #[tracing::instrument]
    async fn ping(&self, request: Request<Ping>) -> Result<()> {
        let _user_id = self.authenticate(&request).await?;
        Ok(())
    }
    async fn get_bookmarks(&self, _request: Request<GetBookmarks>) -> Result<Bookmarks> {
        Ok(Default::default())
    }
    async fn get_genres(&self, _request: Request<GetGenres>) -> Result<Genres> {
        Ok(Default::default())
    }
    async fn search3(&self, request: Request<Search3>) -> Result<SearchResult3> {
        const DEFAULT_LIMIT: u32 = 50;

        let user_id = self.authenticate(&request).await?;

        let artist;
        let album;
        let song;

        let artist_limit = request.body.artist_count.unwrap_or(DEFAULT_LIMIT);
        let artist_offset = request.body.artist_offset.unwrap_or(0);
        let album_limit = request.body.album_count.unwrap_or(DEFAULT_LIMIT);
        let album_offset = request.body.album_offset.unwrap_or(0);
        let song_limit = request.body.song_count.unwrap_or(DEFAULT_LIMIT);
        let song_offset = request.body.song_offset.unwrap_or(0);

        // symfonium sends two quotes when the search is empty
        if request.body.query.is_empty() || request.body.query == "\"\"" {
            let artist_params = sonar::ListParams::from((artist_offset, artist_limit));
            let album_params = sonar::ListParams::from((album_offset, album_limit));
            let song_params = sonar::ListParams::from((song_offset, song_limit));

            let artists = sonar::artist_list(&self.context, artist_params);
            let albums = sonar::album_list(&self.context, album_params);
            let songs = sonar::track_list(&self.context, song_params);
            let (artists, albums, songs) = tokio::try_join!(artists, albums, songs).m()?;

            let album_artists = sonar::ext::get_albums_artists_map(&self.context, &albums);
            let album_user_props = sonar::ext::user_property_bulk_map(
                &self.context,
                user_id,
                albums.iter().map(|album| sonar::SonarId::from(album.id)),
            );
            let track_albums = sonar::ext::get_tracks_albums_map(&self.context, &songs);
            let track_artists = sonar::ext::get_tracks_artists_map(&self.context, &songs);
            let (album_artists, album_user_props, track_albums, track_artists) =
                tokio::try_join!(album_artists, album_user_props, track_albums, track_artists)
                    .m()?;

            let albums = sonar::ext::albums_map(albums);

            artist = artists.into_iter().map(artistid3_from_artist).collect();
            album =
                multi_albumid3_from_album_and_artist(&album_artists, &albums, &album_user_props);
            song = songs
                .into_iter()
                .map(|track| {
                    let album = &track_albums[&track.album];
                    let artist = &track_artists[&track.artist];
                    child_from_track_and_album_and_artist(artist, album, track)
                })
                .collect();
        } else {
            let mut flags = 0;
            if artist_limit > 0 {
                flags |= sonar::SearchQuery::FLAG_ARTIST;
            }
            if album_limit > 0 {
                flags |= sonar::SearchQuery::FLAG_ALBUM;
            }
            if song_limit > 0 {
                flags |= sonar::SearchQuery::FLAG_TRACK;
            }

            let result = sonar::search(
                &self.context,
                user_id,
                sonar::SearchQuery {
                    query: request.body.query,
                    limit: Some(artist_limit + album_limit + song_limit),
                    flags,
                },
            )
            .await
            .m()?;

            let album_artists = sonar::ext::get_albums_artists_map(&self.context, result.albums());
            let album_user_props = sonar::ext::user_property_bulk_map(
                &self.context,
                user_id,
                result.albums().map(|album| sonar::SonarId::from(album.id)),
            );
            let track_albums = sonar::ext::get_tracks_albums_map(&self.context, result.tracks());
            let track_artists = sonar::ext::get_tracks_artists_map(&self.context, result.tracks());
            let (album_artists, album_user_props, track_albums, track_artists) =
                tokio::try_join!(album_artists, album_user_props, track_albums, track_artists)
                    .m()?;
            let albums = sonar::ext::albums_map(result.albums().cloned());

            artist = result
                .artists()
                .cloned()
                .map(artistid3_from_artist)
                .collect::<Vec<_>>();
            album =
                multi_albumid3_from_album_and_artist(&album_artists, &albums, &album_user_props);
            song = result
                .tracks()
                .cloned()
                .map(|track| {
                    let album = &track_albums[&track.album];
                    let artist = &track_artists[&track.artist];
                    child_from_track_and_album_and_artist(artist, album, track)
                })
                .collect();
        }

        Ok(SearchResult3 {
            artist,
            album,
            song,
        })
    }
    async fn get_indexes(&self, _request: Request<GetIndexes>) -> Result<ArtistsID3> {
        // TODO: refactor with get_artists
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
    #[tracing::instrument]
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

    #[tracing::instrument]
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

    #[tracing::instrument]
    async fn get_artist(&self, request: Request<GetArtist>) -> Result<ArtistWithAlbumsID3> {
        let artist_id = request.body.id.parse::<sonar::ArtistId>().m()?;
        let artist = sonar::artist_get(&self.context, artist_id).await.m()?;
        let albums = sonar::album_list_by_artist(&self.context, artist_id, Default::default())
            .await
            .m()?;
        let album = albums
            .into_iter()
            .map(|album| albumid3_from_album_and_artist(&artist, album))
            .collect();
        let artist = artistid3_from_artist(artist);
        Ok(ArtistWithAlbumsID3 { artist, album })
    }

    #[tracing::instrument]
    async fn get_top_songs(&self, _request: Request<GetTopSongs>) -> Result<TopSongs> {
        Ok(Default::default())
    }

    #[tracing::instrument]
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
    async fn get_album_info2(&self, _request: Request<GetAlbumInfo2>) -> Result<AlbumInfo> {
        Ok(AlbumInfo::default())
    }
    #[tracing::instrument]
    async fn get_album_list2(&self, request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        let user_id = self.authenticate(&request).await?;
        let params = sonar::ListParams::from((request.body.offset, request.body.size));
        let albums = sonar::album_list(&self.context, params).await.m()?;
        let albums = sonar::ext::albums_map(albums);
        let artists = sonar::ext::artist_bulk_map(&self.context, albums.values().map(|v| v.artist))
            .await
            .m()?;
        let album_properties = sonar::ext::user_property_bulk_map(
            &self.context,
            user_id,
            albums.keys().copied().map(sonar::SonarId::from),
        )
        .await
        .m()?;
        let album = multi_albumid3_from_album_and_artist(&artists, &albums, &album_properties);
        Ok(AlbumList2 { album })
    }

    #[tracing::instrument]
    async fn star(&self, request: Request<Star>) -> Result<()> {
        let user_id = self.authenticate(&request).await?;
        let star_ids = request
            .body
            .id
            .into_iter()
            .chain(request.body.album_id)
            .chain(request.body.artist_id);
        for id in star_ids {
            let sonar_id = id.parse::<sonar::SonarId>().m()?;
            let value = sonar::PropertyValue::new(sonar::chrono::Utc::now().to_string()).unwrap();
            let update = sonar::PropertyUpdate::set(PROPERTY_USER_STARRED, value);
            sonar::user_property_update(&self.context, user_id, sonar_id, &[update])
                .await
                .m()?;
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn get_starred2(&self, request: Request<GetStarred2>) -> Result<Starred2> {
        let user_id = self.authenticate(&request).await?;
        let starred_ids =
            sonar::list_with_user_property(&self.context, user_id, &PROPERTY_USER_STARRED)
                .await
                .m()?;
        let split = sonar::ext::split_sonar_ids(starred_ids);

        let artists = sonar::artist_get_bulk(&self.context, &split.artist_ids);
        let albums = sonar::album_get_bulk(&self.context, &split.album_ids);
        let tracks = sonar::track_get_bulk(&self.context, &split.track_ids);
        let (artists, albums, tracks) = tokio::try_join!(artists, albums, tracks).m()?;

        let album_artists = sonar::ext::get_albums_artists_map(&self.context, &albums);
        let track_artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks);
        let track_albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks);
        let (album_artists, track_artists, track_albums) =
            tokio::try_join!(album_artists, track_artists, track_albums).m()?;

        let mut song = Vec::with_capacity(tracks.len());
        let mut album = Vec::with_capacity(albums.len());
        let mut artist = Vec::with_capacity(artists.len());

        for track in tracks {
            let album = &track_albums[&track.album];
            let artist = &track_artists[&track.artist];
            song.push(child_from_track_and_album_and_artist(artist, album, track));
        }

        for alb in albums {
            let artist = &album_artists[&alb.artist];
            album.push(albumid3_from_album_and_artist(artist, alb));
        }

        for art in artists {
            artist.push(artistid3_from_artist(art));
        }

        Ok(Starred2 {
            song,
            album,
            artist,
        })
    }

    #[tracing::instrument]
    async fn get_playlists(&self, _request: Request<GetPlaylists>) -> Result<Playlists> {
        let playlists = sonar::playlist_list(&self.context, Default::default())
            .await
            .m()?;
        Ok(Playlists {
            playlist: playlists.into_iter().map(playlist_from_playlist).collect(),
        })
    }

    #[tracing::instrument]
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
        let albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks);
        let artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks);
        let (albums, artists) = tokio::try_join!(albums, artists).m()?;

        Ok(PlaylistWithSongs {
            playlist: playlist_from_playlist(playlist),
            entry: child_from_playlist_tracks(&playlist_tracks, &tracks, &albums, &artists),
        })
    }

    #[tracing::instrument]
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
            (Some(_playlist_id), _) => {
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
        let albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks);
        let artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks);
        let (albums, artists) = tokio::try_join!(albums, artists).m()?;
        Ok(PlaylistWithSongs {
            playlist: playlist_from_playlist(playlist),
            entry: child_from_playlist_tracks(&playlist_tracks, &tracks, &albums, &artists),
        })
    }

    #[tracing::instrument]
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<ByteStream> {
        let image_id = request.body.id.parse::<sonar::ImageId>().m()?;
        let download = sonar::image_download(&self.context, image_id).await.m()?;
        Ok(opensubsonic::common::ByteStream::new(
            download.mime_type,
            download.stream,
        ))
    }

    #[tracing::instrument]
    async fn scrobble(&self, request: Request<Scrobble>) -> Result<()> {
        let user_id = self.authenticate(&request).await?;

        if !request.body.submission.unwrap_or(false) {
            return Ok(());
        }

        for (idx, id) in request.body.id.into_iter().enumerate() {
            let timestamp = match request.body.time.get(idx) {
                Some(timestamp_ms) => sonar::Timestamp::from_duration(timestamp_ms.to_duration()),
                None => sonar::Timestamp::now(),
            };

            let track_id = id.parse::<sonar::TrackId>().m()?;
            let track = sonar::track_get(&self.context, track_id).await.m()?;
            sonar::scrobble_create(
                &self.context,
                sonar::ScrobbleCreate {
                    user: user_id,
                    track: track_id,
                    listen_at: timestamp,
                    listen_duration: track.duration,
                    listen_device: "opensubsonic".to_string(),
                    properties: Default::default(),
                },
            )
            .await
            .m()?;
        }

        Ok(())
    }

    #[tracing::instrument]
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

    #[tracing::instrument]
    async fn get_music_folders(&self, _request: Request<GetMusicFolders>) -> Result<MusicFolders> {
        Ok(MusicFolders {
            music_folder: vec![MusicFolder {
                id: DEFAULT_MUSIC_FOLDER_ID,
                name: Some(DEFAULT_MUSIC_FOLDER_NAME.to_string()),
            }],
        })
    }

    #[tracing::instrument]
    async fn get_podcasts(&self, _request: Request<GetPodcasts>) -> Result<Podcasts> {
        Ok(Default::default())
    }

    #[tracing::instrument]
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
    albums_user_properties: &HashMap<sonar::SonarId, sonar::Properties>,
) -> Vec<AlbumID3> {
    let mut albumid3s = Vec::with_capacity(albums.len());
    for album in albums.values() {
        let artist = &artists[&album.artist];
        let properties = &albums_user_properties[&sonar::SonarId::from(album.id)];
        let mut album = albumid3_from_album_and_artist(artist, album.clone());
        if let Some(value) = properties.get(PROPERTY_USER_STARRED) {
            let starred = value.as_str().parse::<sonar::DateTime>().unwrap();
            album.starred = Some(starred.to_rfc3339().parse().unwrap());
        }
        albumid3s.push(album);
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
        cover_art: track
            .cover_art
            .map(|id| id.to_string())
            .or_else(|| album.cover_art.map(|id| id.to_string())),
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
    albums: &HashMap<sonar::AlbumId, sonar::Album>,
    artists: &HashMap<sonar::ArtistId, sonar::Artist>,
) -> Vec<Child> {
    let mut children = Vec::with_capacity(playlist_tracks.len());
    for (idx, track) in tracks.iter().enumerate() {
        children.push(Child {
            id: track.id.to_string(),
            parent: Some(track.album.to_string()),
            is_dir: false,
            title: track.name.clone(),
            album: Some(albums[&track.album].name.clone()),
            artist: Some(artists[&track.artist].name.clone()),
            track: Some((idx + 1) as u32),
            year: None,
            genre: None,
            cover_art: track
                .cover_art
                .map(|id| id.to_string())
                .or_else(|| albums[&track.album].cover_art.map(|id| id.to_string())),
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
            let code = match err.kind() {
                sonar::ErrorKind::Unauthorized => {
                    opensubsonic::response::ErrorCode::WrongUsernameOrPassword
                }
                _ => opensubsonic::response::ErrorCode::Generic,
            };
            opensubsonic::response::Error::with_message(code, err.to_string())
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
