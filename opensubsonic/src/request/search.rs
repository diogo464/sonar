use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};
use serde::{Deserialize, Serialize};

#[allow(unused)]
use crate::{common::Milliseconds, request::browsing::GetMusicFolders};

/// Returns a listing of files matching the given search criteria. Supports paging through the result.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#search>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct Search {
    /// Artist to search for.
    pub artist: Option<String>,
    /// Album to search for.
    pub album: Option<String>,
    /// Song title to search for.
    pub title: Option<String>,
    /// Searches all fields
    pub any: Option<String>,
    /// Maximum number of results to return.
    pub count: Option<u32>,
    /// Search result offset. Used for paging.
    pub offset: Option<u32>,
    /// Only return matches that are newer than this.
    /// See [`Milliseconds`].
    pub newer_than: Option<Milliseconds>,
}

/// Returns albums, artists and songs matching the given search criteria. Supports paging through the result.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#search2>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct Search2 {
    /// Search query.
    pub query: String,
    /// Maximum number of artists to return.
    pub artist_count: Option<u32>,
    /// Search result offset for artists. Used for paging.
    pub artist_offset: Option<u32>,
    /// Maximum number of albums to return.
    pub album_count: Option<u32>,
    /// Search result offset for albums. Used for paging.
    pub album_offset: Option<u32>,
    /// Maximum number of songs to return.
    pub song_count: Option<u32>,
    /// Search result offset for songs. Used for paging.
    pub song_offset: Option<u32>,
    /// Since 1.12.0
    /// Only return results from the music folder with the given ID. See [`GetMusicFolders`].
    pub music_folder_id: Option<String>,
}

/// Similar to [`Search2`], but organizes music according to ID3 tags.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#search3>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct Search3 {
    /// Search query.
    pub query: String,
    /// Maximum number of artists to return.
    pub artist_count: Option<u32>,
    /// Search result offset for artists. Used for paging.
    pub artist_offset: Option<u32>,
    /// Maximum number of albums to return.
    pub album_count: Option<u32>,
    /// Search result offset for albums. Used for paging.
    pub album_offset: Option<u32>,
    /// Maximum number of songs to return.
    pub song_count: Option<u32>,
    /// Search result offset for songs. Used for paging.
    pub song_offset: Option<u32>,
    /// Since 1.12.0
    /// Only return results from the music folder with the given ID. See [`GetMusicFolders`].
    pub music_folder_id: Option<String>,
}

impl Search3 {
    pub const DEFAULT_ARTIST_COUNT: u32 = 20;
    pub const DEFAULT_ARTIST_OFFSET: u32 = 0;
    pub const DEFAULT_ALBUM_COUNT: u32 = 20;
    pub const DEFAULT_ALBUM_OFFSET: u32 = 0;
    pub const DEFAULT_SONG_COUNT: u32 = 20;
    pub const DEFAULT_SONG_OFFSET: u32 = 0;

    pub fn artist_count_or_default(&self) -> u32 {
        self.artist_count.unwrap_or(Self::DEFAULT_ARTIST_COUNT)
    }

    pub fn artist_offset_or_default(&self) -> u32 {
        self.artist_offset.unwrap_or(Self::DEFAULT_ARTIST_OFFSET)
    }

    pub fn album_count_or_default(&self) -> u32 {
        self.album_count.unwrap_or(Self::DEFAULT_ALBUM_COUNT)
    }

    pub fn album_offset_or_default(&self) -> u32 {
        self.album_offset.unwrap_or(Self::DEFAULT_ALBUM_OFFSET)
    }

    pub fn song_count_or_default(&self) -> u32 {
        self.song_count.unwrap_or(Self::DEFAULT_SONG_COUNT)
    }

    pub fn song_offset_or_default(&self) -> u32 {
        self.song_offset.unwrap_or(Self::DEFAULT_SONG_OFFSET)
    }
}
