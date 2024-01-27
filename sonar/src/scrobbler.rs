use crate::{async_trait, Scrobble};

#[async_trait]
pub trait Scrobbler: Send + Sync + 'static {
    async fn scrobble(&self, scrobble: Scrobble) -> std::io::Result<()>;
}

pub(crate) struct SonarScrobbler {
    identifier: String,
    scrobbler: Box<dyn Scrobbler>,
}

impl std::fmt::Debug for SonarScrobbler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SonarScrobbler")
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl SonarScrobbler {
    pub fn new(identifier: impl Into<String>, scrobbler: impl Scrobbler) -> Self {
        Self {
            identifier: identifier.into(),
            scrobbler: Box::new(scrobbler),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub async fn scrobble(&self, scrobble: Scrobble) -> std::io::Result<()> {
        self.scrobbler.scrobble(scrobble).await
    }
}
