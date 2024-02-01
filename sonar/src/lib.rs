#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(concat_idents)]
#![feature(backtrace_frames)]

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
pub mod extractor;
pub mod metadata;
pub mod prop;
pub mod scrobbler;

#[doc(hidden)]
#[cfg(feature = "test-utilities")]
pub mod test;

pub(crate) mod album;
pub(crate) mod artist;
pub(crate) mod audio;
pub(crate) mod blob;
pub(crate) mod db;
pub(crate) mod genre;
pub(crate) mod image;
pub(crate) mod importer;
pub(crate) mod ks;
pub(crate) mod playlist;
pub(crate) mod property;
pub(crate) mod scrobble;
pub(crate) mod track;
pub(crate) mod user;

pub use album::{Album, AlbumCreate, AlbumUpdate};
pub use artist::{Artist, ArtistCreate, ArtistUpdate};
pub use audio::{Audio, AudioCreate, AudioDownload};
pub use genre::{Genre, GenreUpdateAction, Genres, InvalidGenreError};
pub use image::{ImageCreate, ImageDownload};
pub use importer::Import;
pub use playlist::{Playlist, PlaylistCreate, PlaylistTrack, PlaylistUpdate};
pub use property::{
    InvalidPropertyKeyError, InvalidPropertyValueError, Properties, PropertyKey, PropertyUpdate,
    PropertyUpdateAction, PropertyValue,
};
pub use scrobble::{Scrobble, ScrobbleCreate, ScrobbleUpdate};
pub use track::{Lyrics, LyricsKind, Track, TrackCreate, TrackLyrics, TrackUpdate};
pub use user::{
    InvalidUserTokenError, InvalidUsernameError, User, UserCreate, UserToken, UserUpdate, Username,
};

pub use async_trait::async_trait;
pub use bytes;
pub use chrono;
