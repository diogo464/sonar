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

const ID_TYPE_ARTIST_STR: &str = "artist";
const ID_TYPE_ALBUM_STR: &str = "album";
const ID_TYPE_TRACK_STR: &str = "track";
const ID_TYPE_PLAYLIST_STR: &str = "playlist";
const ID_TYPE_AUDIO_STR: &str = "audio";
const ID_TYPE_IMAGE_STR: &str = "image";
const ID_TYPE_USER_STR: &str = "user";
const ID_TYPE_LYRICS_STR: &str = "lyrics";
const ID_TYPE_SCROBBLE_STR: &str = "scrobble";

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
        write!(f, "{:x} is not a valid ID: {}", self.id, self.message)
    }
}

impl std::error::Error for InvalidIdError {}

pub(crate) trait SonarIdentifier: Sized + Clone + Copy + 'static {
    fn name(&self) -> &'static str;
    fn namespace(&self) -> u32;
    fn identifier(&self) -> u32;
}

macro_rules! impl_id {
    ($t:ident, $v:ident, $n:literal, $k:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $t(u32);

        impl SonarIdentifier for $t {
            fn name(&self) -> &'static str {
                $n
            }

            fn namespace(&self) -> u32 {
                $k
            }

            fn identifier(&self) -> u32 {
                self.0 & !ID_TYPE_MASK
            }
        }

        #[allow(dead_code)]
        impl $t {
            pub(crate) fn from_db(id: i64) -> Self {
                Self(id as u32 | $k << ID_TYPE_SHIFT)
            }

            pub(crate) fn to_db(self) -> i64 {
                (self.0 & !ID_TYPE_MASK) as i64
            }
        }

        impl From<$t> for SonarId {
            fn from(id: $t) -> Self {
                Self::$v(id)
            }
        }

        impl TryFrom<u32> for $t {
            type Error = InvalidIdError;

            fn try_from(id: u32) -> Result<Self, Self::Error> {
                if id & ID_TYPE_MASK == $k << ID_TYPE_SHIFT {
                    Ok(Self(id))
                } else {
                    Err(InvalidIdError::new(id, std::concat!("not an ", $n, " ID")))
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
                let id = s.parse::<SonarId>()?;
                match id {
                    SonarId::$v(id) => Ok(id),
                    _ => Err(InvalidIdError::new(0, std::concat!("not an ", $n, " ID"))),
                }
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                SonarId::from(*self).fmt(f)
            }
        }

        impl sqlx::Type<sqlx::Sqlite> for $t {
            fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
                <i64 as sqlx::Type<sqlx::Sqlite>>::type_info()
            }
        }

        impl<'a> sqlx::Encode<'a, sqlx::Sqlite> for $t {
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'a>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                let db_id = self.to_db();
                <i64 as sqlx::Encode<'a, sqlx::Sqlite>>::encode_by_ref(&db_id, buf)
            }
        }

        impl<'r, DB> sqlx::Decode<'r, DB> for $t
        where
            DB: sqlx::Database,
            i64: sqlx::Decode<'r, DB>,
        {
            fn decode(
                value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
            ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
                let id = <i64 as sqlx::Decode<'r, DB>>::decode(value)?;
                Ok(Self::from_db(id))
            }
        }
    };
}

impl_id!(ArtistId, Artist, "artist", ID_TYPE_ARTIST);
impl_id!(AlbumId, Album, "album", ID_TYPE_ALBUM);
impl_id!(TrackId, Track, "track", ID_TYPE_TRACK);
impl_id!(PlaylistId, Playlist, "playlist", ID_TYPE_PLAYLIST);
impl_id!(AudioId, Audio, "audio", ID_TYPE_AUDIO);
impl_id!(ImageId, Image, "image", ID_TYPE_IMAGE);
impl_id!(UserId, User, "user", ID_TYPE_USER);
impl_id!(LyricsId, Lyrics, "lyrics", ID_TYPE_LYRICS);
impl_id!(ScrobbleId, Scrobble, "scrobble", ID_TYPE_SCROBBLE);

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

impl std::fmt::Display for SonarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = u32::from(*self);
        f.write_str("sonar:")?;
        match (id & ID_TYPE_MASK) >> ID_TYPE_SHIFT {
            ID_TYPE_ARTIST => write!(f, "{}", ID_TYPE_ARTIST_STR)?,
            ID_TYPE_ALBUM => write!(f, "{}", ID_TYPE_ALBUM_STR)?,
            ID_TYPE_TRACK => write!(f, "{}", ID_TYPE_TRACK_STR)?,
            ID_TYPE_PLAYLIST => write!(f, "{}", ID_TYPE_PLAYLIST_STR)?,
            ID_TYPE_AUDIO => write!(f, "{}", ID_TYPE_AUDIO_STR)?,
            ID_TYPE_IMAGE => write!(f, "{}", ID_TYPE_IMAGE_STR)?,
            ID_TYPE_USER => write!(f, "{}", ID_TYPE_USER_STR)?,
            ID_TYPE_LYRICS => write!(f, "{}", ID_TYPE_LYRICS_STR)?,
            ID_TYPE_SCROBBLE => write!(f, "{}", ID_TYPE_SCROBBLE_STR)?,
            _ => unreachable!(),
        };
        write!(f, ":{:x}", id)
    }
}

impl TryFrom<u32> for SonarId {
    type Error = InvalidIdError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        match (id & ID_TYPE_MASK) >> ID_TYPE_SHIFT {
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
        let s = s
            .strip_prefix("sonar:")
            .ok_or_else(|| InvalidIdError::new(0, "not a sonar ID"))?;
        let (kind, value) = s
            .split_once(':')
            .ok_or_else(|| InvalidIdError::new(0, "not a sonar ID"))?;
        let id = u32::from_str_radix(value, 16).map_err(|_| {
            InvalidIdError::new(
                0,
                "failed to parse ID value: must be a 32-bit hexadecimal number",
            )
        })?;
        match kind {
            ID_TYPE_ARTIST_STR => Ok(Self::Artist(ArtistId::try_from(id)?)),
            ID_TYPE_ALBUM_STR => Ok(Self::Album(AlbumId::try_from(id)?)),
            ID_TYPE_TRACK_STR => Ok(Self::Track(TrackId::try_from(id)?)),
            ID_TYPE_PLAYLIST_STR => Ok(Self::Playlist(PlaylistId::try_from(id)?)),
            ID_TYPE_AUDIO_STR => Ok(Self::Audio(AudioId::try_from(id)?)),
            ID_TYPE_IMAGE_STR => Ok(Self::Image(ImageId::try_from(id)?)),
            ID_TYPE_USER_STR => Ok(Self::User(UserId::try_from(id)?)),
            ID_TYPE_LYRICS_STR => Ok(Self::Lyrics(LyricsId::try_from(id)?)),
            ID_TYPE_SCROBBLE_STR => Ok(Self::Scrobble(ScrobbleId::try_from(id)?)),
            _ => Err(InvalidIdError::new(id, "unknown ID type")),
        }
    }
}
