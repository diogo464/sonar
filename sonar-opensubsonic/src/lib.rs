use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use eyre::Context;
use opensubsonic::service::prelude::*;
use sonar::PropertyKey;
use tower_http::cors::Any;

const PROPERTY_USER_STARRED: PropertyKey =
    PropertyKey::new_const("user.opensubsonic.sonar.io/starred");
const DEFAULT_MUSIC_FOLDER_ID: u32 = 1;
const DEFAULT_MUSIC_FOLDER_NAME: &str = "sonar";

#[derive(Debug, Default)]
struct FavoritesSet {
    favorites: HashMap<sonar::SonarId, sonar::Favorite>,
}

impl FavoritesSet {
    async fn populate_with<I, ID>(
        &mut self,
        context: &sonar::Context,
        user_id: sonar::UserId,
        ids: I,
    ) -> Result<()>
    where
        I: IntoIterator<Item = ID>,
        ID: Into<sonar::SonarId>,
    {
        let ids = ids.into_iter().map(|id| id.into()).collect::<Vec<_>>();
        let favorites = sonar::favorite_get_bulk(context, user_id, &ids).await.m()?;
        for favorite in favorites {
            self.favorites.insert(favorite.id, favorite);
        }
        Ok(())
    }

    fn starred(&self, id: impl Into<sonar::SonarId>) -> Option<DateTime> {
        let id = id.into();
        let favorite = self.favorites.get(&id)?;
        Some(DateTime::from_unix_seconds(favorite.favorite_at.seconds()))
    }
}

#[derive(Debug)]
struct Server {
    image_url_prefix: String,
    context: sonar::Context,
    tokens: Mutex<HashMap<String, sonar::UserToken>>,
}

