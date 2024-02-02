use std::sync::Arc;

use crate::{async_trait, Context, Scrobble, Username};

#[async_trait]
pub trait Scrobbler: Send + Sync + 'static {
    async fn scrobble(&self, context: &Context, scrobble: Scrobble) -> std::io::Result<()>;
}

#[derive(Clone)]
pub(crate) struct SonarScrobbler {
    identifier: String,
    username: Option<Username>,
    scrobbler: Arc<dyn Scrobbler>,
}

impl std::fmt::Debug for SonarScrobbler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SonarScrobbler")
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl SonarScrobbler {
    pub fn new(
        identifier: impl Into<String>,
        username: Option<Username>,
        scrobbler: impl Scrobbler,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            username,
            scrobbler: Arc::new(scrobbler),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub async fn scrobble(&self, context: &Context, scrobble: Scrobble) -> std::io::Result<()> {
        self.scrobbler.scrobble(context, scrobble).await
    }
}
