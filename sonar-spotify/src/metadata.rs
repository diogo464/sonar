use std::collections::HashMap;

use rspotify::clients::BaseClient;
use sonar::{bytes::Bytes, metadata_prelude::*, PropertyValue};
use spotdl::SpotifyId;
use tokio_stream::StreamExt;

use crate::convert_genres;

pub struct SpotifyMetadata {
    client: rspotify::ClientCredsSpotify,
    http_client: reqwest::Client,
}

impl SpotifyMetadata {
    pub async fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self> {
        let client = rspotify::ClientCredsSpotify::new(rspotify::Credentials {
            id: client_id.into(),
            secret: Some(client_secret.into()),
        });
        client.request_token().await.map_err(Error::wrap)?;
        Ok(Self {
            client,
            http_client: reqwest::Client::new(),
        })
    }

    async fn download_first_image(
        &self,
        images: &[rspotify::model::Image],
    ) -> Result<Option<Bytes>> {
        Ok(match images.first() {
            Some(image) => {
                tracing::debug!("downloading image: {}", image.url);
                Some(
                    self.http_client
                        .get(&image.url)
                        .send()
                        .await
                        .map_err(Error::wrap)?
                        .bytes()
                        .await
                        .map_err(Error::wrap)?,
                )
            }
            None => None,
        })
    }
}

#[sonar::async_trait]
impl MetadataProvider for SpotifyMetadata {
    fn supports(&self, kind: MetadataRequestKind) -> bool {
        match kind {
            MetadataRequestKind::Artist => true,
            MetadataRequestKind::Album => true,
            MetadataRequestKind::AlbumTracks => true,
            MetadataRequestKind::Track => true,
        }
    }
    async fn artist_metadata(
        &self,
        _context: &sonar::Context,
        request: &ArtistMetadataRequest,
    ) -> Result<ArtistMetadata> {
        tracing::info!(
            "fetching artist metadata for {}({})",
            request.artist.name,
            request.artist.id
        );
        let spotify_id = spotify_id_from_properties(&request.artist.properties)?;
        let artist_id = spotify_id_to_artist_id(spotify_id)?;
        let artist = self.client.artist(artist_id).await.map_err(Error::wrap)?;
        let image = self.download_first_image(&artist.images).await?;
        let metadata = ArtistMetadata {
            name: Some(artist.name),
            genres: convert_genres(artist.genres),
            properties: Default::default(),
            cover: image,
        };
        tracing::debug!("artist metadata: {:#?}", metadata);
        Ok(metadata)
    }
    async fn album_metadata(
        &self,
        _context: &sonar::Context,
        request: &AlbumMetadataRequest,
    ) -> Result<AlbumMetadata> {
        tracing::info!(
            "fetching album metadata for {}({})",
            request.album.name,
            request.album.id
        );
        let spotify_id = spotify_id_from_properties(&request.album.properties)?;
        let album_id = spotify_id_to_album_id(spotify_id)?;
        let album = self
            .client
            .album(album_id, None)
            .await
            .map_err(Error::wrap)?;
        let image = self.download_first_image(&album.images).await?;
        let metadata = AlbumMetadata {
            name: Some(album.name),
            genres: convert_genres(album.genres),
            properties: Default::default(),
            cover: image,
        };
        tracing::debug!("album metadata: {:#?}", metadata);
        Ok(metadata)
    }
    async fn album_tracks_metadata(
        &self,
        _context: &sonar::Context,
        request: &AlbumTracksMetadataRequest,
    ) -> Result<AlbumTracksMetadata> {
        tracing::info!(
            "fetching album tracks metadata for {}({})",
            request.album.name,
            request.album.id
        );
        let spotify_id = spotify_id_from_properties(&request.album.properties)?;
        let album_id = spotify_id_to_album_id(spotify_id)?;
        let mut stream = self.client.album_track(album_id, None);
        let mut tracks = HashMap::new();

        while let Some(Ok(track)) = stream.next().await {
            tracing::debug!("found track: {:?}", track);
            let id = match track.id {
                Some(ref id) => id.to_string(),
                None => continue,
            };
            let t = request.tracks.iter().find(|track| {
                track
                    .properties
                    .get(sonar::prop::EXTERNAL_SPOTIFY_ID)
                    .map(|sid| sid.as_str() == id)
                    .unwrap_or(false)
            });
            let t = match t {
                Some(t) => t,
                None => continue,
            };
            tracks.insert(t.id, simplified_track_to_track_metadata(track));
        }

        let metadata = AlbumTracksMetadata { tracks };
        tracing::debug!("album tracks metadata: {:#?}", metadata);
        Ok(metadata)
    }
    async fn track_metadata(
        &self,
        _context: &sonar::Context,
        request: &TrackMetadataRequest,
    ) -> Result<TrackMetadata> {
        tracing::info!(
            "fetching track metadata for {}({})",
            request.track.name,
            request.track.id
        );
        let spotify_id = spotify_id_from_properties(&request.track.properties)?;
        let track_id = spotify_id_to_track_id(spotify_id)?;
        let track = self
            .client
            .track(track_id, None)
            .await
            .map_err(Error::wrap)?;
        let metadata = full_track_to_track_metadata(track);
        tracing::debug!("track metadata: {:#?}", metadata);
        Ok(metadata)
    }
}