impl Server {
    fn new(context: sonar::Context, image_url_prefix: String) -> Self {
        Self {
            image_url_prefix,
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
        let (user_id, token) = sonar::user_login(&self.context, &username, password)
            .await
            .m()?;
        self.set_token(username.as_str(), token);
        Ok(user_id)
    }

    async fn get_artists_id3(&self, user_id: sonar::UserId) -> Result<ArtistsID3> {
        let artists = sonar::artist_list(&self.context, Default::default())
            .await
            .m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(
                &self.context,
                user_id,
                artists.iter().map(|artist| artist.id),
            )
            .await?;

        let mut index: HashMap<char, Vec<ArtistID3>> = HashMap::new();
        for artist in artists {
            index
                .entry(artist.name.chars().next().unwrap_or('#'))
                .or_default()
                .push(artistid3_from_artist(&favorites, artist));
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
    #[tracing::instrument(skip(self))]
    async fn ping(&self, request: Request<Ping>) -> Result<()> {
        let _user_id = self.authenticate(&request).await?;
        Ok(())
    }
    async fn get_bookmarks(&self, _request: Request<GetBookmarks>) -> Result<Bookmarks> {
        Ok(Default::default())
    }
    async fn get_genres(&self, _request: Request<GetGenres>) -> Result<Genres> {
        let genres = sonar::genre_list(&self.context)
            .await
            .m()?
            .into_iter()
            .map(|g| Genre {
                song_count: g.num_tracks,
                album_count: g.num_albums,
                value: g.genre.to_string(),
            })
            .collect();
        Ok(Genres { genre: genres })
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

            let mut favorites = FavoritesSet::default();
            favorites
                .populate_with(&self.context, user_id, artists.iter().map(|v| v.id))
                .await?;
            favorites
                .populate_with(&self.context, user_id, albums.iter().map(|v| v.id))
                .await?;
            favorites
                .populate_with(&self.context, user_id, songs.iter().map(|v| v.id))
                .await?;

            let album_artists = sonar::ext::get_albums_artists_map(&self.context, &albums);
            let track_albums = sonar::ext::get_tracks_albums_map(&self.context, &songs);
            let track_artists = sonar::ext::get_tracks_artists_map(&self.context, &songs);
            let (album_artists, track_albums, track_artists) =
                tokio::try_join!(album_artists, track_albums, track_artists).m()?;
            let audios = sonar::ext::get_tracks_audios_map(&self.context, &songs)
                .await
                .m()?;
            let albums = sonar::ext::albums_map(albums);

            artist = artists
                .into_iter()
                .map(|a| artistid3_from_artist(&favorites, a))
                .collect();
            album = multi_albumid3_from_album_and_artist(&favorites, &album_artists, &albums);
            song = songs
                .into_iter()
                .map(|track| {
                    let album = &track_albums[&track.album];
                    let artist = &track_artists[&track.artist];
                    let audio = track.audio.and_then(|id| audios.get(&id)).cloned();
                    child_from_audio_track_and_album_and_artist(
                        &favorites, artist, album, track, audio,
                    )
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

            let mut favorites = FavoritesSet::default();
            favorites
                .populate_with(&self.context, user_id, result.artists().map(|v| v.id))
                .await?;
            favorites
                .populate_with(&self.context, user_id, result.albums().map(|v| v.id))
                .await?;
            favorites
                .populate_with(&self.context, user_id, result.tracks().map(|v| v.id))
                .await?;

            let album_artists = sonar::ext::get_albums_artists_map(&self.context, result.albums());
            let track_albums = sonar::ext::get_tracks_albums_map(&self.context, result.tracks());
            let track_artists = sonar::ext::get_tracks_artists_map(&self.context, result.tracks());
            let (album_artists, track_albums, track_artists) =
                tokio::try_join!(album_artists, track_albums, track_artists).m()?;
            let audios = sonar::ext::get_tracks_audios_map(&self.context, result.tracks())
                .await
                .m()?;
            let albums = sonar::ext::albums_map(result.albums().cloned());

            artist = result
                .artists()
                .cloned()
                .map(|a| artistid3_from_artist(&favorites, a))
                .collect::<Vec<_>>();
            album = multi_albumid3_from_album_and_artist(&favorites, &album_artists, &albums);
            song = result
                .tracks()
                .cloned()
                .map(|track| {
                    let album = &track_albums[&track.album];
                    let artist = &track_artists[&track.artist];
                    let audio = track.audio.and_then(|id| audios.get(&id)).cloned();
                    child_from_audio_track_and_album_and_artist(
                        &favorites, artist, album, track, audio,
                    )
                })
                .collect();
        }

        Ok(SearchResult3 {
            artist,
            album,
            song,
        })
    }

    async fn get_indexes(&self, request: Request<GetIndexes>) -> Result<ArtistsID3> {
        let user_id = self.authenticate(&request).await?;
        self.get_artists_id3(user_id).await
    }

    async fn get_random_songs(&self, request: Request<GetRandomSongs>) -> Result<Songs> {
        let user_id = self.authenticate(&request).await?;
        let limit = request.body.size.unwrap_or(500);
        let genre = match request.body.genre {
            Some(genre) => Some(genre.parse::<sonar::Genre>().m()?),
            None => None,
        };
        let params = sonar::TrackListRandom {
            limit: Some(limit),
            genre,
        };
        let tracks = sonar::track_list_random(&self.context, params).await.m()?;
        let artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks).await.m()?;
        let albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks).await.m()?;
        let audios = sonar::ext::get_tracks_audios_map(&self.context, &tracks)
            .await
            .m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, tracks.iter().map(|t| t.id))
            .await?;

        let mut child = Vec::with_capacity(tracks.len());
        for track in tracks {
            let artist = &artists[&track.artist];
            let album = &albums[&track.album];
            let audio = track.audio.map(|id| &audios[&id]).cloned();
            child.push(child_from_audio_track_and_album_and_artist(
                &favorites, artist, album, track, audio,
            ));
        }

        Ok(Songs { song: child })
    }

    #[tracing::instrument(skip(self))]
    async fn get_artists(&self, request: Request<GetArtists>) -> Result<ArtistsID3> {
        let user_id = self.authenticate(&request).await?;
        self.get_artists_id3(user_id).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_artist_info2(&self, request: Request<GetArtistInfo2>) -> Result<ArtistInfo2> {
        let artist_id = request.body.id.parse::<sonar::ArtistId>().m()?;
        let artist = sonar::artist_get(&self.context, artist_id).await.m()?;
        let cover_art = match artist.cover_art {
            Some(cover_id) => Some(format!(
                "{}/rest/getCoverArt?id={}&v=1.15.0&u=0&p=0&c=sonar",
                self.image_url_prefix, cover_id
            )),
            None => None,
        };

        Ok(ArtistInfo2 {
            info: ArtistInfoBase {
                large_image_url: cover_art,
                ..Default::default()
            },
            similar_artist: Default::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_artist(&self, request: Request<GetArtist>) -> Result<ArtistWithAlbumsID3> {
        let user_id = self.authenticate(&request).await?;
        let artist_id = request.body.id.parse::<sonar::ArtistId>().m()?;
        let artist = sonar::artist_get(&self.context, artist_id).await.m()?;
        let albums = sonar::album_list_by_artist(&self.context, artist_id, Default::default())
            .await
            .m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, std::iter::once(artist_id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, albums.iter().map(|v| v.id))
            .await?;

        let album = albums
            .into_iter()
            .map(|album| albumid3_from_album_and_artist(&favorites, &artist, album))
            .collect();
        let artist = artistid3_from_artist(&favorites, artist);
        Ok(ArtistWithAlbumsID3 { artist, album })
    }

    #[tracing::instrument(skip(self))]
    async fn get_top_songs(&self, _request: Request<GetTopSongs>) -> Result<TopSongs> {
        Ok(Default::default())
    }

    #[tracing::instrument(skip(self))]
    async fn get_album(&self, request: Request<GetAlbum>) -> Result<AlbumWithSongsID3> {
        let user_id = self.authenticate(&request).await?;
        let album_id = request.body.id.parse::<sonar::AlbumId>().m()?;
        let album = sonar::album_get(&self.context, album_id).await.m()?;
        let artist = sonar::artist_get(&self.context, album.artist).await.m()?;
        let tracks = sonar::track_list_by_album(&self.context, album_id, Default::default())
            .await
            .m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, std::iter::once(album_id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, std::iter::once(artist.id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, tracks.iter().map(|v| v.id))
            .await?;

        let audios = sonar::ext::get_tracks_audios_map(&self.context, &tracks)
            .await
            .m()?;
        let mut song: Vec<Child> = tracks
            .into_iter()
            .map(|track| {
                let audio = track.audio.and_then(|id| audios.get(&id)).cloned();
                child_from_audio_track_and_album_and_artist(
                    &favorites, &artist, &album, track, audio,
                )
            })
            .collect();
        song.sort_by_key(|child| child.track.unwrap_or_default());
        Ok(AlbumWithSongsID3 {
            album: albumid3_from_album_and_artist(&favorites, &artist, album),
            song,
        })
    }
    async fn get_album_info2(&self, _request: Request<GetAlbumInfo2>) -> Result<AlbumInfo> {
        Ok(AlbumInfo::default())
    }
    #[tracing::instrument(skip(self))]
    async fn get_album_list2(&self, request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        let user_id = self.authenticate(&request).await?;
        let params = sonar::ListParams::from((request.body.offset, request.body.size));

        let albums = match request.body.list_type {
            ListType::ByGenre => {
                let genre = request
                    .body
                    .genre
                    .unwrap_or_default()
                    .parse::<sonar::Genre>()
                    .m()?;
                sonar::album_list_by_genre(&self.context, &genre, params)
                    .await
                    .m()?
            }
            _ => sonar::album_list(&self.context, params).await.m()?,
        };

        let albums = sonar::ext::albums_map(albums);
        let artists = sonar::ext::artist_bulk_map(&self.context, albums.values().map(|v| v.artist))
            .await
            .m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, albums.values().map(|v| v.id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, artists.values().map(|v| v.id))
            .await?;

        let album = multi_albumid3_from_album_and_artist(&favorites, &artists, &albums);
        Ok(AlbumList2 { album })
    }

    #[tracing::instrument(skip(self))]
    async fn get_song(&self, request: Request<GetSong>) -> Result<Child> {
        let user_id = self.authenticate(&request).await?;
        let track_id = request.body.id.parse::<sonar::TrackId>().m()?;
        let track = sonar::track_get(&self.context, track_id).await.m()?;
        let album = sonar::album_get(&self.context, track.album).await.m()?;
        let artist = sonar::artist_get(&self.context, track.artist).await.m()?;
        let audio = match track.audio {
            Some(audio_id) => Some(sonar::audio_get(&self.context, audio_id).await.m()?),
            None => None,
        };

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(
                &self.context,
                user_id,
                &[
                    sonar::SonarId::from(track.id),
                    sonar::SonarId::from(album.id),
                    sonar::SonarId::from(artist.id),
                ],
            )
            .await?;

        let child =
            child_from_audio_track_and_album_and_artist(&favorites, &artist, &album, track, audio);
        Ok(child)
    }

    #[tracing::instrument(skip(self))]
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
            sonar::favorite_add(&self.context, user_id, sonar_id)
                .await
                .m()?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_starred(&self, request: Request<GetStarred>) -> Result<Starred> {
        let user_id = self.authenticate(&request).await?;
        let starred_ids = sonar::favorite_list(&self.context, user_id)
            .await
            .m()?
            .into_iter()
            .map(|f| f.id)
            .collect::<Vec<_>>();
        let split = sonar::ext::split_sonar_ids(starred_ids);

        let artists = sonar::artist_get_bulk(&self.context, &split.artist_ids);
        let albums = sonar::album_get_bulk(&self.context, &split.album_ids);
        let tracks = sonar::track_get_bulk(&self.context, &split.track_ids);
        let (artists, albums, tracks) = tokio::try_join!(artists, albums, tracks).m()?;

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, artists.iter().map(|v| v.id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, albums.iter().map(|v| v.id))
            .await?;
        favorites
            .populate_with(&self.context, user_id, tracks.iter().map(|v| v.id))
            .await?;

        let audio_ids = tracks.iter().filter_map(|t| t.audio).collect::<Vec<_>>();

        let album_artists = sonar::ext::get_albums_artists_map(&self.context, &albums);
        let track_artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks);
        let track_albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks);
        let audios = sonar::ext::audio_bulk_map(&self.context, audio_ids);
        let (album_artists, track_artists, track_albums, audios) =
            tokio::try_join!(album_artists, track_artists, track_albums, audios).m()?;

        let mut song = Vec::with_capacity(tracks.len());
        let mut album = Vec::with_capacity(albums.len());
        let mut artist = Vec::with_capacity(artists.len());

        for track in tracks {
            let album = &track_albums[&track.album];
            let artist = &track_artists[&track.artist];
            let audio = track.audio.and_then(|id| audios.get(&id));
            song.push(child_from_audio_track_and_album_and_artist(
                &favorites,
                artist,
                album,
                track,
                audio.cloned(),
            ));
        }

        for alb in albums {
            let artist = &album_artists[&alb.artist];
            album.push(child_from_album_and_artist(&favorites, &alb, &artist));
        }

        for art in artists {
            artist.push(artist_from_artist(art));
        }

        Ok(Starred {
            song,
            album,
            artist,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_starred2(&self, request: Request<GetStarred2>) -> Result<Starred2> {
        let user_id = self.authenticate(&request).await?;
        let starred_ids = sonar::favorite_list(&self.context, user_id)
            .await
            .m()?
            .into_iter()
            .map(|f| f.id)
            .collect::<Vec<_>>();

        let mut favorites = FavoritesSet::default();
        favorites
            .populate_with(&self.context, user_id, &starred_ids)
            .await?;

        let split = sonar::ext::split_sonar_ids(starred_ids);

        let artists = sonar::artist_get_bulk(&self.context, &split.artist_ids);
        let albums = sonar::album_get_bulk(&self.context, &split.album_ids);
        let tracks = sonar::track_get_bulk(&self.context, &split.track_ids);
        let (artists, albums, tracks) = tokio::try_join!(artists, albums, tracks).m()?;

        let audio_ids = tracks.iter().filter_map(|t| t.audio).collect::<Vec<_>>();

        let album_artists = sonar::ext::get_albums_artists_map(&self.context, &albums);
        let track_artists = sonar::ext::get_tracks_artists_map(&self.context, &tracks);
        let track_albums = sonar::ext::get_tracks_albums_map(&self.context, &tracks);
        let audios = sonar::ext::audio_bulk_map(&self.context, audio_ids);
        let (album_artists, track_artists, track_albums, audios) =
            tokio::try_join!(album_artists, track_artists, track_albums, audios).m()?;

        let mut song = Vec::with_capacity(tracks.len());
        let mut album = Vec::with_capacity(albums.len());
        let mut artist = Vec::with_capacity(artists.len());

        for track in tracks {
            let album = &track_albums[&track.album];
            let artist = &track_artists[&track.artist];
            let audio = track.audio.and_then(|id| audios.get(&id));
            song.push(child_from_audio_track_and_album_and_artist(
                &favorites,
                artist,
                album,
                track,
                audio.cloned(),
            ));
        }

        for alb in albums {
            let artist = &album_artists[&alb.artist];
            album.push(albumid3_from_album_and_artist(&favorites, artist, alb));
        }

        for art in artists {
            artist.push(artistid3_from_artist(&favorites, art));
        }

        Ok(Starred2 {
            song,
            album,
            artist,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_playlists(&self, _request: Request<GetPlaylists>) -> Result<Playlists> {
        let mut playlists = sonar::playlist_list(&self.context, Default::default())
            .await
            .m()?;
        playlists.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Playlists {
            playlist: playlists.into_iter().map(playlist_from_playlist).collect(),
        })
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<ByteStream> {
        let image_id = request.body.id.parse::<sonar::ImageId>().m()?;
        let download = sonar::image_download(&self.context, image_id).await.m()?;
        Ok(opensubsonic::common::ByteStream::new(
            download.mime_type,
            download.stream,
        ))
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    async fn download(&self, request: Request<Download>) -> Result<ByteStream> {
        let track_id = request.body.id.parse::<sonar::TrackId>().m()?;
        let download = sonar::track_download(&self.context, track_id, sonar::ByteRange::default())
            .await
            .m()?;
        Ok(opensubsonic::common::ByteStream::new(
            download.mime_type,
            download.stream,
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn stream(&self, request: Request<Stream>, range: ByteRange) -> Result<StreamChunk> {
        let track_id = request.body.id.parse::<sonar::TrackId>().m()?;
        let range = sonar::ByteRange {
            offset: range.offset,
            length: range.length,
        };
        let download = sonar::track_download(&self.context, track_id, range)
            .await
            .m()?;

        let data = sonar::bytestream::to_bytes(download.stream)
            .await
            .map_err(Error::custom)?;

        Ok(StreamChunk {
            content_duration: download.audio.duration,
            content_length: u64::from(download.audio.size),
            mime_type: download.audio.mime_type,
            data,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_music_folders(&self, _request: Request<GetMusicFolders>) -> Result<MusicFolders> {
        Ok(MusicFolders {
            music_folder: vec![MusicFolder {
                id: DEFAULT_MUSIC_FOLDER_ID,
                name: Some(DEFAULT_MUSIC_FOLDER_NAME.to_string()),
            }],
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_podcasts(&self, _request: Request<GetPodcasts>) -> Result<Podcasts> {
        Ok(Default::default())
    }

    #[tracing::instrument(skip(self))]
    async fn get_internet_radio_stations(
        &self,
        _request: Request<GetInternetRadioStations>,
    ) -> Result<InternetRadioStations> {
        Ok(Default::default())
    }
}

fn artist_from_artist(artist: sonar::Artist) -> Artist {
    Artist {
        id: artist.id.to_string(),
        name: artist.name,
        artist_image_url: None,
        starred: None,
        user_rating: None,
        average_rating: None,
    }
}

fn artistid3_from_artist(favorites: &FavoritesSet, artist: sonar::Artist) -> ArtistID3 {
    ArtistID3 {
        id: artist.id.to_string(),
        name: artist.name,
        cover_art: artist.cover_art.map(|id| id.to_string()),
        artist_image_url: None,
        album_count: artist.album_count,
        starred: favorites.starred(artist.id),
    }
}

fn albumid3_from_album_and_artist(
    favorites: &FavoritesSet,
    artist: &sonar::Artist,
    album: sonar::Album,
) -> AlbumID3 {
    let genre = genre_string_from_genres(artist.genres.iter().chain(album.genres.iter()));
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
        starred: favorites.starred(album.id),
        year: None,
        genre: Some(genre),
        user_rating: None,
        record_labels: Default::default(),
    }
}

fn multi_albumid3_from_album_and_artist(
    favorites: &FavoritesSet,
    artists: &HashMap<sonar::ArtistId, sonar::Artist>,
    albums: &HashMap<sonar::AlbumId, sonar::Album>,
) -> Vec<AlbumID3> {
    let mut albumid3s = Vec::with_capacity(albums.len());
    for album in albums.values() {
        let artist = &artists[&album.artist];
        let album = albumid3_from_album_and_artist(favorites, artist, album.clone());
        albumid3s.push(album);
    }
    albumid3s
}

fn child_from_album_and_artist(
    favorites: &FavoritesSet,
    album: &sonar::Album,
    artist: &sonar::Artist,
) -> Child {
    Child {
        id: album.id.to_string(),
        parent: Some(album.artist.to_string()),
        is_dir: true,
        album: Some(album.name.clone()),
        artist: Some(artist.name.clone()),
        cover_art: album.cover_art.map(|id| id.to_string()),
        duration: Some(Seconds::from(album.duration)),
        album_id: Some(album.id.to_string()),
        artist_id: Some(artist.id.to_string()),
        starred: favorites.starred(album.id),
        ..Default::default()
    }
}

fn child_from_audio_track_and_album_and_artist(
    favorites: &FavoritesSet,
    artist: &sonar::Artist,
    album: &sonar::Album,
    track: sonar::Track,
    audio: Option<sonar::Audio>,
) -> Child {
    Child {
        id: track.id.to_string(),
        parent: Some(track.album.to_string()),
        is_dir: false,
        title: track.name,
        album: Some(album.name.clone()),
        artist: Some(artist.name.clone()),
        track: track.properties.get_parsed(sonar::prop::TRACK_NUMBER),
        genre: None,
        cover_art: track
            .cover_art
            .map(|id| id.to_string())
            .or_else(|| album.cover_art.map(|id| id.to_string())),
        duration: Some(From::from(track.duration)),
        play_count: Some(track.listen_count as u64),
        disc_number: track.properties.get_parsed(sonar::prop::DISC_NUMBER),
        starred: favorites.starred(track.id),
        album_id: Some(album.id.to_string()),
        artist_id: Some(artist.id.to_string()),
        media_type: Some(MediaType::Music),
        is_video: Some(false),
        content_type: audio.as_ref().map(|a| a.mime_type.clone()),
        bit_rate: audio.as_ref().map(|a| a.bitrate),
        size: audio.as_ref().map(|a| u64::from(a.size)),
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
        duration: Seconds::from(playlist.duration),
        created: Default::default(),
        changed: Default::default(),
        cover_art: None,
        allowed_user: Default::default(),
    }
}

// TODO: refactor with above method
fn child_from_playlist_tracks(
    playlist_tracks: &[sonar::PlaylistTrack],
    tracks: &[sonar::Track],
    albums: &HashMap<sonar::AlbumId, sonar::Album>,
    artists: &HashMap<sonar::ArtistId, sonar::Artist>,
) -> Vec<Child> {
    let mut children = Vec::with_capacity(playlist_tracks.len());
    for (idx, track) in tracks.iter().enumerate() {
        let artist = &artists[&track.artist];
        let album = &albums[&track.album];
        let genre = genre_string_from_genres(artist.genres.iter().chain(album.genres.iter()));

        children.push(Child {
            id: track.id.to_string(),
            parent: Some(track.album.to_string()),
            is_dir: false,
            title: track.name.clone(),
            album: Some(album.name.clone()),
            artist: Some(artist.name.clone()),
            track: Some((idx + 1) as u32),
            year: None,
            genre: Some(genre),
            cover_art: track
                .cover_art
                .map(|id| id.to_string())
                .or_else(|| album.cover_art.map(|id| id.to_string())),
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

fn genre_string_from_genres<'a>(genres: impl IntoIterator<Item = &'a sonar::Genre>) -> String {
    genres
        .into_iter()
        .next()
        .map(|g| g.to_string())
        .unwrap_or_default()
}

pub async fn start_server(
    address: SocketAddr,
    context: sonar::Context,
    image_url_prefix: String,
) -> eyre::Result<()> {
    tracing::info!("starting opensubsonic server on {}", address);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("creating tcp listener")?;
    let cors = tower_http::cors::CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        // allow requests from any origin
        .allow_origin(Any);
    let service =
        OpenSubsonicService::new("0.0.0", "sonar", Server::new(context, image_url_prefix));
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

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidGenreError> {
    fn m(self) -> Result<T, opensubsonic::response::Error> {
        self.map_err(|err| {
            opensubsonic::response::Error::with_message(
                opensubsonic::response::ErrorCode::Generic,
                err.to_string(),
            )
        })
    }
}
