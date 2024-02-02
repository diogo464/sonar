use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ListenBrainzScrobbler {
    token: String,
    client: reqwest::Client,
}

impl ListenBrainzScrobbler {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: reqwest::Client::new(),
        }
    }
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
}

#[sonar::async_trait]
impl sonar::scrobbler::Scrobbler for ListenBrainzScrobbler {
    async fn scrobble(
        &self,
        context: &sonar::Context,
        scrobble: sonar::Scrobble,
    ) -> std::io::Result<()> {
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

        let submit = Submit {
            listen_type: "import",
            payload: &[SubmitPayload {
                listened_at: listen_at,
                track_metadata: SubmitTrackMetadata {
                    artist_name,
                    album_name,
                    track_name,
                    additional_info: SubmitAdditionalInfo {
                        submission_client: "sonar",
                        duration_ms: track.duration.as_millis() as u64,
                    },
                },
            }],
        };

        let response = self
            .client
            .post("https://api.listenbrainz.org/1/submit-listens")
            .header("Authorization", format!("Token {}", self.token))
            .json(&submit)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if response.status() == reqwest::StatusCode::OK {
            return Ok(());
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unexpected status code: {}", response.status()),
            ));
        }
    }
}
