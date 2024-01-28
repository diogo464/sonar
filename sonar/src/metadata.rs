use std::{path::Path, sync::Arc, time::Duration};

use crate::{DateTime, Genres};

#[derive(Debug, Clone)]
pub struct ExtractedImage {
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Option<Duration>,
    pub release_date: Option<DateTime>,
    pub cover_art: Option<ExtractedImage>,
    pub genres: Genres,
}

pub trait Extractor: Send + Sync + 'static {
    fn extract(&self, path: &Path) -> std::io::Result<ExtractedMetadata>;
}

#[derive(Clone)]
pub(crate) struct SonarExtractor {
    name: String,
    extractor: Arc<dyn Extractor>,
}

impl std::fmt::Debug for SonarExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SonarExtractor")
            .field("name", &self.name)
            .finish()
    }
}

impl SonarExtractor {
    pub fn new(name: impl Into<String>, extractor: impl Extractor) -> Self {
        Self {
            name: name.into(),
            extractor: Arc::new(extractor),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn extract(&self, path: &Path) -> std::io::Result<ExtractedMetadata> {
        self.extractor.extract(path)
    }
}
