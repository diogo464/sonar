#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(concat_idents)]
#![feature(backtrace_frames)]
#![feature(let_chains)]
#![feature(is_sorted)]

mod error;
pub use error::*;

mod id;
pub use id::*;

mod types;
pub use types::*;

mod context;
pub use context::*;

pub mod bytestream;
pub mod ext;
pub mod prop;

#[doc(hidden)]
#[cfg(feature = "test-utilities")]
pub mod test;

pub(crate) mod album;
pub(crate) mod artist;
pub(crate) mod audio;
pub(crate) mod blob;
pub(crate) mod db;
pub(crate) mod download;
pub(crate) mod external;
pub(crate) mod extractor;
pub(crate) mod favorite;
pub(crate) mod gc;
pub(crate) mod genre;
pub(crate) mod image;
pub(crate) mod importer;
pub(crate) mod ks;
pub(crate) mod metadata;
pub(crate) mod migrations;
pub(crate) mod pin;
pub(crate) mod playlist;
pub(crate) mod property;
pub(crate) mod scrobble;
pub(crate) mod scrobbler;
pub(crate) mod search;
pub(crate) mod subscription;
pub(crate) mod track;
pub(crate) mod user;

pub use album::{Album, AlbumCreate, AlbumUpdate};
pub use artist::{Artist, ArtistCreate, ArtistUpdate};
pub use audio::{Audio, AudioCreate, AudioDownload, AudioStat};
pub use download::{Download, DownloadCreate, DownloadDelete, DownloadStatus};
pub use external::{
    ExternalAlbum, ExternalArtist, ExternalImage, ExternalMediaId, ExternalMediaType,
    ExternalPlaylist, ExternalService, ExternalTrack,
};
pub use extractor::{ExtractedImage, ExtractedMetadata, Extractor};
pub use favorite::Favorite;
pub use genre::{Genre, GenreUpdate, GenreUpdateAction, Genres, InvalidGenreError};
pub use image::{ImageCreate, ImageDownload};
pub use importer::Import;
pub use metadata::{
    AlbumMetadata, AlbumMetadataRequest, AlbumTracksMetadata, AlbumTracksMetadataRequest,
    ArtistMetadata, ArtistMetadataRequest, MetadataFetchMask, MetadataFetchParams,
    MetadataProvider, MetadataRequestKind, TrackMetadata, TrackMetadataRequest,
    METADATA_FETCH_MASK_ALL, METADATA_FETCH_MASK_COVER, METADATA_FETCH_MASK_EMPTY,
    METADATA_FETCH_MASK_GENRES, METADATA_FETCH_MASK_NAME, METADATA_FETCH_MASK_PROPERTIES,
};
pub use playlist::{Playlist, PlaylistCreate, PlaylistTrack, PlaylistUpdate};
pub use property::{
    InvalidPropertyKeyError, InvalidPropertyValueError, Properties, PropertyKey, PropertyUpdate,
    PropertyUpdateAction, PropertyValue,
};
pub use scrobble::{Scrobble, ScrobbleCreate, ScrobbleUpdate};
pub use scrobbler::Scrobbler;
pub use search::{SearchFlags, SearchQuery, SearchResult};
pub use subscription::{Subscription, SubscriptionCreate, SubscriptionDelete};
pub use track::{
    Lyrics, LyricsKind, LyricsLine, Track, TrackCreate, TrackListRandom, TrackLyrics, TrackUpdate,
};
pub use user::{
    InvalidUserTokenError, InvalidUsernameError, User, UserCreate, UserToken, UserUpdate, Username,
};

pub use async_trait::async_trait;
pub use bytes;
pub use chrono;

pub mod metadata_prelude {
    pub use crate::{
        Album, AlbumId, AlbumMetadata, AlbumMetadataRequest, AlbumTracksMetadata,
        AlbumTracksMetadataRequest, Artist, ArtistId, ArtistMetadata, ArtistMetadataRequest, Error,
        ErrorKind, MetadataProvider, MetadataRequestKind, Properties, Result, Track, TrackId,
        TrackMetadata, TrackMetadataRequest,
    };
}
