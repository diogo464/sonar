use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ListenBrainzScrobbler {
    client: ListenBrainzClient,
}

impl ListenBrainzScrobbler {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            client: ListenBrainzClient::new(token),
        }
    }
}

#[derive(Debug, Clone)]
struct ListenBrainzClient {
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feedback {
    Love,
    Neutral,
}

#[derive(Debug, Serialize)]
struct Submit<'a> {
    listen_type: &'static str,
    payload: &'a [SubmitPayload<'a>],
}

#[derive(Debug, Serialize)]
struct SubmitPayload<'a> {
    listened_at: u64,
    track_metadata: SubmitTrackMetadata<'a>,
}

#[derive(Debug, Serialize)]
struct SubmitTrackMetadata<'a> {
    artist_name: &'a str,
    track_name: &'a str,
    album_name: &'a str,
    additional_info: SubmitAdditionalInfo<'a>,
}

#[derive(Debug, Serialize)]
struct SubmitAdditionalInfo<'a> {
    submission_client: &'a str,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    recording_mbid: Option<&'a str>,
}

#[derive(Serialize)]
struct SubmitFeedbackBody<'a> {
    recording_mbid: &'a str,
    score: &'a str,
}

#[derive(Debug, Deserialize)]
struct LookupResponse {
    // Example output as of Fri Apr 19 07:12:56 PM WEST 2024
    // {
    //   "artist_credit_name": "System of a Down",
    //   "artist_mbids": [
    //     "cc0b7089-c08d-4c10-b6b0-873582c17fd6"
    //   ],
    //   "recording_mbid": "9563c521-5b7d-4db9-be73-1a2abb99ccd2",
    //   "recording_name": "Aerials",
    //   "release_mbid": "b8e92589-8b7c-4d0a-9986-02d129997e04",
    //   "release_name": "Toxicity"
    // }
    recording_mbid: Option<String>,
}

impl ListenBrainzClient {
    fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: Default::default(),
        }
    }

    async fn recording_feedback(
        &self,
        recording_mbid: &str,
        feedback: Feedback,
    ) -> std::io::Result<()> {
        let score = match feedback {
            Feedback::Love => "1",
            Feedback::Neutral => "0",
        };
        let body = SubmitFeedbackBody {
            recording_mbid,
            score,
        };
        self.client
            .post("https://api.listenbrainz.org/1/feedback/recording-feedback")
            .header("Authorization", format!("Token {}", self.token))
            .json(&body)
            .send()
            .await
            .map_err(std::io::Error::other)?
            .error_for_status()
            .map_err(std::io::Error::other)?;
        Ok(())
    }

    /// returns a recording mbid
    async fn lookup(&self, artist_name: &str, track_name: &str) -> std::io::Result<Option<String>> {
        let request = self
            .client
            .get("https://api.listenbrainz.org/1/metadata/lookup")
            .header("Authorization", format!("Token {}", self.token))
            .query(&[("artist_name", artist_name), ("recording_name", track_name)]);
        let response = request
            .send()
            .await
            .map_err(std::io::Error::other)?
            .error_for_status()
            .map_err(std::io::Error::other)?
            .json::<LookupResponse>()
            .await
            .map_err(std::io::Error::other)?;
        Ok(response.recording_mbid)
    }

    async fn submit_listen(
        &self,
        artist_name: &str,
        album_name: &str,
        track_name: &str,
        listened_at: u64,
        track_duration: Duration,
        recording_mbid: Option<&str>,
    ) -> std::io::Result<()> {
        let submit = Submit {
            listen_type: "import",
            payload: &[SubmitPayload {
                listened_at,
                track_metadata: SubmitTrackMetadata {
                    artist_name,
                    album_name,
                    track_name,
                    additional_info: SubmitAdditionalInfo {
                        submission_client: "sonar",
                        duration_ms: track_duration.as_millis() as u64,
                        recording_mbid,
                    },
                },
            }],
        };

        tracing::debug!("submitting to ListenBrainz: {:#?}", submit);
        let response = self
            .client
            .post("https://api.listenbrainz.org/1/submit-listens")
            .header("Authorization", format!("Token {}", self.token))
            .json(&submit)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        tracing::trace!("response = {response:#?}");

        if response.status() == reqwest::StatusCode::OK {
            tracing::info!("scrobbled successfully to ListenBrainz");
            return Ok(());
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            tracing::info!("rate limited by ListenBrainz");
            if let Some(reset_in) = response.headers().get("X-RateLimit-Reset-In") {
                let reset_in = reset_in
                    .to_str()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                    .parse::<u64>()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                tokio::time::sleep(std::time::Duration::from_secs(reset_in)).await;
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Rate limited",
            ));
        } else {
            tracing::warn!("failed to scrobble to ListenBrainz: {:#?}", response);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unexpected status code: {}", response.status()),
            ));
        }
    }
}

#[sonar::async_trait]
impl sonar::Scrobbler for ListenBrainzScrobbler {
    #[tracing::instrument(skip(context,scrobble),fields(scrobble=scrobble.id.to_string()))]
    async fn scrobble(
        &self,
        context: &sonar::Context,
        scrobble: sonar::Scrobble,
    ) -> std::io::Result<()> {
        tracing::info!("scrobbling to ListenBrainz: {:#?}", scrobble);
        let track = sonar::track_get(context, scrobble.track)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let album = sonar::album_get(context, track.album)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let artist = sonar::artist_get(context, album.artist)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let artist_name = artist.name.as_str();
        let album_name = album.name.as_str();
        let track_name = track.name.as_str();
        let listen_at = scrobble.listen_at.seconds();
        let favorited =
            sonar::favorite_find(context, scrobble.user, sonar::SonarId::from(track.id))
                .await
                .map_err(std::io::Error::other)?
                .is_some();

        let mbid = self.client.lookup(&artist_name, &track_name).await?;
        let feedback = match favorited {
            true => Feedback::Love,
            false => Feedback::Neutral,
        };
        tracing::debug!("track {artist_name}/{album_name}/{track_name} has mbid {mbid:?} and favorited = {favorited}");

        let mbid = mbid.as_ref().map(|v| v.as_str());

        if let Some(mbid) = mbid {
            self.client.recording_feedback(&mbid, feedback).await?;
        }

        self.client
            .submit_listen(
                &artist_name,
                &album_name,
                &track_name,
                listen_at,
                track.duration,
                mbid,
            )
            .await?;

        Ok(())
    }
}

#[doc(hidden)]
pub async fn test_main() {
    tracing_subscriber::fmt::init();

    let token = "0f0494b8-f720-4fd3-b052-a5e432320501";
    let client = ListenBrainzClient::new(token);
    client.lookup("System of a Down", "Aerials").await.unwrap();
    client
        .recording_feedback("616d5df1-cef3-4dd3-b796-5065e0e4d5ba", Feedback::Love)
        .await
        .unwrap();
    client
        .submit_listen(
            "Metallica",
            "Metallica",
            "Enter Sandman",
            1713558174,
            Duration::from_secs(331),
            None,
        )
        .await
        .unwrap();
}
