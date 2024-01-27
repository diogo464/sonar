use std::str::FromStr;

pub(crate) const ID_TYPE_MASK: u32 = 0xFF00_0000;
pub(crate) const ID_TYPE_SHIFT: u32 = 24;

pub(crate) const ID_TYPE_ARTIST: u32 = 1;
pub(crate) const ID_TYPE_ALBUM: u32 = 2;
pub(crate) const ID_TYPE_TRACK: u32 = 3;
pub(crate) const ID_TYPE_PLAYLIST: u32 = 4;
pub(crate) const ID_TYPE_AUDIO: u32 = 5;
pub(crate) const ID_TYPE_IMAGE: u32 = 6;
pub(crate) const ID_TYPE_USER: u32 = 7;
pub(crate) const ID_TYPE_LYRICS: u32 = 8;
pub(crate) const ID_TYPE_SCROBBLE: u32 = 9;

#[derive(Debug)]
pub struct InvalidIdError {
    id: u32,
    message: &'static str,
}

impl InvalidIdError {
    fn new(id: u32, message: &'static str) -> Self {
        Self { id, message }
    }
}

impl std::fmt::Display for InvalidIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x} is not a valid ID: {}", self.id, self.message)
    }
}

impl std::error::Error for InvalidIdError {}

macro_rules! impl_id {
    ($t:ident, $k:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $t(u32);

        #[allow(dead_code)]
        impl $t {
            pub(crate) fn from_db(id: i64) -> Self {
                Self(id as u32 | $k << ID_TYPE_SHIFT)
            }

            pub(crate) fn to_db(self) -> i64 {
                (self.0 & !ID_TYPE_MASK) as i64
            }
        }

        impl TryFrom<u32> for $t {
            type Error = InvalidIdError;

            fn try_from(id: u32) -> Result<Self, Self::Error> {
                if id & ID_TYPE_MASK == $k << ID_TYPE_SHIFT {
                    Ok(Self(id))
                } else {
                    Err(InvalidIdError::new(id, "not an artist ID"))
                }
            }
        }

        impl From<$t> for u32 {
            fn from(id: $t) -> Self {
                id.0
            }
        }

        impl FromStr for $t {
            type Err = InvalidIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let id = u32::from_str_radix(s, 16).map_err(|_| {
                    InvalidIdError::new(
                        0,
                        "failed to parse artist ID: must be a 32-bit hexadecimal number",
                    )
                })?;
                Self::try_from(id)
            }
        }
    };
}

impl_id!(ArtistId, ID_TYPE_ARTIST);
impl_id!(AlbumId, ID_TYPE_ALBUM);
impl_id!(TrackId, ID_TYPE_TRACK);
impl_id!(PlaylistId, ID_TYPE_PLAYLIST);
impl_id!(AudioId, ID_TYPE_AUDIO);
impl_id!(ImageId, ID_TYPE_IMAGE);
impl_id!(UserId, ID_TYPE_USER);
impl_id!(LyricsId, ID_TYPE_LYRICS);
impl_id!(ScrobbleId, ID_TYPE_SCROBBLE);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SonarId {
    Artist(ArtistId),
    Album(AlbumId),
    Track(TrackId),
    Playlist(PlaylistId),
    Audio(AudioId),
    Image(ImageId),
    User(UserId),
    Lyrics(LyricsId),
    Scrobble(ScrobbleId),
}

impl TryFrom<u32> for SonarId {
    type Error = InvalidIdError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        match id & ID_TYPE_MASK {
            ID_TYPE_ARTIST => Ok(Self::Artist(ArtistId::try_from(id)?)),
            ID_TYPE_ALBUM => Ok(Self::Album(AlbumId::try_from(id)?)),
            ID_TYPE_TRACK => Ok(Self::Track(TrackId::try_from(id)?)),
            ID_TYPE_PLAYLIST => Ok(Self::Playlist(PlaylistId::try_from(id)?)),
            ID_TYPE_AUDIO => Ok(Self::Audio(AudioId::try_from(id)?)),
            ID_TYPE_IMAGE => Ok(Self::Image(ImageId::try_from(id)?)),
            ID_TYPE_USER => Ok(Self::User(UserId::try_from(id)?)),
            ID_TYPE_LYRICS => Ok(Self::Lyrics(LyricsId::try_from(id)?)),
            ID_TYPE_SCROBBLE => Ok(Self::Scrobble(ScrobbleId::try_from(id)?)),
            _ => Err(InvalidIdError::new(id, "unknown ID type")),
        }
    }
}

impl From<SonarId> for u32 {
    fn from(id: SonarId) -> Self {
        match id {
            SonarId::Artist(id) => id.into(),
            SonarId::Album(id) => id.into(),
            SonarId::Track(id) => id.into(),
            SonarId::Playlist(id) => id.into(),
            SonarId::Audio(id) => id.into(),
            SonarId::Image(id) => id.into(),
            SonarId::User(id) => id.into(),
            SonarId::Lyrics(id) => id.into(),
            SonarId::Scrobble(id) => id.into(),
        }
    }
}

impl FromStr for SonarId {
    type Err = InvalidIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = u32::from_str_radix(s, 16).map_err(|_| {
            InvalidIdError::new(
                0,
                "failed to parse artist ID: must be a 32-bit hexadecimal number",
            )
        })?;
        match id & ID_TYPE_MASK {
            ID_TYPE_ARTIST => Ok(Self::Artist(ArtistId::try_from(id)?)),
            ID_TYPE_ALBUM => Ok(Self::Album(AlbumId::try_from(id)?)),
            ID_TYPE_TRACK => Ok(Self::Track(TrackId::try_from(id)?)),
            ID_TYPE_PLAYLIST => Ok(Self::Playlist(PlaylistId::try_from(id)?)),
            ID_TYPE_AUDIO => Ok(Self::Audio(AudioId::try_from(id)?)),
            ID_TYPE_IMAGE => Ok(Self::Image(ImageId::try_from(id)?)),
            ID_TYPE_USER => Ok(Self::User(UserId::try_from(id)?)),
            ID_TYPE_LYRICS => Ok(Self::Lyrics(LyricsId::try_from(id)?)),
            ID_TYPE_SCROBBLE => Ok(Self::Scrobble(ScrobbleId::try_from(id)?)),
            _ => Err(InvalidIdError::new(id, "unknown ID type")),
        }
    }
}

impl From<ArtistId> for SonarId {
    fn from(id: ArtistId) -> Self {
        Self::Artist(id)
    }
}

impl From<AlbumId> for SonarId {
    fn from(id: AlbumId) -> Self {
        Self::Album(id)
    }
}

impl From<TrackId> for SonarId {
    fn from(id: TrackId) -> Self {
        Self::Track(id)
    }
}

impl From<PlaylistId> for SonarId {
    fn from(id: PlaylistId) -> Self {
        Self::Playlist(id)
    }
}

impl From<AudioId> for SonarId {
    fn from(id: AudioId) -> Self {
        Self::Audio(id)
    }
}

impl From<ImageId> for SonarId {
    fn from(id: ImageId) -> Self {
        Self::Image(id)
    }
}

impl From<UserId> for SonarId {
    fn from(id: UserId) -> Self {
        Self::User(id)
    }
}

impl From<LyricsId> for SonarId {
    fn from(id: LyricsId) -> Self {
        Self::Lyrics(id)
    }
}

impl From<ScrobbleId> for SonarId {
    fn from(id: ScrobbleId) -> Self {
        Self::Scrobble(id)
    }
}
