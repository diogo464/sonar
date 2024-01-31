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
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<ArtistUpdateRequest> for (sonar::ArtistId, sonar::ArtistUpdate) {
    type Error = tonic::Status;

    fn try_from(value: ArtistUpdateRequest) -> Result<Self, Self::Error> {
        let artist_id = parse_artistid(value.artist_id)?;
        let update = sonar::ArtistUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            cover_art: Default::default(),
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((artist_id, update))
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
            artists: vec![From::from(value.artist)],
            coverart_id: value.cover_art.map(|id| id.to_string()),
            properties: convert_properties_to_pb(value.properties),
        }
    }
}

impl TryFrom<AlbumCreateRequest> for sonar::AlbumCreate {
    type Error = tonic::Status;

    fn try_from(value: AlbumCreateRequest) -> Result<Self, Self::Error> {
        let artist = parse_artistid(value.artist_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        let properties = convert_properties_from_pb(value.properties)?;
        Ok(sonar::AlbumCreate {
            name: value.name,
            artist,
            cover_art,
            properties,
        })
    }
}

impl TryFrom<AlbumUpdateRequest> for (sonar::AlbumId, sonar::AlbumUpdate) {
    type Error = tonic::Status;

    fn try_from(value: AlbumUpdateRequest) -> Result<Self, Self::Error> {
        let artist_id = parse_artistid_opt(value.artist_id)?;
        let album_id = parse_albumid(value.album_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        let update = sonar::AlbumUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            artist: sonar::ValueUpdate::from_option_unchanged(artist_id),
            cover_art: sonar::ValueUpdate::from_option_unchanged(cover_art),
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((album_id, update))
    }
}

impl From<sonar::Track> for Track {
    fn from(value: sonar::Track) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            artist_id: From::from(value.artist),
            album_id: From::from(value.album),
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

impl TryFrom<TrackUpdateRequest> for (sonar::TrackId, sonar::TrackUpdate) {
    type Error = tonic::Status;

    fn try_from(value: TrackUpdateRequest) -> Result<Self, Self::Error> {
        let track_id = parse_trackid(value.track_id)?;
        let album_id = parse_albumid_opt(value.album_id)?;
        let cover_art = parse_imageid_opt(value.coverart_id)?;
        let update = sonar::TrackUpdate {
            name: sonar::ValueUpdate::from_option_unchanged(value.name),
            album: sonar::ValueUpdate::from_option_unchanged(album_id),
            cover_art: sonar::ValueUpdate::from_option_unchanged(cover_art),
            lyrics: Default::default(),
            properties: convert_property_updates_from_pb(value.properties)?,
        };
        Ok((track_id, update))
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

impl From<sonar::metadata::TrackMetadata> for TrackMetadata {
    fn from(value: sonar::metadata::TrackMetadata) -> Self {
        Self {
            name: value.name,
            properties: convert_properties_to_pb(value.properties),
            cover: value.cover.map(From::from),
        }
    }
}

impl From<sonar::metadata::AlbumTracksMetadata> for MetadataAlbumTracksResponse {
    fn from(value: sonar::metadata::AlbumTracksMetadata) -> Self {
        Self {
            tracks: value
                .tracks
                .into_iter()
                .map(|(id, v)| (From::from(id), From::from(v)))
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

pub fn parse_trackid_opt(id: Option<String>) -> Result<Option<sonar::TrackId>, tonic::Status> {
    id.map(|id| id.parse::<sonar::TrackId>().m()).transpose()
}
