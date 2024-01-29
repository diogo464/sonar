use std::time::Duration;

use bytes::Bytes;
use rand::RngCore;

use crate::{
    extractor::{ExtractedMetadata, Extractor},
    Album, AlbumId, Artist, ArtistId, ByteStream, Context, Playlist, Track, User, UserId,
};

#[derive(Debug, Clone)]
pub struct StaticMetadataExtractor(ExtractedMetadata);

impl StaticMetadataExtractor {
    pub fn new(metadata: ExtractedMetadata) -> Self {
        Self(metadata)
    }
}

impl Extractor for StaticMetadataExtractor {
    fn extract(&self, _path: &std::path::Path) -> std::io::Result<ExtractedMetadata> {
        Ok(self.0.clone())
    }
}

pub fn create_config_memory() -> crate::Config {
    let config = crate::Config::new(":memory:", crate::StorageBackend::Memory);
    config
}

pub async fn create_context_memory() -> Context {
    let config = create_config_memory();
    crate::new(config).await.unwrap()
}

pub async fn create_context(config: crate::Config) -> Context {
    crate::new(config).await.unwrap()
}

pub async fn create_user(ctx: &Context, username: &str) -> User {
    create_user_with_password(ctx, username, "password").await
}

pub async fn create_user_with_password(ctx: &Context, username: &str, password: &str) -> User {
    crate::user_create(
        ctx,
        crate::UserCreate {
            username: username.parse().unwrap(),
            password: password.to_owned(),
            avatar: None,
        },
    )
    .await
    .unwrap()
}

pub async fn create_artist(ctx: &Context, name: &str) -> Artist {
    crate::artist_create(
        ctx,
        crate::ArtistCreate {
            name: name.to_string(),
            cover_art: None,
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub async fn create_album(ctx: &Context, artist: ArtistId, name: &str) -> Album {
    crate::album_create(
        ctx,
        crate::AlbumCreate {
            artist,
            name: name.to_string(),
            cover_art: None,
            properties: Default::default(),
            release_date: "2020-01-01T00:00:00Z".parse().unwrap(),
        },
    )
    .await
    .unwrap()
}

pub async fn create_track(ctx: &Context, album: AlbumId, name: &str) -> Track {
    let mut track_data = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut track_data);
    create_track_with_data(ctx, album, name, &track_data).await
}

pub async fn create_track_with_data(
    ctx: &Context,
    album: AlbumId,
    name: &str,
    data: &[u8],
) -> Track {
    let track_data = Bytes::from(data.to_vec());
    crate::track_create(
        ctx,
        crate::TrackCreate {
            name: name.to_string(),
            album,
            duration: Duration::from_secs(60),
            cover_art: None,
            lyrics: None,
            properties: Default::default(),
            audio_stream: crate::bytestream::from_bytes(track_data),
            audio_filename: format!("{}.mp3", name),
        },
    )
    .await
    .unwrap()
}

pub async fn create_playlist(ctx: &Context, owner: UserId, name: &str) -> Playlist {
    crate::playlist_create(
        ctx,
        crate::PlaylistCreate {
            name: name.to_string(),
            owner,
            tracks: Default::default(),
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub fn create_simple_genres() -> crate::Genres {
    let mut genres = crate::Genres::default();
    genres.set(&"heavy metal".parse().unwrap());
    genres.set(&"electronic".parse().unwrap());
    genres
}

pub fn create_simple_properties() -> crate::Properties {
    let mut properties = crate::Properties::default();
    properties.insert(
        crate::PropertyKey::new_uncheked("key1"),
        crate::PropertyValue::new_uncheked("value1"),
    );
    properties.insert(
        crate::PropertyKey::new_uncheked("key2"),
        crate::PropertyValue::new_uncheked("value2"),
    );
    properties
}

pub fn create_stream(data: &[u8]) -> ByteStream {
    crate::bytestream::from_bytes(Bytes::from(data.to_vec()))
}
