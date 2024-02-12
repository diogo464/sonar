use std::str::FromStr;

pub(crate) const ID_NAMESPACE_MASK: u32 = 0xFF00_0000;
pub(crate) const ID_NAMESPACE_SHIFT: u32 = 24;

pub(crate) const ID_NAMESPACE_ARTIST: u32 = 1;
pub(crate) const ID_NAMESPACE_ALBUM: u32 = 2;
pub(crate) const ID_NAMESPACE_TRACK: u32 = 3;
pub(crate) const ID_NAMESPACE_PLAYLIST: u32 = 4;
pub(crate) const ID_NAMESPACE_AUDIO: u32 = 5;
pub(crate) const ID_NAMESPACE_IMAGE: u32 = 6;
pub(crate) const ID_NAMESPACE_USER: u32 = 7;
pub(crate) const ID_NAMESPACE_LYRICS: u32 = 8;
pub(crate) const ID_NAMESPACE_SCROBBLE: u32 = 9;

const ID_NAMESPACE_ARTIST_STR: &str = "artist";
const ID_NAMESPACE_ALBUM_STR: &str = "album";
const ID_NAMESPACE_TRACK_STR: &str = "track";
const ID_NAMESPACE_PLAYLIST_STR: &str = "playlist";
const ID_NAMESPACE_AUDIO_STR: &str = "audio";
const ID_NAMESPACE_IMAGE_STR: &str = "image";
const ID_NAMESPACE_USER_STR: &str = "user";
const ID_NAMESPACE_LYRICS_STR: &str = "lyrics";
const ID_NAMESPACE_SCROBBLE_STR: &str = "scrobble";

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

