use serde::Deserialize;
use sonar::{
    bytestream::ByteStream, Error, ErrorKind, ExternalAlbum, ExternalArtist,
    ExternalMediaEnrichStatus, ExternalMediaId, ExternalMediaRequest, ExternalMediaType,
    ExternalPlaylist, ExternalTrack, MultiExternalMediaId, Result,
};

mod rate_limiter;
use rate_limiter::*;

const USER_AGENT: &str = "sonar/1.0.0 ( diogo464@d464.sh )";

#[derive(Debug)]
pub struct MusicBrainzService {
    client: reqwest::Client,
    limiter: RateLimiter,
}

impl Default for MusicBrainzService {
    fn default() -> Self {
        Self {
            client: Default::default(),
            limiter: RateLimiter::new(1.0),
        }
    }
}

impl MusicBrainzService {
    async fn expand_recording(&self, ids: MultiExternalMediaId) -> MultiExternalMediaId {
        #[derive(Debug, Deserialize)]
        struct Response {
            relations: Vec<ResponseRelation>,
        }
        #[derive(Debug, Deserialize)]
        struct ResponseRelation {
            url: Option<ResponseRelationUrl>,
        }
        #[derive(Debug, Deserialize)]
        struct ResponseRelationUrl {
            resource: String,
        }

        let mut external_ids = Vec::with_capacity(ids.len());
        for id in ids {
            external_ids.push(id.clone());
            if !id_is_mbid(id.as_str()) {
                continue;
            }

            tracing::debug!("waiting for rate limiter");
            self.limiter.request().await;

            tracing::debug!("looking up recording: {id}");
            let response = self
                .client
                .get(format!(
                    "https://musicbrainz.org/ws/2/recording/{}?inc=url-rels",
                    id.as_str()
                ))
                .header("Accept", "application/json")
                .header("User-Agent", USER_AGENT)
                .send()
                .await
                .unwrap();

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                continue;
            }

            let content = response.text().await.unwrap();
            tracing::debug!("recording response: {content}");
            let response: Response = serde_json::from_str(&content).unwrap();
            for relation in response.relations {
                if let Some(url) = relation.url {
                    external_ids.push(ExternalMediaId::from(url.resource));
                }
            }
        }
        MultiExternalMediaId::from(external_ids)
    }
}

#[sonar::async_trait]
impl sonar::ExternalService for MusicBrainzService {
    #[tracing::instrument(skip(self))]
    async fn enrich(
        &self,
        request: &mut ExternalMediaRequest,
    ) -> Result<ExternalMediaEnrichStatus> {
        Ok(ExternalMediaEnrichStatus::NotModified)
    }
    #[tracing::instrument(skip(self))]
    async fn extract(
        &self,
        request: &ExternalMediaRequest,
    ) -> Result<(ExternalMediaType, ExternalMediaId)> {
        Err(Error::new(ErrorKind::Internal, "not implemented"))
    }
    async fn fetch_artist(&self, _id: &ExternalMediaId) -> Result<ExternalArtist> {
        Err(Error::new(
            ErrorKind::Invalid,
            "musicbrainz service does not support fetching artists",
        ))
    }
    async fn fetch_album(&self, _id: &ExternalMediaId) -> Result<ExternalAlbum> {
        Err(Error::new(
            ErrorKind::Invalid,
            "musicbrainz service does not support fetching albums",
        ))
    }
    async fn fetch_track(&self, _id: &ExternalMediaId) -> Result<ExternalTrack> {
        Err(Error::new(
            ErrorKind::Invalid,
            "musicbrainz service does not support fetching tracks",
        ))
    }
    async fn fetch_playlist(&self, _id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        Err(Error::new(
            ErrorKind::Invalid,
            "musicbrainz service does not support fetching playlists",
        ))
    }
    async fn download_track(&self, _id: &ExternalMediaId) -> Result<ByteStream> {
        Err(Error::new(
            ErrorKind::Invalid,
            "musicbrainz service does not support downloading tracks",
        ))
    }
}

fn id_is_mbid(id: &str) -> bool {
    let id = id.as_bytes();
    if id.len() != 36 {
        return false;
    }
    if id[8] != id[13] || id[13] != id[18] || id[18] != id[23] || id[23] != b'-' {
        return false;
    }
    id.iter()
        .all(|c| c.is_ascii_digit() || (*c >= b'a' && *c <= b'f') || *c == b'-')
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_expand() {
        let service = MusicBrainzService::default();
        let recording_mbid = ExternalMediaId::from("307ce9da-5690-4e21-ab71-9d12ea106e52");
        let expanded = service
            .expand_recording(MultiExternalMediaId::from(recording_mbid))
            .await;
        insta::assert_debug_snapshot!(expanded);
    }

    #[test]
    fn test_id_is_mbid() {
        assert!(id_is_mbid("307ce9da-5690-4e21-ab71-9d12ea106e52"));
        assert!(!id_is_mbid("spotify:artist:762310PdDnwsDxAQxzQkfX"));
    }
}