fn spotify_id_from_properties(properties: &sonar::Properties) -> Result<SpotifyId> {
    properties
        .get_parsed(sonar::prop::EXTERNAL_SPOTIFY_ID)
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Invalid,
                format!(
                    "missing required property: {}",
                    sonar::prop::EXTERNAL_SPOTIFY_ID
                ),
            )
        })
}

fn spotify_id_to_artist_id(spotify_id: SpotifyId) -> Result<rspotify::model::ArtistId<'static>> {
    rspotify::model::ArtistId::from_id(spotify_id.to_string()).map_err(|_| {
        Error::new(
            ErrorKind::Invalid,
            format!("invalid spotify id: {}", spotify_id),
        )
    })
}

fn spotify_id_to_album_id(spotify_id: SpotifyId) -> Result<rspotify::model::AlbumId<'static>> {
    rspotify::model::AlbumId::from_id(spotify_id.to_string()).map_err(|_| {
        Error::new(
            ErrorKind::Invalid,
            format!("invalid spotify id: {}", spotify_id),
        )
    })
}

fn spotify_id_to_track_id(spotify_id: SpotifyId) -> Result<rspotify::model::TrackId<'static>> {
    rspotify::model::TrackId::from_id(spotify_id.to_string()).map_err(|_| {
        Error::new(
            ErrorKind::Invalid,
            format!("invalid spotify id: {}", spotify_id),
        )
    })
}

fn simplified_track_to_track_metadata(track: rspotify::model::SimplifiedTrack) -> TrackMetadata {
    let mut properties = Properties::default();
    properties.insert(
        sonar::prop::DISC_NUMBER,
        PropertyValue::new(track.disc_number.to_string()).unwrap(),
    );
    properties.insert(
        sonar::prop::TRACK_NUMBER,
        PropertyValue::new(track.track_number.to_string()).unwrap(),
    );
    TrackMetadata {
        name: Some(track.name),
        properties,
        cover: None,
    }
}

fn full_track_to_track_metadata(track: rspotify::model::FullTrack) -> TrackMetadata {
    let mut properties = Properties::default();
    properties.insert(
        sonar::prop::DISC_NUMBER,
        PropertyValue::new(track.disc_number.to_string()).unwrap(),
    );
    properties.insert(
        sonar::prop::TRACK_NUMBER,
        PropertyValue::new(track.track_number.to_string()).unwrap(),
    );
    TrackMetadata {
        name: Some(track.name),
        properties,
        cover: None,
    }
}
