use std::{path::Path, sync::Arc, time::Duration};

use rspotify::clients::BaseClient;
use sonar::{
    bytestream::ByteStream, Error, ErrorKind, ExternalAlbum, ExternalArtist, ExternalImage,
    ExternalMediaEnrichStatus, ExternalMediaId, ExternalMediaRequest, ExternalMediaType,
    ExternalPlaylist, ExternalService, ExternalTrack, MultiExternalMediaId, Result,
};
use spotdl::{LoginCredentials, Resource, ResourceId, SpotifyId};
use tokio::sync::Semaphore;

use crate::{convert, convert_genres};

const MAX_CONCURRENT_TRACK_DOWNLOADS: usize = 3;

pub struct SpotifyService {
    client: rspotify::ClientCredsSpotify,
    session: spotdl::session::Session,
    fetcher: Arc<dyn spotdl::fetcher::MetadataFetcher>,
    download_sem: Arc<Semaphore>,
}

impl SpotifyService {
    pub async fn new(
        credentials: LoginCredentials,
        cache_dir: &Path,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self> {
        let client = rspotify::ClientCredsSpotify::new(rspotify::Credentials {
            id: client_id.into(),
            secret: Some(client_secret.into()),
        });
        client.request_token().await.map_err(Error::wrap)?;

        let credentials = spotdl::session::login(&credentials)
            .await
            .map_err(sonar::Error::wrap)?;

        let session = spotdl::session::Session::connect(credentials)
            .await
            .map_err(sonar::Error::wrap)?;

        let fetcher = spotdl::fetcher::SpotifyMetadataFetcher::new(session.clone());
        let fetcher =
            spotdl::fetcher::FsCacheMetadataFetcher::new(fetcher, cache_dir.to_path_buf())
                .await
                .map_err(sonar::Error::wrap)?;
        let fetcher = Arc::new(fetcher);

        Ok(Self {
            client,
            session,
            fetcher,
            download_sem: Arc::new(Semaphore::new(MAX_CONCURRENT_TRACK_DOWNLOADS)),
        })
    }

    async fn enrich_by_id(
        &self,
        request: &mut ExternalMediaRequest,
        resource_id: ResourceId,
    ) -> Result<ExternalMediaEnrichStatus> {
        let mut canvas = ExternalMediaRequest::default();
        let external_id = ExternalMediaId::from(resource_id.to_uri());
        match resource_id.resource {
            Resource::Artist => {
                let artist = self.fetch_artist(&external_id).await?;
                canvas.artist = Some(artist.name);
                canvas.media_type = Some(ExternalMediaType::Artist);
            }
            Resource::Album => {
                let album = self.fetch_album(&external_id).await?;
                let artist = self.fetch_artist(&album.artist).await?;
                canvas.artist = Some(artist.name);
                canvas.album = Some(album.name);
                canvas.media_type = Some(ExternalMediaType::Album);
            }
            Resource::Track => {
                let track = self.fetch_track(&external_id).await?;
                let album = self.fetch_album(&track.album).await?;
                let artist = self.fetch_artist(&track.artist).await?;
                canvas.artist = Some(artist.name);
                canvas.album = Some(album.name);
                canvas.track = Some(track.name);
                canvas.media_type = Some(ExternalMediaType::Track);
            }
            Resource::Playlist => {
                let playlist = self.fetch_playlist(&external_id).await?;
                canvas.playlist = Some(playlist.name);
                canvas.media_type = Some(ExternalMediaType::Playlist);
            }
        }
        Ok(request.merge(canvas))
    }

    async fn enrich_by_search(
        &self,
        request: &mut ExternalMediaRequest,
    ) -> Result<ExternalMediaEnrichStatus> {
        let media_type = match request.media_type {
            Some(media_type) => media_type,
            None => return Ok(ExternalMediaEnrichStatus::NotModified),
        };

        let (query, search_type) = match media_type {
            ExternalMediaType::Artist => {
                let artist_name = match request.artist.as_ref() {
                    Some(artist_name) => artist_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                (
                    format!("{}", artist_name),
                    rspotify::model::SearchType::Artist,
                )
            }
            ExternalMediaType::Album => {
                let artist_name = match request.artist.as_ref() {
                    Some(artist_name) => artist_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                let album_name = match request.album.as_ref() {
                    Some(album_name) => album_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                (
                    format!("{} {}", artist_name, album_name),
                    rspotify::model::SearchType::Album,
                )
            }
            ExternalMediaType::Track => {
                let artist_name = match request.artist.as_ref() {
                    Some(artist_name) => artist_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                let album_name = match request.album.as_ref() {
                    Some(album_name) => album_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                let track_name = match request.track.as_ref() {
                    Some(track_name) => track_name,
                    None => return Ok(ExternalMediaEnrichStatus::NotModified),
                };
                (
                    format!("{} {} {}", artist_name, album_name, track_name),
                    rspotify::model::SearchType::Track,
                )
            }
            _ => return Ok(ExternalMediaEnrichStatus::NotModified),
        };

        let search_result = self
            .client
            .search(&query, search_type, None, None, Some(10), Some(0))
            .await
            .map_err(Error::wrap)?;

        let mut canvas = ExternalMediaRequest::default();
        match search_result {
            rspotify::model::SearchResult::Artists(page) => {
                tracing::trace!("{page:#?}");
                if let Some(artist) = page.items.first() {
                    canvas
                        .external_ids
                        .push(ExternalMediaId::new(artist.id.to_string()));
                }
            }
            rspotify::model::SearchResult::Albums(page) => {
                tracing::trace!("{page:#?}");
                if let Some(album) = page.items.first()
                    && let Some(ref album_id) = album.id
                    && &album.artists[0].name == request.artist.as_ref().unwrap()
                {
                    canvas
                        .external_ids
                        .push(ExternalMediaId::new(album_id.to_string()));
                }
            }
            rspotify::model::SearchResult::Tracks(page) => {
                tracing::trace!("{page:#?}");
                if let Some(track) = Self::pick_best_track_for_request(request, &page.items) {
                    let track_id = track
                        .id
                        .as_ref()
                        .expect("rspotify full track did not have id");
                    canvas
                        .external_ids
                        .push(ExternalMediaId::new(track_id.to_string()));
                }
            }
            _ => {}
        }

        Ok(request.merge(canvas))
    }

    fn pick_best_track_for_request<'t>(
        request: &'_ ExternalMediaRequest,
        tracks: &'t [rspotify::model::FullTrack],
    ) -> Option<&'t rspotify::model::FullTrack> {
        let match_album = |t: &rspotify::model::FullTrack| {
            let request_album = match request.album {
                Some(ref name) => name,
                None => return true,
            };
            request_album.eq_ignore_ascii_case(&t.album.name)
                || t.album
                    .name
                    .to_lowercase()
                    .contains(&request_album.to_lowercase())
        };
        let match_track = |t: &rspotify::model::FullTrack| {
            let request_track = match request.track {
                Some(ref name) => name,
                None => return false,
            };
            request_track.eq_ignore_ascii_case(&t.name)
        };
        tracks
            .into_iter()
            .filter(|t| match_album(t))
            .filter(|t| match_track(t))
            .next()
            .or(tracks.into_iter().filter(|t| match_album(t)).next())
    }
}

#[sonar::async_trait]
impl sonar::ExternalService for SpotifyService {
    #[tracing::instrument(skip(self))]
    async fn enrich(
        &self,
        request: &mut ExternalMediaRequest,
    ) -> Result<ExternalMediaEnrichStatus> {
        if let Some(resource_id) = fix_and_extract_resource_id(request) {
            self.enrich_by_id(request, resource_id).await
        } else {
            self.enrich_by_search(request).await
        }
    }

    #[tracing::instrument(skip(self))]
    async fn extract(
        &self,
        request: &ExternalMediaRequest,
    ) -> Result<(ExternalMediaType, ExternalMediaId)> {
        for external_id in &request.external_ids {
            if let Ok(resource_id) = parse_resource_id(&external_id) {
                let media_type = match resource_id.resource {
                    Resource::Artist => ExternalMediaType::Artist,
                    Resource::Album => ExternalMediaType::Album,
                    Resource::Track => ExternalMediaType::Track,
                    Resource::Playlist => ExternalMediaType::Playlist,
                };
                return Ok((media_type, external_id.clone()));
            }
        }
        return Err(Error::new(
            ErrorKind::Invalid,
            "no valid spotify id in request",
        ));
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_artist(&self, id: &ExternalMediaId) -> Result<ExternalArtist> {
        tracing::info!("fetching artist: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Artist {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid artist id",
            ));
        }
        let artist = self
            .fetcher
            .get_artist(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("artist: {:#?}", artist);
        Ok(ExternalArtist {
            name: artist.name,
            albums: artist
                .albums
                .into_iter()
                .chain(artist.singles)
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            cover: None,
            genres: convert_genres(artist.genres),
            properties: properties_for_resource(resource_id.id),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_album(&self, id: &ExternalMediaId) -> Result<ExternalAlbum> {
        tracing::info!("fetching album: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Album {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid album id",
            ));
        }
        let album = self
            .fetcher
            .get_album(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("album: {:#?}", album);
        let cover = match album.cover {
            Some(url) => Some(ExternalImage {
                data: reqwest::get(url)
                    .await
                    .map_err(sonar::Error::wrap)?
                    .bytes()
                    .await
                    .map_err(sonar::Error::wrap)?
                    .to_vec(),
            }),
            None => None,
        };
        let external = ExternalAlbum {
            name: album.name,
            artist: ExternalMediaId::new(album.artists[0].to_string()),
            tracks: album
                .discs
                .into_iter()
                .flat_map(|disc| disc.tracks)
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            cover,
            genres: Default::default(),
            properties: properties_for_resource(resource_id.id),
        };
        tracing::debug!("external album: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_track(&self, id: &ExternalMediaId) -> Result<ExternalTrack> {
        tracing::info!("fetching track: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Track {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }
        let track = self
            .fetcher
            .get_track(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("track: {:#?}", track);

        let lyrics = match track.lyrics {
            Some(lyrics) => Some(match lyrics.kind {
                spotdl::metadata::LyricsKind::Unsynchronized(u) => sonar::TrackLyrics {
                    kind: sonar::LyricsKind::Unsynced,
                    lines: u
                        .into_iter()
                        .map(|line| sonar::LyricsLine {
                            offset: Default::default(),
                            duration: Default::default(),
                            text: line,
                        })
                        .collect(),
                },
                spotdl::metadata::LyricsKind::Synchronized(s) => sonar::TrackLyrics {
                    kind: sonar::LyricsKind::Synced,
                    lines: s
                        .into_iter()
                        .map(|line| sonar::LyricsLine {
                            offset: line.start_time,
                            duration: if line.end_time >= line.start_time {
                                line.end_time - line.start_time
                            } else {
                                Duration::default()
                            },
                            text: line.text,
                        })
                        .collect(),
                },
            }),
            None => None,
        };

        let mut properties = properties_for_resource(resource_id.id);
        properties.insert(
            sonar::prop::DISC_NUMBER,
            sonar::PropertyValue::new(track.disc_number.to_string()).unwrap(),
        );
        properties.insert(
            sonar::prop::TRACK_NUMBER,
            sonar::PropertyValue::new(track.track_number.to_string()).unwrap(),
        );

        let external = ExternalTrack {
            name: track.name,
            artist: ExternalMediaId::new(track.artists[0].to_string()),
            album: ExternalMediaId::new(track.album.to_string()),
            lyrics,
            properties,
        };
        tracing::debug!("external track: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_playlist(&self, id: &ExternalMediaId) -> Result<ExternalPlaylist> {
        tracing::info!("fetching playlist: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Playlist {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid playlist id",
            ));
        }

        let playlist = self
            .fetcher
            .get_playlist(resource_id.id)
            .await
            .map_err(sonar::Error::wrap)?;
        tracing::debug!("playlist: {:#?}", playlist);

        let external = ExternalPlaylist {
            name: playlist.name,
            tracks: playlist
                .tracks
                .into_iter()
                .map(|id| ExternalMediaId::new(id.to_string()))
                .collect(),
            properties: properties_for_resource(resource_id.id),
        };
        tracing::debug!("external playlist: {:#?}", external);
        Ok(external)
    }

    #[tracing::instrument(skip(self))]
    async fn download_track(&self, id: &ExternalMediaId) -> Result<ByteStream> {
        tracing::info!("downloading track: {}", id);
        let resource_id = parse_resource_id(id)?;
        if resource_id.resource != Resource::Track {
            return Err(sonar::Error::new(
                sonar::ErrorKind::Invalid,
                "invalid track id",
            ));
        }

        tracing::debug!("preparing to download track: {:#?}", resource_id.id);
        let track_id = resource_id.id;
        let temp_dir = tempfile::tempdir().map_err(sonar::Error::wrap)?;
        let temp_file_path = temp_dir.path().join("samples");
        let mp3_file_path = temp_dir.path().join("track.mp3");
        let sink = spotdl::download::FileDownloadSink::from_path(&temp_file_path)
            .map_err(sonar::Error::wrap)?;

        tracing::debug!("acquiring download permit");
        let _permit = self
            .download_sem
            .acquire()
            .await
            .expect("failed to acquire semaphore permit");

        tracing::debug!("downloading track: {:#?}", track_id);
        spotdl::download::download(&self.session, sink, track_id)
            .await
            .map_err(sonar::Error::wrap)?;

        tracing::debug!("converting samples to mp3: {:#?}", mp3_file_path);
        convert::convert_samples_i16(
            convert::ConvertSamplesSource::Path(&temp_file_path),
            &mp3_file_path,
        )
        .await
        .map_err(sonar::Error::wrap)?;

        tracing::debug!("creating stream from file: {:#?}", mp3_file_path);
        Ok(sonar::bytestream::from_file(&mp3_file_path).await?)
    }
}

fn parse_resource_id(external_id: &ExternalMediaId) -> Result<ResourceId> {
    let resource_id = external_id
        .as_str()
        .parse::<ResourceId>()
        .map_err(|_| sonar::Error::new(sonar::ErrorKind::Invalid, "invalid spotify id"))?;
    Ok(resource_id)
}

fn properties_for_resource(id: SpotifyId) -> sonar::Properties {
    let mut properties = sonar::Properties::default();
    properties.insert(
        sonar::prop::EXTERNAL_SPOTIFY_ID,
        sonar::PropertyValue::new(id.to_string()).unwrap(),
    );
    properties
}

fn expand_multi_external_media_id(ids: MultiExternalMediaId) -> MultiExternalMediaId {
    let mut external_ids = Vec::with_capacity(ids.len());
    for id in ids {
        let external_id = match parse_resource_id(&id) {
            Ok(spotify_id) => ExternalMediaId::from(spotify_id.to_string()),
            Err(_) => id,
        };
        external_ids.push(external_id);
    }
    MultiExternalMediaId::from(external_ids)
}

fn fix_and_extract_resource_id(request: &mut ExternalMediaRequest) -> Option<ResourceId> {
    for id in &mut request.external_ids {
        if let Ok(resource_id) = parse_resource_id(id) {
            *id = ExternalMediaId::from(resource_id.to_uri());
            return Some(resource_id);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_expand_multi_external_media_id() {
        let ids = MultiExternalMediaId::from(vec![
            ExternalMediaId::from("spotify:track:3HOe5HB3E9tmz9ocHwsPgP"),
            ExternalMediaId::from("some-id-that-spotify-doesnt-recognize"),
            ExternalMediaId::from(
                "https://open.spotify.com/artist/762310PdDnwsDxAQxzQkfX?si=a88CtChQRO67ArejwWVYrg",
            ),
            ExternalMediaId::from("https://open.spotify.com/artist/762310PdDnwsDxAQxzQkfX"),
            ExternalMediaId::from("spotify:album:5AQ7uKRSpAv7SNUl4j24ru"),
        ]);

        let expanded = expand_multi_external_media_id(ids);
        let external_ids: Vec<ExternalMediaId> = From::from(expanded);
        assert_eq!(
            external_ids[0].as_str(),
            "spotify:track:3HOe5HB3E9tmz9ocHwsPgP"
        );
        assert_eq!(
            external_ids[1].as_str(),
            "some-id-that-spotify-doesnt-recognize"
        );
        assert_eq!(
            external_ids[2].as_str(),
            "spotify:artist:762310PdDnwsDxAQxzQkfX"
        );
        assert_eq!(
            external_ids[3].as_str(),
            "spotify:artist:762310PdDnwsDxAQxzQkfX"
        );
        assert_eq!(
            external_ids[4].as_str(),
            "spotify:album:5AQ7uKRSpAv7SNUl4j24ru"
        );
    }
}
