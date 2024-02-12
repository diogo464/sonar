use crate::*;

impl From<sonar::User> for User {
    fn from(value: sonar::User) -> Self {
        Self {
            user_id: value.id.to_string(),
            username: From::from(value.username),
            avatar_id: value.avatar.map(|id| id.to_string()),
        }
    }
}

impl From<sonar::Artist> for Artist {
    fn from(value: sonar::Artist) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            album_count: value.album_count,
            listen_count: value.listen_count,
            coverart_id: value.cover_art.map(|id| id.to_string()),
            genres: convert_genres_to_pb(value.genres),
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<ArtistUpdateRequest> for (String, sonar::ArtistUpdate) {
    type Error = tonic::Status;

    fn try_from(value: ArtistUpdateRequest) -> Result<Self, Self::Error> {
        let update = sonar::ArtistUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            cover_art: sonar::ValueUpdate::from_option_unchanged(parse_imageid_opt(
                value.coverart_id,
            )?),
            genres: convert_genre_updates_from_pb(value.genres)?,
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((value.artist_id, update))
    }
}

impl From<sonar::Album> for Album {
    fn from(value: sonar::Album) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            track_count: value.track_count,
            duration: Some(TryFrom::try_from(value.duration).expect("failed to convert duration")),
            listen_count: value.listen_count,
            artist_id: value.artist.to_string(),
            coverart_id: value.cover_art.map(|id| id.to_string()),
            genres: convert_genres_to_pb(value.genres),
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<AlbumCreateRequest> for sonar::AlbumCreate {
    type Error = tonic::Status;

    fn try_from(value: AlbumCreateRequest) -> Result<Self, Self::Error> {
        Ok(sonar::AlbumCreate {
            name: value.name,
            artist: parse_artistid(value.artist_id)?,
            cover_art: parse_imageid_opt(value.coverart_id)?,
            genres: convert_genres_from_pb(value.genres)?,
            properties: convert_properties_from_pb(value.properties)?,
        })
    }
}

impl TryFrom<AlbumUpdateRequest> for (String, sonar::AlbumUpdate) {
    type Error = tonic::Status;

