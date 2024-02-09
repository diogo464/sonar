use std::sync::Arc;

use crate::{async_trait, bytestream::ByteStream, Properties, Result};

// requirements and use cases for an external service:
// - we should be able to subscribe/unsubscribe to some type of external media.
//   ex: an artist (periodically check for new albums, tracks, etc)
//   ex: a playlist (periodically check for new tracks)
// - we should be able to download media from the external service. (one-off subscription)
// - external media is identified by some ExternalMediaId that is just a string.
//   the external service should be able to verify that the id is valid and obtain information
//   about it. (ex: this id corresponds to an artist)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalMediaId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalMediaType {
    Artist,
    Album,
    Track,
    Playlist,
}

#[derive(Clone)]
pub struct ExternalImage {
    pub data: Vec<u8>,
}

impl std::fmt::Debug for ExternalImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalImage")
            .field("data", &self.data.len())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ExternalArtist {
    pub name: String,
    pub albums: Vec<ExternalMediaId>,
    pub cover: Option<ExternalImage>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalAlbum {
    pub name: String,
    pub artist: ExternalMediaId,
    pub tracks: Vec<ExternalMediaId>,
    pub cover: Option<ExternalImage>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalTrack {
    pub name: String,
    pub artist: ExternalMediaId,
    pub album: ExternalMediaId,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalPlaylist {
    pub name: String,
    pub tracks: Vec<ExternalMediaId>,
    pub properties: Properties,
}

#[async_trait]
pub trait ExternalService: Send + Sync + 'static {
    async fn validate_id(&self, id: &ExternalMediaId) -> Result<ExternalMediaType>;
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist>;
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum>;
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack>;
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist>;
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream>;
}

impl std::fmt::Display for ExternalMediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Artist => write!(f, "artist"),
            Self::Album => write!(f, "album"),
            Self::Track => write!(f, "track"),
            Self::Playlist => write!(f, "playlist"),
        }
    }
}

impl ExternalMediaId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExternalMediaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for ExternalMediaId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ExternalMediaId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for ExternalMediaId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub(crate) struct SonarExternalService {
    priority: u32,
    identifier: String,
    service: Arc<dyn ExternalService>,
}

impl std::fmt::Debug for SonarExternalService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SonarExternalService")
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl SonarExternalService {
    pub fn new(
        priority: u32,
        identifier: impl Into<String>,
        service: impl ExternalService,
    ) -> Self {
        Self {
            priority,
            identifier: identifier.into(),
            service: Arc::new(service),
        }
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub async fn validate_id(&self, id: &ExternalMediaId) -> Result<ExternalMediaType> {
        self.service.validate_id(id).await
    }

    pub async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        self.service.fetch_artist(id).await
    }

    pub async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        self.service.fetch_album(id).await
    }

    pub async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        self.service.fetch_track(id).await
    }

    pub async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        self.service.fetch_playlist(id).await
    }

    pub async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        self.service.download_track(id).await
    }
}
