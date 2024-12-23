use std::{net::SocketAddr, str::FromStr};

use bytes::Bytes;

mod result_ext;
use result_ext::*;

mod conversions;
use conversions::*;
use tokio_stream::StreamExt;
use tonic::{service::interceptor::InterceptedService, transport::Endpoint};

pub mod ext;

tonic::include_proto!("sonar");

pub type Client = sonar_service_client::SonarServiceClient<
    InterceptedService<tonic::transport::Channel, AuthInterceptor>,
>;

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    token: String,
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::try_from(self.token.as_str()).unwrap(),
        );
        Ok(request)
    }
}

#[derive(Clone)]
struct Server {
    context: sonar::Context,
}

impl Server {
    fn new(context: sonar::Context) -> Self {
        Self { context }
    }

    async fn require_user_mt(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<sonar::User, tonic::Status> {
        let token = sonar::UserToken::from_str(
            metadata
                .get("authorization")
                .map(|v| v.to_str().unwrap_or_default())
                .unwrap_or_default(),
        )
        .m()?;
        let user_id = sonar::user_validate_token(&self.context, &token)
            .await
            .m()?;
        let user = sonar::user_get(&self.context, user_id).await.m()?;
        Ok(user)
    }

    async fn require_user<T>(
        &self,
        request: &tonic::Request<T>,
    ) -> Result<sonar::User, tonic::Status> {
        self.require_user_mt(request.metadata()).await
    }

    async fn require_admin_mt(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<sonar::User, tonic::Status> {
        let user = self.require_user_mt(metadata).await?;
        if !user.admin {
            return Err(tonic::Status::permission_denied("not an admin"));
        }
        Ok(user)
    }

    async fn require_admin<T>(
        &self,
        request: &tonic::Request<T>,
    ) -> Result<sonar::User, tonic::Status> {
        self.require_admin_mt(request.metadata()).await
    }

    async fn artist_lookup(&self, id_or_name: &str) -> Result<sonar::Artist, tonic::Status> {
        match id_or_name.parse::<sonar::ArtistId>() {
            Ok(artist_id) => sonar::artist_get(&self.context, artist_id).await.m(),
            Err(_) => sonar::artist_get_by_name(&self.context, id_or_name)
                .await
                .m(),
        }
    }

    async fn album_lookup(&self, id_or_name: &str) -> Result<sonar::Album, tonic::Status> {
        match id_or_name.parse::<sonar::AlbumId>() {
            Ok(album_id) => sonar::album_get(&self.context, album_id).await.m(),
            Err(_) => sonar::album_get_by_name(&self.context, id_or_name)
                .await
                .m(),
        }
    }

    async fn track_lookup(&self, id_or_name: &str) -> Result<sonar::Track, tonic::Status> {
        match id_or_name.parse::<sonar::TrackId>() {
            Ok(track_id) => sonar::track_get(&self.context, track_id).await.m(),
            Err(_) => sonar::track_get_by_name(&self.context, id_or_name)
                .await
                .m(),
        }
    }
}

#[tonic::async_trait]
impl sonar_service_server::SonarService for Server {
    type ImageDownloadStream = SonarImageDownloadStream;
    type TrackDownloadStream = SonarTrackDownloadStream;

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
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let username = req.username.parse::<sonar::Username>().m()?;
        let avatar = parse_imageid_opt(req.avatar_id)?;
        let create = sonar::UserCreate {
            username,
            password: req.password,
            avatar,
            admin: req.admin.unwrap_or(false),
        };
        let user = sonar::user_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(user.into()))
    }
    async fn user_update(
        &self,
        request: tonic::Request<UserUpdateRequest>,
    ) -> std::result::Result<tonic::Response<User>, tonic::Status> {
        self.require_admin(&request).await?;

        todo!()
    }
    async fn user_delete(
        &self,
        request: tonic::Request<UserDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        sonar::user_delete(&self.context, user_id).await.m()?;
        Ok(tonic::Response::new(()))
    }
    async fn user_login(
        &self,
        request: tonic::Request<UserLoginRequest>,
    ) -> std::result::Result<tonic::Response<UserLoginResponse>, tonic::Status> {
        let req = request.into_inner();
        let username = req.username.parse::<sonar::Username>().m()?;
        let password = req.password;
        let (user_id, user_token) = sonar::user_login(&self.context, &username, &password)
            .await
            .m()?;
        Ok(tonic::Response::new(UserLoginResponse {
            token: user_token.to_string(),
            user_id: user_id.to_string(),
        }))
    }
    async fn user_logout(
        &self,
        request: tonic::Request<UserLogoutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let req = request.into_inner();
        let user_token = req.token.parse::<sonar::UserToken>().m()?;
        sonar::user_logout(&self.context, &user_token).await.m()?;
        Ok(tonic::Response::new(()))
    }
    async fn image_create(
        &self,
        request: tonic::Request<ImageCreateRequest>,
    ) -> std::result::Result<tonic::Response<ImageCreateResponse>, tonic::Status> {
        self.require_admin(&request).await?;

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
            image_id: image_id.to_string(),
        }))
    }
    async fn image_delete(
        &self,
        request: tonic::Request<ImageDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        todo!()
    }
    async fn image_download(
        &self,
        request: tonic::Request<ImageDownloadRequest>,
    ) -> std::result::Result<tonic::Response<SonarImageDownloadStream>, tonic::Status> {
        let req = request.into_inner();
        let image_id = parse_imageid(req.image_id)?;
        let image_download = sonar::image_download(&self.context, image_id).await.m()?;
        Ok(tonic::Response::new(SonarImageDownloadStream::new(
            image_id.to_string(),
            image_download.mime_type,
            image_download.stream,
        )))
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
    async fn artist_get(
        &self,
        request: tonic::Request<ArtistGetRequest>,
    ) -> std::result::Result<tonic::Response<Artist>, tonic::Status> {
        let req = request.into_inner();
        let artist = self.artist_lookup(&req.artist).await?;
        Ok(tonic::Response::new(artist.into()))
    }
    async fn artist_create(
        &self,
        request: tonic::Request<ArtistCreateRequest>,
    ) -> std::result::Result<tonic::Response<Artist>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let create = sonar::ArtistCreate {
            name: req.name,
            cover_art: parse_imageid_opt(req.coverart_id)?,
            genres: convert_genres_from_pb(req.genres)?,
            properties: convert_properties_from_pb(req.properties)?,
        };
        let artist = sonar::artist_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(artist.into()))
    }
    async fn artist_update(
        &self,
        request: tonic::Request<ArtistUpdateRequest>,
    ) -> std::result::Result<tonic::Response<Artist>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let (artist_id, update) = TryFrom::try_from(req)?;
        let artist = self.artist_lookup(&artist_id).await?;
        let artist = sonar::artist_update(&self.context, artist.id, update)
            .await
            .m()?;
        Ok(tonic::Response::new(artist.into()))
    }
    async fn artist_delete(
        &self,
        request: tonic::Request<ArtistDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let artist = self.artist_lookup(&req.artist_id).await?;
        sonar::artist_delete(&self.context, artist.id).await.m()?;
        Ok(tonic::Response::new(()))
    }
    async fn album_list(
        &self,
        request: tonic::Request<AlbumListRequest>,
    ) -> std::result::Result<tonic::Response<AlbumListResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = sonar::ListParams::from((req.offset, req.count));
        let albums = sonar::album_list(&self.context, params).await.m()?;
        let albums = albums.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(AlbumListResponse { albums }))
    }
    async fn album_list_by_artist(
        &self,
        request: tonic::Request<AlbumListByArtistRequest>,
    ) -> std::result::Result<tonic::Response<AlbumListResponse>, tonic::Status> {
        let req = request.into_inner();
        let artist = self.artist_lookup(&req.artist_id).await?;
        let artist_id = artist.id;
        let params = sonar::ListParams::from((req.offset, req.count));
        let albums = sonar::album_list_by_artist(&self.context, artist_id, params)
            .await
            .m()?;
        let albums = albums.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(AlbumListResponse { albums }))
    }
    async fn album_get(
        &self,
        request: tonic::Request<AlbumGetRequest>,
    ) -> std::result::Result<tonic::Response<Album>, tonic::Status> {
        let req = request.into_inner();
        let album = self.album_lookup(&req.album).await?;
        Ok(tonic::Response::new(album.into()))
    }
    async fn album_create(
        &self,
        request: tonic::Request<AlbumCreateRequest>,
    ) -> std::result::Result<tonic::Response<Album>, tonic::Status> {
        let req = request.into_inner();
        let create = TryFrom::try_from(req)?;
        let album = sonar::album_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(album.into()))
    }
    async fn album_update(
        &self,
        request: tonic::Request<AlbumUpdateRequest>,
    ) -> std::result::Result<tonic::Response<Album>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let (album_id, update) = TryFrom::try_from(req)?;
        let album = self.album_lookup(&album_id).await?;
        let album = sonar::album_update(&self.context, album.id, update)
            .await
            .m()?;
        Ok(tonic::Response::new(album.into()))
    }
    async fn album_delete(
        &self,
        request: tonic::Request<AlbumDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let album = self.album_lookup(&req.album_id).await?;
        sonar::album_delete(&self.context, album.id).await.m()?;
        Ok(tonic::Response::new(()))
    }
    async fn track_list(
        &self,
        request: tonic::Request<TrackListRequest>,
    ) -> std::result::Result<tonic::Response<TrackListResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = sonar::ListParams::from((req.offset, req.count));
        let tracks = sonar::track_list(&self.context, params).await.m()?;
        let tracks = tracks.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(TrackListResponse { tracks }))
    }
    async fn track_list_by_album(
        &self,
        request: tonic::Request<TrackListByAlbumRequest>,
    ) -> std::result::Result<tonic::Response<TrackListResponse>, tonic::Status> {
        let req = request.into_inner();
        let album = self.album_lookup(&req.album_id).await?;
        let params = sonar::ListParams::from((req.offset, req.count));
        let tracks = sonar::track_list_by_album(&self.context, album.id, params)
            .await
            .m()?;
        let tracks = tracks.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(TrackListResponse { tracks }))
    }
    async fn track_get(
        &self,
        request: tonic::Request<TrackGetRequest>,
    ) -> std::result::Result<tonic::Response<Track>, tonic::Status> {
        let req = request.into_inner();
        let track = self.track_lookup(&req.track).await?;
        Ok(tonic::Response::new(track.into()))
    }
    async fn track_create(
        &self,
        request: tonic::Request<TrackCreateRequest>,
    ) -> std::result::Result<tonic::Response<Track>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let create = TryFrom::try_from(req)?;
        let track = sonar::track_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(track.into()))
    }
    async fn track_update(
        &self,
        request: tonic::Request<TrackUpdateRequest>,
    ) -> std::result::Result<tonic::Response<Track>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let (track_id, update) = TryFrom::try_from(req)?;
        let track = self.track_lookup(&track_id).await?;
        let track = sonar::track_update(&self.context, track.id, update)
            .await
            .m()?;
        Ok(tonic::Response::new(track.into()))
    }
    async fn track_delete(
        &self,
        request: tonic::Request<TrackDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let track = self.track_lookup(&req.track_id).await?;
        sonar::track_delete(&self.context, track.id).await.m()?;
        Ok(tonic::Response::new(()))
    }
    async fn track_lyrics(
        &self,
        request: tonic::Request<TrackLyricsRequest>,
    ) -> std::result::Result<tonic::Response<TrackLyricsResponse>, tonic::Status> {
        let req = request.into_inner();
        let track = self.track_lookup(&req.track_id).await?;
        let lyrics = sonar::track_get_lyrics(&self.context, track.id).await.m()?;
        Ok(tonic::Response::new(TrackLyricsResponse {
            lyrics: Some(lyrics.into()),
        }))
    }
    async fn track_download(
        &self,
        request: tonic::Request<TrackDownloadRequest>,
    ) -> std::result::Result<tonic::Response<Self::TrackDownloadStream>, tonic::Status> {
        let req = request.into_inner();
        let track = self.track_lookup(&req.track_id).await?;
        let download = sonar::track_download(&self.context, track.id, Default::default())
            .await
            .m()?;
        Ok(tonic::Response::new(SonarTrackDownloadStream::new(
            download,
        )))
    }
    async fn track_stat(
        &self,
        request: tonic::Request<TrackStatRequest>,
    ) -> std::result::Result<tonic::Response<TrackStatResponse>, tonic::Status> {
        let req = request.into_inner();
        let track = self.track_lookup(&req.track_id).await?;
        let stat = sonar::track_stat(&self.context, track.id).await.m()?;
        Ok(tonic::Response::new(TrackStatResponse {
            track_id: track.id.to_string(),
            size: stat.size as u32,
        }))
    }
    async fn track_download_chunk(
        &self,
        request: tonic::Request<TrackDownloadChunkRequest>,
    ) -> std::result::Result<tonic::Response<TrackDownloadChunkResponse>, tonic::Status> {
        let req = request.into_inner();
        let track = self.track_lookup(&req.track_id).await?;
        let range = sonar::ByteRange::new(req.offset as u64, req.size as u64);
        let mut download = sonar::track_download(&self.context, track.id, range)
            .await
            .m()?;
        let mut buffer = Vec::<u8>::with_capacity(req.size as usize);
        while let Some(Ok(chunk)) = download.stream.next().await {
            buffer.extend_from_slice(&chunk);
        }
        Ok(tonic::Response::new(TrackDownloadChunkResponse {
            data: buffer,
        }))
    }
    async fn favorite_list(
        &self,
        request: tonic::Request<FavoriteListRequest>,
    ) -> std::result::Result<tonic::Response<FavoriteListResponse>, tonic::Status> {
        let user = self.require_user(&request).await?;
        let req = request.into_inner();
        let target_user = parse_userid(req.user_id)?;

        if target_user != user.id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not allowed to view user's favorites",
            ));
        }

        let favorites = sonar::favorite_list(&self.context, target_user)
            .await
            .m()?
            .into_iter()
            .map(|f| Favorite {
                item_id: f.id.to_string(),
                favorite_at: Some(convert_timestamp_to_pb(f.favorite_at)),
            })
            .collect();

        Ok(tonic::Response::new(FavoriteListResponse { favorites }))
    }
    async fn favorite_add(
        &self,
        request: tonic::Request<FavoriteAddRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;
        let req = request.into_inner();
        let target_user = parse_userid(req.user_id)?;
        let item_id = parse_sonarid(req.item_id)?;

        if target_user != user.id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not allowed to add favorites to target user",
            ));
        }

        sonar::favorite_add(&self.context, target_user, item_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn favorite_remove(
        &self,
        request: tonic::Request<FavoriteRemoveRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;
        let req = request.into_inner();
        let target_user = parse_userid(req.user_id)?;
        let item_id = parse_sonarid(req.item_id)?;

        if target_user != user.id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not allowed to remove favorites from target user",
            ));
        }

        sonar::favorite_remove(&self.context, target_user, item_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn playlist_list(
        &self,
        request: tonic::Request<PlaylistListRequest>,
    ) -> std::result::Result<tonic::Response<PlaylistListResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = sonar::ListParams::from((req.offset, req.count));
        let playlists = sonar::playlist_list(&self.context, params).await.m()?;
        let playlists = playlists.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(PlaylistListResponse { playlists }))
    }
    async fn playlist_get(
        &self,
        request: tonic::Request<PlaylistGetRequest>,
    ) -> std::result::Result<tonic::Response<Playlist>, tonic::Status> {
        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;
        Ok(tonic::Response::new(playlist.into()))
    }
    async fn playlist_create(
        &self,
        request: tonic::Request<PlaylistCreateRequest>,
    ) -> std::result::Result<tonic::Response<Playlist>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let create: sonar::PlaylistCreate = TryFrom::try_from(req)?;

        if user.id != create.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        let playlist = sonar::playlist_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(playlist.into()))
    }
    async fn playlist_duplicate(
        &self,
        request: tonic::Request<PlaylistDuplicateRequest>,
    ) -> std::result::Result<tonic::Response<Playlist>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let new_name = req.new_playlist_name;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;
        if playlist.owner != user_id {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        if user.id != playlist.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        let playlist = sonar::playlist_duplicate(&self.context, playlist_id, &new_name)
            .await
            .m()?;
        Ok(tonic::Response::new(playlist.into()))
    }
    async fn playlist_update(
        &self,
        request: tonic::Request<PlaylistUpdateRequest>,
    ) -> std::result::Result<tonic::Response<Playlist>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let (playlist_id, update): (sonar::PlaylistId, sonar::PlaylistUpdate) =
            TryFrom::try_from(req)?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;

        if user.id != playlist.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        let playlist = sonar::playlist_update(&self.context, playlist_id, update)
            .await
            .m()?;
        Ok(tonic::Response::new(playlist.into()))
    }
    async fn playlist_delete(
        &self,
        request: tonic::Request<PlaylistDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;

        if user.id != playlist.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        sonar::playlist_delete(&self.context, playlist_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn playlist_track_list(
        &self,
        request: tonic::Request<PlaylistTrackListRequest>,
    ) -> std::result::Result<tonic::Response<PlaylistTrackListResponse>, tonic::Status> {
        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let playlist_tracks =
            sonar::playlist_list_tracks(&self.context, playlist_id, Default::default())
                .await
                .m()?;
        let track_ids = playlist_tracks
            .into_iter()
            .map(|playlist_track| playlist_track.track)
            .collect::<Vec<_>>();
        let tracks = sonar::track_get_bulk(&self.context, &track_ids).await.m()?;
        let tracks = tracks.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(PlaylistTrackListResponse { tracks }))
    }
    async fn playlist_track_insert(
        &self,
        request: tonic::Request<PlaylistTrackInsertRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let track_ids = req
            .track_ids
            .into_iter()
            .map(parse_trackid)
            .collect::<Result<Vec<_>, _>>()?;
        sonar::playlist_insert_tracks(&self.context, playlist_id, &track_ids)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn playlist_track_remove(
        &self,
        request: tonic::Request<PlaylistTrackRemoveRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;

        if user.id != playlist.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        let track_ids = req
            .track_ids
            .into_iter()
            .map(parse_trackid)
            .collect::<Result<Vec<_>, _>>()?;
        sonar::playlist_remove_tracks(&self.context, playlist_id, &track_ids)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn playlist_track_clear(
        &self,
        request: tonic::Request<PlaylistTrackClearRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let playlist_id = req.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let playlist = sonar::playlist_get(&self.context, playlist_id).await.m()?;

        if user.id != playlist.owner && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the playlist",
            ));
        }

        sonar::playlist_clear_tracks(&self.context, playlist_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn scrobble_list(
        &self,
        request: tonic::Request<ScrobbleListRequest>,
    ) -> std::result::Result<tonic::Response<ScrobbleListResponse>, tonic::Status> {
        let req = request.into_inner();
        let params = sonar::ListParams::from((req.offset, req.count));
        let scrobbles = sonar::scrobble_list(&self.context, params).await.m()?;
        let scrobbles = scrobbles.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(ScrobbleListResponse { scrobbles }))
    }
    async fn scrobble_create(
        &self,
        request: tonic::Request<ScrobbleCreateRequest>,
    ) -> std::result::Result<tonic::Response<Scrobble>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let create = TryFrom::try_from(req)?;
        let scrobble = sonar::scrobble_create(&self.context, create).await.m()?;
        Ok(tonic::Response::new(scrobble.into()))
    }
    async fn scrobble_delete(
        &self,
        request: tonic::Request<ScrobbleDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let scrobble_id = req.scrobble_id.parse::<sonar::ScrobbleId>().m()?;
        sonar::scrobble_delete(&self.context, scrobble_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn pin_list(
        &self,
        request: tonic::Request<PinListRequest>,
    ) -> std::result::Result<tonic::Response<PinListResponse>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        if user.id != user_id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the pins",
            ));
        }

        let pins = sonar::pin_list(&self.context, user_id).await.m()?;
        let pins = pins.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(PinListResponse { sonar_ids: pins }))
    }
    async fn pin_set(
        &self,
        request: tonic::Request<PinSetRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        if user.id != user_id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the pins",
            ));
        }

        let sonar_ids = req
            .sonar_ids
            .into_iter()
            .map(parse_sonarid)
            .collect::<Result<Vec<_>, _>>()?;
        for sonar_id in sonar_ids {
            sonar::pin_set(&self.context, user_id, sonar_id).await.m()?;
        }
        Ok(tonic::Response::new(()))
    }
    async fn pin_unset(
        &self,
        request: tonic::Request<PinUnsetRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let user = self.require_user(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        if user.id != user_id && !user.admin {
            return Err(tonic::Status::permission_denied(
                "not the owner of the pins",
            ));
        }

        let sonar_ids = req
            .sonar_ids
            .into_iter()
            .map(parse_sonarid)
            .collect::<Result<Vec<_>, _>>()?;
        for sonar_id in sonar_ids {
            sonar::pin_unset(&self.context, user_id, sonar_id)
                .await
                .m()?;
        }
        Ok(tonic::Response::new(()))
    }
    async fn subscription_list(
        &self,
        request: tonic::Request<SubscriptionListRequest>,
    ) -> std::result::Result<tonic::Response<SubscriptionListResponse>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        let subscriptions = sonar::subscription_list(&self.context, user_id).await.m()?;
        let subscriptions = subscriptions.into_iter().map(Into::into).collect();
        Ok(tonic::Response::new(SubscriptionListResponse {
            subscriptions,
        }))
    }
    async fn subscription_create(
        &self,
        request: tonic::Request<SubscriptionCreateRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let user_id = req.user_id.parse::<sonar::UserId>().m()?;
        sonar::subscription_create(
            &self.context,
            sonar::SubscriptionCreate {
                user: user_id,
                external_id: req.external_id,
                interval: req.interval.map(|d| TryFrom::try_from(d).unwrap()),
                description: req.description,
                artist: req.artist,
                album: req.album,
                track: req.track,
                playlist: req.playlist,
                media_type: match req.media_type {
                    Some(media_type) => match media_type.as_str() {
                        "artist" => Some(sonar::SubscriptionMediaType::Artist),
                        "album" => Some(sonar::SubscriptionMediaType::Album),
                        "track" => Some(sonar::SubscriptionMediaType::Track),
                        "playlist" => Some(sonar::SubscriptionMediaType::Playlist),
                        _ => return Err(tonic::Status::invalid_argument("invalid media type")),
                    },
                    None => None,
                },
            },
        )
        .await
        .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn subscription_delete(
        &self,
        request: tonic::Request<SubscriptionDeleteRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;
        let req = request.into_inner();
        let subscription_id = req.id.parse::<sonar::SubscriptionId>().m()?;
        sonar::subscription_delete(&self.context, subscription_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn subscription_submit(
        &self,
        request: tonic::Request<SubscriptionSubmitRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;
        let req = request.into_inner();
        let subscription_id = req.id.parse::<sonar::SubscriptionId>().m()?;
        sonar::subscription_submit(&self.context, subscription_id)
            .await
            .m()?;
        Ok(tonic::Response::new(()))
    }
    async fn import(
        &self,
        request: tonic::Request<tonic::Streaming<ImportRequest>>,
    ) -> std::result::Result<tonic::Response<Track>, tonic::Status> {
        let (mt, _, mut stream) = request.into_parts();
        self.require_admin_mt(&mt).await?;

        let first_message = match stream.message().await? {
            Some(message) => message,
            None => return Err(tonic::Status::invalid_argument("empty stream")),
        };
        let filepath = first_message.filepath;
        let artist = first_message
            .artist_id
            .map(|id| id.parse::<sonar::ArtistId>())
            .transpose()
            .m()?;
        let album = first_message
            .album_id
            .map(|id| id.parse::<sonar::AlbumId>())
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
    async fn search(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> std::result::Result<tonic::Response<SearchResponse>, tonic::Status> {
        let req = request.into_inner();
        let (user_id, query) = TryFrom::try_from(req)?;
        let results = sonar::search(&self.context, user_id, query).await.m()?;
        Ok(tonic::Response::new(SearchResponse {
            results: results.results.into_iter().map(Into::into).collect(),
        }))
    }
    async fn metadata_providers(
        &self,
        request: tonic::Request<MetadataProvidersRequest>,
    ) -> std::result::Result<tonic::Response<MetadataProvidersResponse>, tonic::Status> {
        self.require_admin(&request).await?;

        let providers = sonar::metadata_providers(&self.context);
        Ok(tonic::Response::new(MetadataProvidersResponse {
            providers,
        }))
    }
    async fn metadata_fetch(
        &self,
        request: tonic::Request<MetadataFetchRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.require_admin(&request).await?;

        let req = request.into_inner();
        let mask = metadata_mask_from_fields(req.fields)?;
        let params = sonar::MetadataFetchParams {
            mask,
            providers: req.providers,
        };
        match req.kind {
            _ if req.kind == MetadataFetchKind::Artist as i32 => {
                let artist_id = req.item_id.parse::<sonar::ArtistId>().m()?;
                sonar::metadata_fetch_artist(&self.context, artist_id, params)
                    .await
                    .m()?;
            }
            _ if req.kind == MetadataFetchKind::Album as i32 => {
                let album_id = req.item_id.parse::<sonar::AlbumId>().m()?;
                sonar::metadata_fetch_album(&self.context, album_id, params)
                    .await
                    .m()?;
            }
            _ if req.kind == MetadataFetchKind::Albumtracks as i32 => {
                let album_id = req.item_id.parse::<sonar::AlbumId>().m()?;
                sonar::metadata_fetch_album_tracks(&self.context, album_id, params)
                    .await
                    .m()?;
            }
            _ if req.kind == MetadataFetchKind::Track as i32 => {
                let track_id = parse_trackid(req.item_id)?;
                sonar::metadata_fetch_track(&self.context, track_id, params)
                    .await
                    .m()?;
            }
            _ => {
                return Err(tonic::Status::invalid_argument(format!(
                    "invalid metadata fetch kind: {}",
                    req.kind
                )))
            }
        }
        Ok(tonic::Response::new(()))
    }
    async fn metadata_album_tracks(
        &self,
        request: tonic::Request<MetadataAlbumTracksRequest>,
    ) -> std::result::Result<tonic::Response<MetadataAlbumTracksResponse>, tonic::Status> {
        self.require_admin(&request).await?;

        let request = request.into_inner();
        let album_id = request.album_id.parse::<sonar::AlbumId>().m()?;
        let metadata =
            sonar::metadata_view_album_tracks(&self.context, album_id, &Default::default())
                .await
                .m()?;
        Ok(tonic::Response::new(metadata.into()))
    }
}

pub async fn client(endpoint: &str) -> eyre::Result<Client> {
    client_with_token(endpoint, "").await
}

pub async fn client_with_token(endpoint: &str, token: &str) -> eyre::Result<Client> {
    tracing::info!("connecting to grpc server on {}", endpoint);
    let channel = Endpoint::from_str(endpoint)?.connect().await?;
    let client = sonar_service_client::SonarServiceClient::with_interceptor(
        channel,
        AuthInterceptor {
            token: token.to_string(),
        },
    );
    Ok(client)
}

pub async fn start_server(address: SocketAddr, context: sonar::Context) -> eyre::Result<()> {
    tracing::info!("starting grpc server on {}", address);
    tonic::transport::Server::builder()
        .add_service(sonar_service_server::SonarServiceServer::new(Server::new(
            context,
        )))
        .serve(address)
        .await?;
    Ok(())
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

struct SonarImageDownloadStream {
    image_id: String,
    mime_type: String,
    stream: sonar::bytestream::ByteStream,
}

impl SonarImageDownloadStream {
    fn new(image_id: String, mime_type: String, stream: sonar::bytestream::ByteStream) -> Self {
        Self {
            image_id,
            mime_type,
            stream,
        }
    }
}

impl tokio_stream::Stream for SonarImageDownloadStream {
    type Item = Result<ImageDownloadResponse, tonic::Status>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let stream = std::pin::Pin::new(&mut this.stream);
        match stream.poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(data))) => {
                let image_id = this.image_id.clone();
                let mime_type = this.mime_type.clone();
                let content = data.to_vec();
                std::task::Poll::Ready(Some(Ok(ImageDownloadResponse {
                    image_id,
                    mime_type,
                    content,
                })))
            }
            std::task::Poll::Ready(Some(Err(err))) => std::task::Poll::Ready(Some(Err(
                tonic::Status::new(tonic::Code::Internal, err.to_string()),
            ))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

struct SonarTrackDownloadStream {
    download: sonar::AudioDownload,
}

impl SonarTrackDownloadStream {
    fn new(download: sonar::AudioDownload) -> Self {
        Self { download }
    }
}

impl tokio_stream::Stream for SonarTrackDownloadStream {
    type Item = Result<TrackDownloadResponse, tonic::Status>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let stream = std::pin::Pin::new(&mut this.download.stream);
        match stream.poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(data))) => {
                std::task::Poll::Ready(Some(Ok(TrackDownloadResponse {
                    chunk: data.to_vec(),
                })))
            }
            std::task::Poll::Ready(Some(Err(err))) => std::task::Poll::Ready(Some(Err(
                tonic::Status::new(tonic::Code::Internal, err.to_string()),
            ))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
