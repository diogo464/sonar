use std::{sync::Arc, time::Duration};

use crate::{
    async_trait, bytestream::ByteStream, Error, ErrorKind, Genres, Properties, Result, TrackLyrics,
};

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

#[derive(Debug, Clone, Hash)]
pub struct MultiExternalMediaId(Vec<ExternalMediaId>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalMediaType {
    Artist,
    Album,
    Track,
    Playlist,
    Compilation,
    Group,
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
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalAlbum {
    pub name: String,
    pub artist: ExternalMediaId,
    pub tracks: Vec<ExternalMediaId>,
    pub cover: Option<ExternalImage>,
    pub genres: Genres,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalTrack {
    pub name: String,
    pub artist: ExternalMediaId,
    pub album: ExternalMediaId,
    pub lyrics: Option<TrackLyrics>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalPlaylist {
    pub name: String,
    pub tracks: Vec<ExternalMediaId>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalCompilation {
    pub name: String,
    pub tracks: Vec<ExternalCompilationTrack>,
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct ExternalCompilationTrack {
    pub artist: String,
    pub album: String,
    pub track: String,
}

#[derive(Debug, Default, Clone)]
pub struct ExternalMediaRequest {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<String>,
    pub playlist: Option<String>,
    pub duration: Option<Duration>,
    pub media_type: Option<ExternalMediaType>,
    pub external_ids: Vec<ExternalMediaId>,
}

impl ExternalMediaRequest {
    pub fn merge(&mut self, mut other: ExternalMediaRequest) -> ExternalMediaEnrichStatus {
        fn merge_field<T>(
            left: &mut Option<T>,
            right: Option<T>,
            status: &mut ExternalMediaEnrichStatus,
        ) {
            if left.is_none() && right.is_some() {
                *left = right;
                *status = ExternalMediaEnrichStatus::Modified;
            }
        }

        let mut status = ExternalMediaEnrichStatus::NotModified;
        merge_field(&mut self.artist, other.artist, &mut status);
        merge_field(&mut self.album, other.album, &mut status);
        merge_field(&mut self.track, other.track, &mut status);
        merge_field(&mut self.playlist, other.playlist, &mut status);
        merge_field(&mut self.duration, other.duration, &mut status);
        merge_field(&mut self.media_type, other.media_type, &mut status);

        other
            .external_ids
            .retain(|v| !self.external_ids.contains(v));
        if !other.external_ids.is_empty() {
            self.external_ids.extend(other.external_ids);
            status = ExternalMediaEnrichStatus::Modified;
        }

        status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalMediaEnrichStatus {
    Modified,
    NotModified,
}

#[async_trait]
#[allow(unused)]
pub trait ExternalService: Send + Sync + 'static {
    async fn enrich(
        &self,
        request: &mut ExternalMediaRequest,
    ) -> Result<ExternalMediaEnrichStatus> {
        Ok(ExternalMediaEnrichStatus::NotModified)
    }
    async fn extract(
        &self,
        request: &ExternalMediaRequest,
    ) -> Result<(ExternalMediaType, ExternalMediaId)> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_compilation(&self, id: &ExternalMediaId) -> Result<ExternalCompilation> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn fetch_group(&self, id: &ExternalMediaId) -> Result<Vec<ExternalMediaId>> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        Err(Error::new(ErrorKind::Invalid, "not supported"))
    }
}

struct ExternalServicesInner {
    services: Vec<ExternalServicesEntry>,
}

impl std::fmt::Debug for ExternalServicesInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalServicesInner").finish()
    }
}

pub(crate) struct ExternalServicesEntry {
    pub identifier: String,
    pub service: Box<dyn ExternalService>,
    pub priority: u32,
}

impl std::fmt::Debug for ExternalServicesEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalServicesEntry")
            .field("identifier", &self.identifier)
            .field("priority", &self.priority)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ExternalServices(Arc<ExternalServicesInner>);

impl ExternalServices {
    pub fn new(entries: impl IntoIterator<Item = ExternalServicesEntry>) -> Self {
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|s| s.priority);
        Self(Arc::new(ExternalServicesInner { services: entries }))
    }

    pub async fn enrich(&self, request: &mut ExternalMediaRequest) -> Result<()> {
        let mut status = ExternalMediaEnrichStatus::Modified;
        while status == ExternalMediaEnrichStatus::Modified {
            status = ExternalMediaEnrichStatus::NotModified;
            for service in self.services() {
                if service.enrich(request).await? == ExternalMediaEnrichStatus::Modified {
                    status = ExternalMediaEnrichStatus::Modified;
                }
            }
        }
        Ok(())
    }

    pub async fn extract(
        &self,
        request: &ExternalMediaRequest,
    ) -> Result<(&dyn ExternalService, ExternalMediaType, ExternalMediaId)> {
        for service in self.services() {
            if let Ok((media_type, external_id)) = service.extract(request).await {
                return Ok((service, media_type, external_id));
            }
        }
        tracing::warn!("failed to extract request: {request:#?}");
        Err(Error::new(ErrorKind::Invalid, "failed to extract request"))
    }

    pub fn services(&self) -> impl Iterator<Item = &dyn ExternalService> {
        self.0.services.iter().map(|v| &*v.service)
    }
}

impl<'a> IntoIterator for &'a ExternalServices {
    type Item = <&'a [ExternalServicesEntry] as IntoIterator>::Item;

    type IntoIter = <&'a [ExternalServicesEntry] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.services.as_slice().iter()
    }
}

impl std::fmt::Display for ExternalMediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Artist => write!(f, "artist"),
            Self::Album => write!(f, "album"),
            Self::Track => write!(f, "track"),
            Self::Playlist => write!(f, "playlist"),
            Self::Compilation => write!(f, "compilation"),
            Self::Group => write!(f, "group"),
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

impl MultiExternalMediaId {
    pub fn new(ids: impl IntoIterator<Item = ExternalMediaId>) -> Self {
        Self(ids.into_iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ExternalMediaId> {
        self.0.iter()
    }
}

impl From<Vec<ExternalMediaId>> for MultiExternalMediaId {
    fn from(value: Vec<ExternalMediaId>) -> Self {
        Self(value)
    }
}

impl From<MultiExternalMediaId> for Vec<ExternalMediaId> {
    fn from(value: MultiExternalMediaId) -> Self {
        value.0
    }
}

impl From<ExternalMediaId> for MultiExternalMediaId {
    fn from(value: ExternalMediaId) -> Self {
        Self::from(vec![value])
    }
}

impl<'a> IntoIterator for &'a MultiExternalMediaId {
    type Item = &'a ExternalMediaId;

    type IntoIter = <&'a Vec<ExternalMediaId> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for MultiExternalMediaId {
    type Item = ExternalMediaId;

    type IntoIter = <Vec<ExternalMediaId> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<ExternalMediaId> for ExternalMediaRequest {
    fn from(value: ExternalMediaId) -> Self {
        Self {
            external_ids: vec![value],
            ..Default::default()
        }
    }
}

// #[derive(Clone)]
// pub(crate) struct SonarExternalService {
//     priority: u32,
//     identifier: String,
//     service: Arc<dyn ExternalService>,
// }
//
// impl std::fmt::Debug for SonarExternalService {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("SonarExternalService")
//             .field("identifier", &self.identifier)
//             .finish()
//     }
// }
//
// impl SonarExternalService {
//     pub fn new(
//         priority: u32,
//         identifier: impl Into<String>,
//         service: impl ExternalService,
//     ) -> Self {
//         Self {
//             priority,
//             identifier: identifier.into(),
//             service: Arc::new(service),
//         }
//     }
//
//     pub fn priority(&self) -> u32 {
//         self.priority
//     }
//
//     pub fn identifier(&self) -> &str {
//         &self.identifier
//     }
//
//     pub async fn enrich(
//         &self,
//         request: &mut ExternalMediaRequest,
//     ) -> Result<ExternalMediaEnrichStatus> {
//         self.service.enrich(request).await
//     }
//
//     pub async fn extract(
//         &self,
//         request: &ExternalMediaRequest,
//     ) -> Result<(ExternalMediaType, ExternalMediaId)> {
//         self.service.extract(request).await
//     }
//
//     pub async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
//         self.service.fetch_artist(id).await
//     }
//
//     pub async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
//         self.service.fetch_album(id).await
//     }
//
//     pub async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
//         self.service.fetch_track(id).await
//     }
//
//     pub async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
//         self.service.fetch_playlist(id).await
//     }
//
//     pub async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
//         self.service.download_track(id).await
//     }
// }