    fn try_from(value: AlbumUpdateRequest) -> Result<Self, Self::Error> {
        let artist_id = parse_artistid_opt(value.artist_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        let update = sonar::AlbumUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            artist: sonar::ValueUpdate::from_option_unchanged(artist_id),
            cover_art: sonar::ValueUpdate::from_option_unchanged(cover_art),
            genres: convert_genre_updates_from_pb(value.genres)?,
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((value.album_id, update))
    }
}

impl From<sonar::Track> for Track {
    fn from(value: sonar::Track) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            artist_id: value.artist.to_string(),
            album_id: value.album.to_string(),
            duration: Some(TryFrom::try_from(value.duration).expect("failed to convert duration")),
            listen_count: value.listen_count,
            cover_art_id: value.cover_art.map(|id| id.to_string()),
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<TrackCreateRequest> for sonar::TrackCreate {
    type Error = tonic::Status;

    fn try_from(value: TrackCreateRequest) -> Result<Self, Self::Error> {
        let album_id = parse_albumid(value.album_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        Ok(Self {
            name: value.name,
            album: album_id,
            cover_art,
            lyrics: Default::default(),
            audio: Default::default(),
            properties: convert_properties_from_pb(value.properties)?,
        })
    }
}

impl TryFrom<TrackUpdateRequest> for (String, sonar::TrackUpdate) {
    type Error = tonic::Status;

    fn try_from(value: TrackUpdateRequest) -> Result<Self, Self::Error> {
        let album_id = parse_albumid_opt(value.album_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        let update = sonar::TrackUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            album: sonar::ValueUpdate::from_option_unchanged(album_id),
            cover_art: sonar::ValueUpdate::from_option_unchanged(cover_art),
            lyrics: Default::default(),
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((value.track_id, update))
    }
}

impl From<sonar::Lyrics> for Lyrics {
    fn from(value: sonar::Lyrics) -> Self {
        Self {
            synced: value.kind == sonar::LyricsKind::Synced,
            lines: value
                .lines
                .into_iter()
                .map(|l| LyricsLine {
                    offset: l.offset.as_secs() as u32,
                    text: l.text,
                })
                .collect(),
        }
    }
}

impl From<sonar::Playlist> for Playlist {
    fn from(value: sonar::Playlist) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            user_id: value.owner.to_string(),
            track_count: value.track_count,
            duration: None,
            coverart_id: None,
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<PlaylistCreateRequest> for sonar::PlaylistCreate {
    type Error = tonic::Status;

    fn try_from(value: PlaylistCreateRequest) -> Result<Self, Self::Error> {
        let owner = value.owner_id.parse::<sonar::UserId>().m()?;
        let properties = convert_properties_from_pb(value.properties)?;
        let tracks = value
            .track_ids
            .into_iter()
            .map(parse_trackid)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            name: value.name,
            owner,
            tracks,
            properties,
        })
    }
}

impl TryFrom<PlaylistUpdateRequest> for (sonar::PlaylistId, sonar::PlaylistUpdate) {
    type Error = tonic::Status;

    fn try_from(value: PlaylistUpdateRequest) -> Result<Self, Self::Error> {
        let playlist_id = value.playlist_id.parse::<sonar::PlaylistId>().m()?;
        let update = sonar::PlaylistUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((playlist_id, update))
    }
}

impl From<sonar::Scrobble> for Scrobble {
    fn from(value: sonar::Scrobble) -> Self {
        Self {
            id: value.id.to_string(),
            track_id: value.track.to_string(),
            user_id: value.user.to_string(),
            listen_at: Some(convert_timestamp_to_pb(value.listen_at)),
            listen_duration: Some(
                TryFrom::try_from(value.listen_duration).expect("failed to convert duration"),
            ),
            listen_device: value.listen_device,
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<ScrobbleCreateRequest> for sonar::ScrobbleCreate {
    type Error = tonic::Status;

    fn try_from(value: ScrobbleCreateRequest) -> Result<Self, Self::Error> {
        let track = parse_trackid(value.track_id)?;
        let user = parse_userid(value.user_id)?;
        let listen_at = value
            .listen_at
            .ok_or_else(|| tonic::Status::invalid_argument("listen_at is required"))?;
        let listen_duration = value
            .listen_duration
            .ok_or_else(|| tonic::Status::invalid_argument("listen_duration is required"))?;
        let listen_duration =
            std::time::Duration::new(listen_duration.seconds as u64, listen_duration.nanos as u32);
        let properties = convert_properties_from_pb(value.properties)?;
        Ok(Self {
            track,
            user,
            listen_at: convert_timestamp_from_pb(listen_at),
            listen_duration,
            listen_device: value.listen_device,
            properties,
        })
    }
}

impl From<sonar::SearchResult> for SearchResult {
    fn from(value: sonar::SearchResult) -> Self {
        match value {
            sonar::SearchResult::Artist(v) => SearchResult {
                result: Some(crate::search_result::Result::Artist(v.into())),
            },
            sonar::SearchResult::Album(v) => SearchResult {
                result: Some(crate::search_result::Result::Album(v.into())),
            },
            sonar::SearchResult::Track(v) => SearchResult {
                result: Some(crate::search_result::Result::Track(v.into())),
            },
            sonar::SearchResult::Playlist(v) => SearchResult {
                result: Some(crate::search_result::Result::Playlist(v.into())),
            },
        }
    }
}

impl TryFrom<SearchRequest> for (sonar::UserId, sonar::SearchQuery) {
    type Error = tonic::Status;

    fn try_from(value: SearchRequest) -> Result<Self, Self::Error> {
        let user_id = parse_userid(value.user_id)?;
        let flags = match value.flags {
            Some(f) => {
                let mut flags = sonar::SearchQuery::FLAG_NONE;
                if f & (crate::search_request::Flags::FlagArtist as u32) != 0 {
                    flags |= sonar::SearchQuery::FLAG_ARTIST;
                }
                if f & (crate::search_request::Flags::FlagAlbum as u32) != 0 {
                    flags |= sonar::SearchQuery::FLAG_ALBUM;
                }
                if f & (crate::search_request::Flags::FlagTrack as u32) != 0 {
                    flags |= sonar::SearchQuery::FLAG_TRACK;
                }
                if f & (crate::search_request::Flags::FlagPlaylist as u32) != 0 {
                    flags |= sonar::SearchQuery::FLAG_PLAYLIST;
                }
                flags
            }
            None => sonar::SearchQuery::FLAG_ALL,
        };
        let query = sonar::SearchQuery {
            query: value.query,
            limit: value.limit,
            flags,
        };
        Ok((user_id, query))
    }
}

impl From<sonar::Subscription> for Subscription {
    fn from(value: sonar::Subscription) -> Self {
        Self {
            user_id: value.user.to_string(),
            external_id: value.external_id.to_string(),
            description: value.description.unwrap_or_default(),
        }
    }
}

impl From<sonar::Download> for Download {
    fn from(value: sonar::Download) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            external_id: value.external_id.to_string(),
            description: value.description,
        }
    }
}

impl From<sonar::TrackMetadata> for TrackMetadata {
    fn from(value: sonar::TrackMetadata) -> Self {
        Self {
            name: value.name,
            properties: convert_properties_to_pb(value.properties),
            cover: value.cover.map(From::from),
        }
    }
}

impl From<sonar::AlbumTracksMetadata> for MetadataAlbumTracksResponse {
    fn from(value: sonar::AlbumTracksMetadata) -> Self {
        Self {
            tracks: value
                .tracks
                .into_iter()
                .map(|(id, v)| (id.to_string(), From::from(v)))
                .collect(),
        }
    }
}

pub fn convert_properties_to_pb(properties: sonar::Properties) -> Vec<Property> {
    let mut props = Vec::with_capacity(properties.len());
    for (key, value) in properties {
        props.push(Property {
            key: From::from(key),
            value: From::from(value),
        });
    }
    props
}

pub fn convert_properties_from_pb(
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

pub fn convert_property_updates_from_pb(
    updates: Vec<PropertyUpdate>,
) -> Result<Vec<sonar::PropertyUpdate>, tonic::Status> {
    let mut props = Vec::with_capacity(updates.len());
    for update in updates {
        let key = update.key.parse::<sonar::PropertyKey>().m()?;
        let value = update
            .value
            .map(|v| v.parse::<sonar::PropertyValue>())
            .transpose()
            .m()?;
        props.push(sonar::PropertyUpdate::from_option(key, value));
    }
    Ok(props)
}

pub fn convert_genres_to_pb(genres: sonar::Genres) -> Vec<String> {
    genres.into()
}

pub fn convert_genres_from_pb(genres: Vec<String>) -> Result<sonar::Genres, tonic::Status> {
    sonar::Genres::new(genres).map_err(|_| tonic::Status::invalid_argument("invalid genre"))
}

pub fn convert_genre_updates_from_pb(
    updates: Vec<GenreUpdate>,
) -> Result<Vec<sonar::GenreUpdate>, tonic::Status> {
    let mut out = Vec::with_capacity(updates.len());
    for update in updates {
        let genre = sonar::Genre::new(update.genre)
            .map_err(|err| tonic::Status::invalid_argument(format!("invalid genre: {}", err)))?;
        let action = if update.action == genre_update::Action::Set as i32 {
            sonar::GenreUpdateAction::Set
        } else if update.action == genre_update::Action::Unset as i32 {
            sonar::GenreUpdateAction::Unset
        } else {
            return Err(tonic::Status::invalid_argument(
                "invalid genre update action",
            ));
        };
        out.push(sonar::GenreUpdate { action, genre });
    }
    Ok(out)
}

pub fn convert_timestamp_to_pb(timestamp: sonar::Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: timestamp.seconds() as i64,
        nanos: timestamp.nanos() as i32,
    }
}

pub fn convert_timestamp_from_pb(timestamp: prost_types::Timestamp) -> sonar::Timestamp {
    sonar::Timestamp::new(timestamp.seconds as u64, timestamp.nanos as u32)
}

pub fn parse_userid(id: String) -> Result<sonar::UserId, tonic::Status> {
    id.parse::<sonar::UserId>().m()
}

pub fn parse_artistid(id: String) -> Result<sonar::ArtistId, tonic::Status> {
    id.parse::<sonar::ArtistId>().m()
}

pub fn parse_artistid_opt(id: Option<String>) -> Result<Option<sonar::ArtistId>, tonic::Status> {
    id.map(|id| id.parse::<sonar::ArtistId>().m()).transpose()
}

pub fn parse_albumid(id: String) -> Result<sonar::AlbumId, tonic::Status> {
    id.parse::<sonar::AlbumId>().m()
}

pub fn parse_albumid_opt(id: Option<String>) -> Result<Option<sonar::AlbumId>, tonic::Status> {
    id.map(|id| id.parse::<sonar::AlbumId>().m()).transpose()
}

pub fn parse_imageid(id: String) -> Result<sonar::ImageId, tonic::Status> {
    id.parse::<sonar::ImageId>().m()
}

pub fn parse_imageid_opt(id: Option<String>) -> Result<Option<sonar::ImageId>, tonic::Status> {
    id.map(|id| id.parse::<sonar::ImageId>().m()).transpose()
}

pub fn parse_trackid(id: String) -> Result<sonar::TrackId, tonic::Status> {
    id.parse::<sonar::TrackId>().m()
}

pub fn parse_sonarid(id: String) -> Result<sonar::SonarId, tonic::Status> {
    id.parse::<sonar::SonarId>().m()
}
