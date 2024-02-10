use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;

use crate::{
    async_trait, Album, Artist, Context, Error, ErrorKind, Properties, Result, Track, TrackId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataRequestKind {
    Artist,
    Album,
    AlbumTracks,
    Track,
}

#[derive(Debug, Clone)]
pub struct ArtistMetadataRequest {
    pub artist: Artist,
}

#[derive(Debug, Default, Clone)]
pub struct ArtistMetadata {
    pub name: Option<String>,
    pub properties: Properties,
    pub cover: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct AlbumMetadataRequest {
    pub artist: Artist,
    pub album: Album,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumMetadata {
    pub name: Option<String>,
    pub properties: Properties,
    pub cover: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct AlbumTracksMetadataRequest {
    pub artist: Artist,
    pub album: Album,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Default, Clone)]
pub struct AlbumTracksMetadata {
    pub tracks: HashMap<TrackId, TrackMetadata>,
}

#[derive(Debug, Clone)]
pub struct TrackMetadataRequest {
    pub artist: Artist,
    pub album: Album,
    pub track: Track,
}

#[derive(Debug, Default, Clone)]
pub struct TrackMetadata {
    pub name: Option<String>,
    pub properties: Properties,
    pub cover: Option<Bytes>,
}

#[async_trait]
#[allow(unused_variables)]
pub trait MetadataProvider: Send + Sync + 'static {
    fn supports(&self, kind: MetadataRequestKind) -> bool;
    async fn artist_metadata(
        &self,
        context: &Context,
        request: &ArtistMetadataRequest,
    ) -> Result<ArtistMetadata> {
        Err(Error::new(
            ErrorKind::Internal,
            "metadata provider does not support artist metadata",
        ))
    }
    async fn album_metadata(
        &self,
        context: &Context,
        request: &AlbumMetadataRequest,
    ) -> Result<AlbumMetadata> {
        Err(Error::new(
            ErrorKind::Internal,
            "metadata provider does not support album metadata",
        ))
    }
    async fn album_tracks_metadata(
        &self,
        context: &Context,
        request: &AlbumTracksMetadataRequest,
    ) -> Result<AlbumTracksMetadata> {
        Err(Error::new(
            ErrorKind::Internal,
            "metadata provider does not support album tracks metadata",
        ))
    }
    async fn track_metadata(
        &self,
        context: &Context,
        request: &TrackMetadataRequest,
    ) -> Result<TrackMetadata> {
        Err(Error::new(
            ErrorKind::Internal,
            "metadata provider does not support track metadata",
        ))
    }
}

#[derive(Clone)]
pub(crate) struct SonarMetadataProvider {
    name: String,
    provider: Arc<dyn MetadataProvider>,
}

impl std::fmt::Debug for SonarMetadataProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SonarMetadataProvider")
            .field("name", &self.name)
            .finish()
    }
}

impl SonarMetadataProvider {
    pub fn new(name: impl Into<String>, provider: impl MetadataProvider + 'static) -> Self {
        Self {
            name: name.into(),
            provider: Arc::new(provider),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn supports(&self, kind: MetadataRequestKind) -> bool {
        self.provider.supports(kind)
    }

    pub async fn artist_metadata(
        &self,
        context: &Context,
        request: &ArtistMetadataRequest,
    ) -> Result<ArtistMetadata> {
        self.provider.artist_metadata(context, request).await
    }

    pub async fn album_metadata(
        &self,
        context: &Context,
        request: &AlbumMetadataRequest,
    ) -> Result<AlbumMetadata> {
        self.provider.album_metadata(context, request).await
    }

    pub async fn album_tracks_metadata(
        &self,
        context: &Context,
        request: &AlbumTracksMetadataRequest,
    ) -> Result<AlbumTracksMetadata> {
        self.provider.album_tracks_metadata(context, request).await
    }

    pub async fn track_metadata(
        &self,
        context: &Context,
        request: &TrackMetadataRequest,
    ) -> Result<TrackMetadata> {
        self.provider.track_metadata(context, request).await
    }
}