pub(crate) trait SonarIdentifier: std::fmt::Debug + Sized + Clone + Copy {
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
                self.0 & !ID_NAMESPACE_MASK
            }
        }

        impl<'a> SonarIdentifier for &'a $t {
            fn name(&self) -> &'static str {
                $n
            }

            fn namespace(&self) -> u32 {
                $k
            }

            fn identifier(&self) -> u32 {
                self.0 & !ID_NAMESPACE_MASK
            }
        }

        #[allow(dead_code)]
        impl $t {
            pub(crate) fn from_db(id: i64) -> Self {
                Self(id as u32 | $k << ID_NAMESPACE_SHIFT)
            }

            pub(crate) fn to_db(self) -> i64 {
                (self.0 & !ID_NAMESPACE_MASK) as i64
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
                if id & ID_NAMESPACE_MASK == $k << ID_NAMESPACE_SHIFT {
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

impl_id!(ArtistId, Artist, "artist", ID_NAMESPACE_ARTIST);
impl_id!(AlbumId, Album, "album", ID_NAMESPACE_ALBUM);
impl_id!(TrackId, Track, "track", ID_NAMESPACE_TRACK);
impl_id!(PlaylistId, Playlist, "playlist", ID_NAMESPACE_PLAYLIST);
impl_id!(AudioId, Audio, "audio", ID_NAMESPACE_AUDIO);
impl_id!(ImageId, Image, "image", ID_NAMESPACE_IMAGE);
impl_id!(UserId, User, "user", ID_NAMESPACE_USER);
impl_id!(LyricsId, Lyrics, "lyrics", ID_NAMESPACE_LYRICS);
impl_id!(ScrobbleId, Scrobble, "scrobble", ID_NAMESPACE_SCROBBLE);

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
        match (id & ID_NAMESPACE_MASK) >> ID_NAMESPACE_SHIFT {
            ID_NAMESPACE_ARTIST => write!(f, "{}", ID_NAMESPACE_ARTIST_STR)?,
            ID_NAMESPACE_ALBUM => write!(f, "{}", ID_NAMESPACE_ALBUM_STR)?,
            ID_NAMESPACE_TRACK => write!(f, "{}", ID_NAMESPACE_TRACK_STR)?,
            ID_NAMESPACE_PLAYLIST => write!(f, "{}", ID_NAMESPACE_PLAYLIST_STR)?,
            ID_NAMESPACE_AUDIO => write!(f, "{}", ID_NAMESPACE_AUDIO_STR)?,
            ID_NAMESPACE_IMAGE => write!(f, "{}", ID_NAMESPACE_IMAGE_STR)?,
            ID_NAMESPACE_USER => write!(f, "{}", ID_NAMESPACE_USER_STR)?,
            ID_NAMESPACE_LYRICS => write!(f, "{}", ID_NAMESPACE_LYRICS_STR)?,
            ID_NAMESPACE_SCROBBLE => write!(f, "{}", ID_NAMESPACE_SCROBBLE_STR)?,
            _ => unreachable!(),
        };
        write!(f, ":{:x}", id)
    }
}

impl TryFrom<u32> for SonarId {
    type Error = InvalidIdError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        match (id & ID_NAMESPACE_MASK) >> ID_NAMESPACE_SHIFT {
            ID_NAMESPACE_ARTIST => Ok(Self::Artist(ArtistId::try_from(id)?)),
            ID_NAMESPACE_ALBUM => Ok(Self::Album(AlbumId::try_from(id)?)),
            ID_NAMESPACE_TRACK => Ok(Self::Track(TrackId::try_from(id)?)),
            ID_NAMESPACE_PLAYLIST => Ok(Self::Playlist(PlaylistId::try_from(id)?)),
            ID_NAMESPACE_AUDIO => Ok(Self::Audio(AudioId::try_from(id)?)),
            ID_NAMESPACE_IMAGE => Ok(Self::Image(ImageId::try_from(id)?)),
            ID_NAMESPACE_USER => Ok(Self::User(UserId::try_from(id)?)),
            ID_NAMESPACE_LYRICS => Ok(Self::Lyrics(LyricsId::try_from(id)?)),
            ID_NAMESPACE_SCROBBLE => Ok(Self::Scrobble(ScrobbleId::try_from(id)?)),
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

impl From<SonarId> for String {
    fn from(id: SonarId) -> Self {
        id.to_string()
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
            ID_NAMESPACE_ARTIST_STR => Ok(Self::Artist(ArtistId::try_from(id)?)),
            ID_NAMESPACE_ALBUM_STR => Ok(Self::Album(AlbumId::try_from(id)?)),
            ID_NAMESPACE_TRACK_STR => Ok(Self::Track(TrackId::try_from(id)?)),
            ID_NAMESPACE_PLAYLIST_STR => Ok(Self::Playlist(PlaylistId::try_from(id)?)),
            ID_NAMESPACE_AUDIO_STR => Ok(Self::Audio(AudioId::try_from(id)?)),
            ID_NAMESPACE_IMAGE_STR => Ok(Self::Image(ImageId::try_from(id)?)),
            ID_NAMESPACE_USER_STR => Ok(Self::User(UserId::try_from(id)?)),
            ID_NAMESPACE_LYRICS_STR => Ok(Self::Lyrics(LyricsId::try_from(id)?)),
            ID_NAMESPACE_SCROBBLE_STR => Ok(Self::Scrobble(ScrobbleId::try_from(id)?)),
            _ => Err(InvalidIdError::new(id, "unknown ID type")),
        }
    }
}

impl SonarIdentifier for SonarId {
    fn name(&self) -> &'static str {
        match self {
            SonarId::Artist(id) => id.name(),
            SonarId::Album(id) => id.name(),
            SonarId::Track(id) => id.name(),
            SonarId::Playlist(id) => id.name(),
            SonarId::Audio(id) => id.name(),
            SonarId::Image(id) => id.name(),
            SonarId::User(id) => id.name(),
            SonarId::Lyrics(id) => id.name(),
            SonarId::Scrobble(id) => id.name(),
        }
    }

    fn namespace(&self) -> u32 {
        match self {
            SonarId::Artist(id) => id.namespace(),
            SonarId::Album(id) => id.namespace(),
            SonarId::Track(id) => id.namespace(),
            SonarId::Playlist(id) => id.namespace(),
            SonarId::Audio(id) => id.namespace(),
            SonarId::Image(id) => id.namespace(),
            SonarId::User(id) => id.namespace(),
            SonarId::Lyrics(id) => id.namespace(),
            SonarId::Scrobble(id) => id.namespace(),
        }
    }

    fn identifier(&self) -> u32 {
        match self {
            SonarId::Artist(id) => id.identifier(),
            SonarId::Album(id) => id.identifier(),
            SonarId::Track(id) => id.identifier(),
            SonarId::Playlist(id) => id.identifier(),
            SonarId::Audio(id) => id.identifier(),
            SonarId::Image(id) => id.identifier(),
            SonarId::User(id) => id.identifier(),
            SonarId::Lyrics(id) => id.identifier(),
            SonarId::Scrobble(id) => id.identifier(),
        }
    }
}

impl SonarId {
    pub(crate) fn from_namespace_and_id(ty: u32, id: u32) -> Result<SonarId, InvalidIdError> {
        let id = id | ty << ID_NAMESPACE_SHIFT;
        TryFrom::try_from(id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_artist_id() {
        let id = ArtistId::try_from(0x01000001).unwrap();
        assert_eq!(id, ArtistId(0x01000001));
        assert_eq!(id.name(), "artist");
        assert_eq!(id.namespace(), ID_NAMESPACE_ARTIST);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(ArtistId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:artist:1000001");
    }

    #[test]
    fn test_album_id() {
        let id = AlbumId::try_from(0x02000001).unwrap();
        assert_eq!(id, AlbumId(0x02000001));
        assert_eq!(id.name(), "album");
        assert_eq!(id.namespace(), ID_NAMESPACE_ALBUM);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(AlbumId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:album:2000001");
    }

    #[test]
    fn test_track_id() {
        let id = TrackId::try_from(0x03000001).unwrap();
        assert_eq!(id, TrackId(0x03000001));
        assert_eq!(id.name(), "track");
        assert_eq!(id.namespace(), ID_NAMESPACE_TRACK);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(TrackId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:track:3000001");
    }

    #[test]
    fn test_playlist_id() {
        let id = PlaylistId::try_from(0x04000001).unwrap();
        assert_eq!(id, PlaylistId(0x04000001));
        assert_eq!(id.name(), "playlist");
        assert_eq!(id.namespace(), ID_NAMESPACE_PLAYLIST);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(PlaylistId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:playlist:4000001");
    }

    #[test]
    fn test_audio_id() {
        let id = AudioId::try_from(0x05000001).unwrap();
        assert_eq!(id, AudioId(0x05000001));
        assert_eq!(id.name(), "audio");
        assert_eq!(id.namespace(), ID_NAMESPACE_AUDIO);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(AudioId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:audio:5000001");
    }

    #[test]
    fn test_image_id() {
        let id = ImageId::try_from(0x06000001).unwrap();
        assert_eq!(id, ImageId(0x06000001));
        assert_eq!(id.name(), "image");
        assert_eq!(id.namespace(), ID_NAMESPACE_IMAGE);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(ImageId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:image:6000001");
    }

    #[test]
    fn test_user_id() {
        let id = UserId::try_from(0x07000001).unwrap();
        assert_eq!(id, UserId(0x07000001));
        assert_eq!(id.name(), "user");
        assert_eq!(id.namespace(), ID_NAMESPACE_USER);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(UserId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:user:7000001");
    }

    #[test]
    fn test_lyrics_id() {
        let id = LyricsId::try_from(0x08000001).unwrap();
        assert_eq!(id, LyricsId(0x08000001));
        assert_eq!(id.name(), "lyrics");
        assert_eq!(id.namespace(), ID_NAMESPACE_LYRICS);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(LyricsId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:lyrics:8000001");
    }

    #[test]
    fn test_scrobble_id() {
        let id = ScrobbleId::try_from(0x09000001).unwrap();
        assert_eq!(id, ScrobbleId(0x09000001));
        assert_eq!(id.name(), "scrobble");
        assert_eq!(id.namespace(), ID_NAMESPACE_SCROBBLE);
        assert_eq!(id.identifier(), 1);
        assert_eq!(id.to_db(), 1);
        assert_eq!(ScrobbleId::from_db(1), id);
        assert_eq!(id.to_string(), "sonar:scrobble:9000001");
    }
}
