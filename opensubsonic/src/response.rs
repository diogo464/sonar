//! Module for Subsonic API responses.
//!
//! # Example
//! Building a response:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use opensubsonic::{common::Version, response::{Response, ResponseBody, License}};
//!     let response = Response::ok(
//!         Version::V1_16_1,
//!         ResponseBody::License(License {
//!             valid: true,
//!             ..Default::default()
//!         }),
//!         "my-server-name",
//!         "my-server-version"
//!     );
//!    # let _ = response;
//! # Ok(())
//! # }
//! ```
//!
//! Parsing a response:
//! Deserialize a response from json
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use opensubsonic::{common::Version, response::{Response, ResponseBody, License}};
//!     let response = Response::ok(
//!         Version::V1_16_1,
//!         ResponseBody::License(License {
//!             valid: true,
//!             ..Default::default()
//!         }),
//!         "my-server-name",
//!         "my-server-version"
//!     );
//!     # let _ = response;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        AverageRating, DateTime, Format, MediaType, Milliseconds, RecordLabel, Seconds, UserRating,
        Version,
    },
    xml::{self, XmlSerialize},
};

pub trait SubsonicSerialize {
    fn serialize(&self, format: Format) -> Vec<u8>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseObject {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: Response,
}

impl From<Response> for ResponseObject {
    fn from(response: Response) -> Self {
        Self {
            subsonic_response: response,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Failed,
}

impl std::fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => f.write_str("ok"),
            Self::Failed => f.write_str("failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub status: ResponseStatus,
    pub version: Version,
    #[serde(flatten)]
    pub body: Option<ResponseBody>,
    #[serde(rename = "type")]
    pub server_type: String,
    pub server_version: String,
    pub open_subsonic: bool,
}

impl Response {
    pub fn ok(
        version: Version,
        body: ResponseBody,
        server_type: impl Into<String>,
        server_version: impl Into<String>,
    ) -> Self {
        Self {
            status: ResponseStatus::Ok,
            version,
            body: Some(body),
            server_type: server_type.into(),
            server_version: server_version.into(),
            open_subsonic: true,
        }
    }

    pub fn ok_empty(
        version: Version,
        server_type: impl Into<String>,
        server_version: impl Into<String>,
    ) -> Self {
        Self {
            status: ResponseStatus::Ok,
            version,
            body: None,
            server_type: server_type.into(),
            server_version: server_version.into(),
            open_subsonic: true,
        }
    }

    pub fn failed(
        version: Version,
        error: Error,
        server_type: impl Into<String>,
        server_version: impl Into<String>,
    ) -> Self {
        Self {
            status: ResponseStatus::Failed,
            version,
            body: Some(ResponseBody::Error(error)),
            server_type: server_type.into(),
            server_version: server_version.into(),
            open_subsonic: true,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseBody {
    MusicFolders(MusicFolders),
    Indexes(ArtistsID3),
    Directory(Directory),
    Genres(Genres),
    Artists(ArtistsID3),
    Artist(ArtistWithAlbumsID3),
    Album(AlbumWithSongsID3),
    Song(Child),
    Videos(Videos),
    VideoInfo(VideoInfo),
    NowPlaying(NowPlaying),
    SearchResult(SearchResult),
    SearchResult2(SearchResult2),
    SearchResult3(SearchResult3),
    Playlists(Playlists),
    Playlist(PlaylistWithSongs),
    JukeboxStatus(JukeboxStatus),
    JukeboxPlaylist(JukeboxPlaylist),
    JukeboxControlResponse(JukeboxControlResponse),
    License(License),
    Users(Users),
    User(User),
    ChatMessages(ChatMessages),
    AlbumList(AlbumList),
    AlbumList2(AlbumList2),
    RandomSongs(Songs),
    SongsByGenre(Songs),
    Lyrics(Lyrics),
    Podcasts(Podcasts),
    NewestPodcasts(NewestPodcasts),
    InternetRadioStations(InternetRadioStations),
    Bookmarks(Bookmarks),
    PlayQueue(PlayQueue),
    Shares(Shares),
    Starred(Starred),
    Starred2(Starred2),
    AlbumInfo(AlbumInfo),
    ArtistInfo(ArtistInfo),
    ArtistInfo2(ArtistInfo2),
    SimilarSongs(SimilarSongs),
    SimilarSongs2(SimilarSongs2),
    TopSongs(TopSongs),
    ScanStatus(ScanStatus),
    Error(Error),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub valid: bool,
    pub email: Option<String>,
    pub license_expires: Option<DateTime>,
    pub trial_expires: Option<DateTime>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolders {
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolder {
    pub id: u32,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    /// Note: Not sure that this is actually milliseconds
    pub last_modified: Milliseconds,
    pub ignored_articles: String,
    pub shortcut: Vec<Artist>,
    pub index: Vec<Index>,
    pub child: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub name: String,
    pub artist: Vec<Artist>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub artist_image_url: Option<String>,
    pub starred: Option<DateTime>,
    pub user_rating: Option<UserRating>,
    pub average_rating: Option<AverageRating>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genres {
    pub genre: Vec<Genre>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub song_count: u32,
    pub album_count: u32,
    pub name: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistsID3 {
    pub index: Vec<IndexID3>,
    pub ignored_articles: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexID3 {
    pub name: String,
    pub artist: Vec<ArtistID3>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistID3 {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    pub album_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<DateTime>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistWithAlbumsID3 {
    #[serde(flatten)]
    pub artist: ArtistID3,
    pub album: Vec<AlbumID3>,
}

/// An album from ID3 tags.
/// <https://opensubsonic.netlify.app/docs/responses/albumid3/>
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumID3 {
    /// The id of the album
    pub id: String,
    /// The album name.
    pub name: String,
    /// Artist name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    /// The id of the artist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    /// A covertArt id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    /// Number of songs
    pub song_count: u32,
    /// Total duration of the album
    pub duration: Milliseconds,
    /// Number of play of the album
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<u64>,
    /// Date the album was added. [ISO 8601]
    pub created: DateTime,
    /// Date the album was starred. [ISO 8601]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<DateTime>,
    /// The album year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// The album genre
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<UserRating>,
    pub record_labels: Vec<RecordLabel>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumWithSongsID3 {
    #[serde(flatten)]
    pub album: AlbumID3,
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Videos {
    pub video: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub id: String,
    pub captions: Vec<Captions>,
    pub audio_track: Vec<AudioTrack>,
    pub conversion: Vec<VideoConversion>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Captions {
    pub id: String,
    pub format: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: String,
    pub name: Option<String>,
    pub language_code: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoConversion {
    pub id: String,
    pub bit_rate: Option<u32>,
    pub audio_track_id: Option<u32>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<UserRating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<AverageRating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<u64>,
    pub child: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub is_dir: bool,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcoded_content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcoded_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Seconds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_video: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<UserRating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<AverageRating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark_position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_height: Option<u32>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub entry: Vec<NowPlayingEntry>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingEntry {
    #[serde(flatten)]
    pub child: Child,
    pub username: String,
    pub minutes_ago: u32,
    pub player_id: u32,
    pub player_name: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub offset: u32,
    pub total_hits: u32,
    #[serde(rename = "match")]
    pub matches: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult2 {
    pub artist: Vec<Artist>,
    pub album: Vec<Child>,
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult3 {
    pub artist: Vec<ArtistID3>,
    pub album: Vec<AlbumID3>,
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlists {
    pub playlist: Vec<Playlist>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    pub song_count: u32,
    pub duration: Seconds,
    pub created: DateTime,
    pub changed: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_user: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistWithSongs {
    #[serde(flatten)]
    pub playlist: Playlist,
    pub entry: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JukeboxStatus {
    pub current_index: u32,
    pub playing: bool,
    pub gain: f32,
    pub position: Option<u32>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JukeboxPlaylist {
    #[serde(flatten)]
    pub status: JukeboxStatus,
    pub entry: Vec<Child>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JukeboxControlResponse {
    JukeboxStatus(JukeboxStatus),
    JukeboxPlaylist(JukeboxPlaylist),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessages {
    pub chat_message: Vec<ChatMessage>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub username: String,
    pub time: DateTime,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumList {
    pub album: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumList2 {
    pub album: Vec<AlbumID3>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Songs {
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    pub artist: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Podcasts {
    pub channel: Vec<PodcastChannel>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastChannel {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_art: Option<String>,
    pub original_image_url: Option<String>,
    pub status: PodcastStatus,
    pub error_message: Option<String>,
    pub episode: Vec<PodcastEpisode>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewestPodcasts {
    pub episode: Vec<PodcastEpisode>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisode {
    #[serde(flatten)]
    pub child: Child,
    /// Use this ID for streaming the podcast
    pub stream_id: Option<String>,
    pub channel_id: String,
    pub description: Option<String>,
    pub status: PodcastStatus,
    pub publish_date: Option<DateTime>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PodcastStatus {
    New,
    Downloading,
    Completed,
    Skipped,
    #[default]
    Error,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternetRadioStations {
    pub internet_radio_station: Vec<InternetRadioStation>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternetRadioStation {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    pub home_page_url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmarks {
    pub bookmark: Vec<Bookmark>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub position: u64,
    pub username: String,
    pub comment: Option<String>,
    pub created: DateTime,
    pub changed: DateTime,
    pub entry: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayQueue {
    /// ID of the currently playing song
    pub current: Option<u64>, // TODO: u64?
    /// Position of the currently playing track
    pub position: Option<Milliseconds>,
    pub username: String,
    pub changed: DateTime,
    /// Name of client app
    pub changed_by: String,
    pub entry: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shares {
    pub share: Vec<Share>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub id: String,
    pub url: String,
    pub description: Option<String>,
    pub username: String,
    pub created: DateTime,
    pub expires: Option<DateTime>,
    pub last_visited: Option<DateTime>,
    pub visit_count: u64,
    pub entry: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Starred {
    pub song: Vec<Child>,
    pub album: Vec<Child>,
    pub artist: Vec<Artist>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfo {
    pub notes: String,
    pub music_brainz_id: String,
    pub last_fm_url: String,
    pub small_image_url: String,
    pub medium_image_url: String,
    pub large_image_url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfoBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image_url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo {
    #[serde(flatten)]
    pub info: ArtistInfoBase,
    pub similar_artist: Vec<Artist>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo2 {
    #[serde(flatten)]
    pub info: ArtistInfoBase,
    pub similar_artist: Vec<ArtistID3>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongs {
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongs2 {
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongs {
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Starred2 {
    pub song: Vec<Child>,
    pub album: Vec<AlbumID3>,
    pub artist: Vec<ArtistID3>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub scanning: bool,
    pub count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Users {
    pub user: Vec<User>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub email: Option<String>,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: Option<u64>,
    pub admin_role: bool,
    pub settings_role: bool,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub stream_role: bool,
    pub jukebox_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub avatar_last_changed: Option<DateTime>,
    pub folder: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: ErrorCode,
    pub message: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u32::from(self.code))?;
        if let Some(message) = &self.message {
            write!(f, ": {}", message)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(code: ErrorCode) -> Self {
        Error {
            code,
            message: None,
        }
    }

    pub fn with_message(code: ErrorCode, message: impl Into<String>) -> Self {
        Error {
            code,
            message: Some(message.into()),
        }
    }

    pub fn custom(err: impl std::error::Error) -> Self {
        Error {
            code: ErrorCode::Generic,
            message: Some(err.to_string()),
        }
    }

    pub fn custom_with_code(code: ErrorCode, err: impl std::error::Error) -> Self {
        Error {
            code,
            message: Some(err.to_string()),
        }
    }
}

macro_rules! error_impl_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Error {
                fn from(err: $t) -> Self {
                    Error::custom(err)
                }
            }
        )*
    };
}
error_impl_from!(
    crate::common::InvalidFormat,
    crate::common::InvalidVersion,
    crate::request::lists::InvalidListType,
    crate::common::InvalidVideoSize,
    crate::common::InvalidUserRating,
    crate::common::InvalidAudioBitrate,
    crate::common::InvalidVideoBitrate,
    crate::common::InvalidAverageRating,
    crate::request::jukebox::InvalidJukeboxAction,
    crate::query::QueryParseError
);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ErrorCode {
    #[default]
    Generic = 0,
    RequiredParameterMissing = 10,
    IncompatibleClient = 20,
    IncompatibleServer = 30,
    WrongUsernameOrPassword = 40,
    TokenAuthenticationNotSupported = 41,
    UserNotAuthorizedForTheGivenOperation = 50,
    TrialExpired = 60,
    DataNotFound = 70,
    Other(u32),
}

impl From<u32> for ErrorCode {
    fn from(code: u32) -> Self {
        match code {
            0 => ErrorCode::Generic,
            10 => ErrorCode::RequiredParameterMissing,
            20 => ErrorCode::IncompatibleClient,
            30 => ErrorCode::IncompatibleServer,
            40 => ErrorCode::WrongUsernameOrPassword,
            41 => ErrorCode::TokenAuthenticationNotSupported,
            50 => ErrorCode::UserNotAuthorizedForTheGivenOperation,
            60 => ErrorCode::TrialExpired,
            70 => ErrorCode::DataNotFound,
            _ => ErrorCode::Other(code),
        }
    }
}

impl From<ErrorCode> for u32 {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::Generic => 0,
            ErrorCode::RequiredParameterMissing => 10,
            ErrorCode::IncompatibleClient => 20,
            ErrorCode::IncompatibleServer => 30,
            ErrorCode::WrongUsernameOrPassword => 40,
            ErrorCode::TokenAuthenticationNotSupported => 41,
            ErrorCode::UserNotAuthorizedForTheGivenOperation => 50,
            ErrorCode::TrialExpired => 60,
            ErrorCode::DataNotFound => 70,
            ErrorCode::Other(code) => code,
        }
    }
}

impl serde::Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(u32::from(*self))
    }
}

impl<'de> serde::Deserialize<'de> for ErrorCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = u32::deserialize(deserializer)?;
        Ok(ErrorCode::from(code))
    }
}

impl XmlSerialize for Response {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "subsonic-response");
        xml::attr(xml, "status", &self.status);
        xml::attr(xml, "version", &self.version);
        xml::attr(xml, "type", &self.server_type);
        xml::attr(xml, "serverVersion", &self.server_version);
        xml::attr(xml, "openSubsonic", &self.open_subsonic);
        xml::elem_begin_close(xml);
        if let Some(ref body) = self.body {
            XmlSerialize::serialize(body, xml);
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for ResponseBody {
    fn serialize(&self, xml: &mut xml::Xml) {
        match self {
            ResponseBody::MusicFolders(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Indexes(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Directory(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Genres(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Artists(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Artist(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Album(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Song(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Videos(_) => todo!(),
            ResponseBody::VideoInfo(_) => todo!(),
            ResponseBody::NowPlaying(_) => todo!(),
            ResponseBody::SearchResult(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::SearchResult2(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::SearchResult3(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Playlists(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Playlist(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::JukeboxStatus(_) => todo!(),
            ResponseBody::JukeboxPlaylist(_) => todo!(),
            ResponseBody::JukeboxControlResponse(_) => todo!(),
            ResponseBody::License(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Users(_) => todo!(),
            ResponseBody::User(_) => todo!(),
            ResponseBody::ChatMessages(_) => todo!(),
            ResponseBody::AlbumList(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::AlbumList2(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::RandomSongs(_) => todo!(),
            ResponseBody::SongsByGenre(_) => todo!(),
            ResponseBody::Lyrics(_) => todo!(),
            ResponseBody::Podcasts(_) => todo!(),
            ResponseBody::NewestPodcasts(_) => todo!(),
            ResponseBody::InternetRadioStations(_) => todo!(),
            ResponseBody::Bookmarks(_) => todo!(),
            ResponseBody::PlayQueue(_) => todo!(),
            ResponseBody::Shares(_) => todo!(),
            ResponseBody::Starred(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::Starred2(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::AlbumInfo(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::ArtistInfo(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::ArtistInfo2(v) => XmlSerialize::serialize(v, xml),
            ResponseBody::SimilarSongs(_) => todo!(),
            ResponseBody::SimilarSongs2(_) => todo!(),
            ResponseBody::TopSongs(_) => todo!(),
            ResponseBody::ScanStatus(_) => todo!(),
            ResponseBody::Error(v) => XmlSerialize::serialize(v, xml),
        }
    }
}

impl XmlSerialize for License {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "license");
        xml::attr(xml, "valid", &self.valid);
        xml::attr_opt(xml, "email", &self.email);
        xml::attr_opt(xml, "licenseExpires", &self.license_expires);
        xml::attr_opt(xml, "trialExpires", &self.trial_expires);
        xml::elem_begin_close_end(xml);
    }
}

impl XmlSerialize for MusicFolder {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "musicFolder");
        xml::attr(xml, "id", &self.id);
        xml::attr_opt(xml, "name", &self.name);
        xml::elem_begin_close_end(xml);
    }
}

impl XmlSerialize for MusicFolders {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "musicFolders");
        xml::elem_begin_close(xml);
        for folder in &self.music_folder {
            XmlSerialize::serialize(folder, xml);
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Index {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "index");
        xml::attr(xml, "name", &self.name);
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Indexes {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "indexes");
        xml::attr(xml, "lastModified", &self.last_modified);
        xml::attr(xml, "ignoredArticles", &self.ignored_articles);
        xml::elem_begin_close(xml);

        // TODO: shortcuts

        for index in &self.index {
            XmlSerialize::serialize(index, xml);
        }
        for child in &self.child {
            XmlSerialize::serialize(child, xml);
        }

        xml::elem_end(xml);
    }
}

impl Artist {
    fn serialize_as(&self, xml: &mut xml::Xml, element: &'static str) {
        xml::elem_begin_open(xml, element);
        xml::attr(xml, "id", &self.id);
        xml::attr(xml, "name", &self.name);
        xml::attr_opt(xml, "artistImageUrl", &self.artist_image_url);
        xml::attr_opt(xml, "starred", &self.starred);
        xml::attr_opt(xml, "userRating", &self.user_rating);
        xml::attr_opt(xml, "averageRating", &self.average_rating);
        xml::elem_begin_close_end(xml);
    }
}

impl XmlSerialize for Artist {
    fn serialize(&self, xml: &mut xml::Xml) {
        self.serialize_as(xml, "artist");
    }
}

impl XmlSerialize for Genres {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "genres");
        xml::elem_begin_close(xml);
        for genre in &self.genre {
            XmlSerialize::serialize(genre, xml);
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Genre {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "genre");
        xml::attr(xml, "songCount", &self.song_count);
        xml::attr(xml, "albumCount", &self.album_count);
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.name);
        xml::elem_end(xml);
    }
}

impl XmlSerialize for ArtistsID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "artists");
        xml::attr(xml, "ignoredArticles", &self.ignored_articles);
        xml::elem_begin_close(xml);
        for index in &self.index {
            XmlSerialize::serialize(index, xml);
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for IndexID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "index");
        xml::attr(xml, "name", &self.name);
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        xml::elem_end(xml);
    }
}

impl ArtistID3 {
    fn serialize_as(&self, xml: &mut xml::Xml, element: &'static str) {
        xml::elem_begin_open(xml, element);
        self.serialize_attributes(xml);
        xml::elem_begin_close_end(xml);
    }

    fn serialize_attributes(&self, xml: &mut xml::Xml) {
        xml::attr(xml, "id", &self.id);
        xml::attr(xml, "name", &self.name);
        xml::attr_opt(xml, "coverArt", &self.cover_art);
        xml::attr(xml, "albumCount", &self.album_count);
        xml::attr_opt(xml, "starred", &self.starred);
    }
}

impl XmlSerialize for ArtistID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        self.serialize_as(xml, "artist");
    }
}

impl XmlSerialize for ArtistWithAlbumsID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "artist");
        self.artist.serialize_attributes(xml);
        xml::elem_begin_close(xml);
        for album in &self.album {
            XmlSerialize::serialize(album, xml);
        }
        xml::elem_end(xml);
    }
}

impl AlbumID3 {
    fn serialize_attributes(&self, xml: &mut xml::Xml) {
        xml::attr(xml, "id", &self.id);
        xml::attr(xml, "name", &self.name);
        xml::attr_opt(xml, "artist", &self.artist);
        xml::attr_opt(xml, "artistId", &self.artist_id);
        xml::attr_opt(xml, "coverArt", &self.cover_art);
        xml::attr(xml, "songCount", &self.song_count);
        xml::attr(xml, "duration", &self.duration);
        xml::attr_opt(xml, "playCount", &self.play_count);
        xml::attr(xml, "created", &self.created);
        xml::attr_opt(xml, "starred", &self.starred);
        xml::attr_opt(xml, "year", &self.year);
        xml::attr_opt(xml, "genre", &self.genre);
        xml::attr_opt(xml, "userRating", &self.user_rating);
    }
}

impl XmlSerialize for AlbumID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "album");
        self.serialize_attributes(xml);
        xml::elem_begin_close_end(xml);
    }
}

impl XmlSerialize for AlbumWithSongsID3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "album");
        self.album.serialize_attributes(xml);
        xml::elem_begin_close(xml);
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Directory {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "directory");
        xml::attr(xml, "id", &self.id);
        xml::attr_opt(xml, "parent", &self.parent);
        xml::attr(xml, "name", &self.name);
        xml::attr_opt(xml, "starred", &self.starred);
        xml::attr_opt(xml, "userRating", &self.user_rating);
        xml::attr_opt(xml, "averageRating", &self.average_rating);
        xml::attr_opt(xml, "playCount", &self.play_count);
        xml::elem_begin_close(xml);
        for child in &self.child {
            XmlSerialize::serialize(child, xml);
        }
        xml::elem_end(xml);
    }
}

impl Child {
    fn serialize_as(&self, xml: &mut xml::Xml, element: &'static str) {
        xml::elem_begin_open(xml, element);
        xml::attr(xml, "id", &self.id);
        xml::attr_opt(xml, "parent", &self.parent);
        xml::attr(xml, "isDir", &self.is_dir);
        xml::attr(xml, "title", &self.title);
        xml::attr_opt(xml, "album", &self.album);
        xml::attr_opt(xml, "artist", &self.artist);
        xml::attr_opt(xml, "track", &self.track);
        xml::attr_opt(xml, "year", &self.year);
        xml::attr_opt(xml, "genre", &self.genre);
        xml::attr_opt(xml, "coverArt", &self.cover_art);
        xml::attr_opt(xml, "size", &self.size);
        xml::attr_opt(xml, "contentType", &self.content_type);
        xml::attr_opt(xml, "suffix", &self.suffix);
        xml::attr_opt(xml, "transcodedContentType", &self.transcoded_content_type);
        xml::attr_opt(xml, "transcodedSuffix", &self.transcoded_suffix);
        xml::attr_opt(xml, "duration", &self.duration);
        xml::attr_opt(xml, "bitRate", &self.bit_rate);
        xml::attr_opt(xml, "path", &self.path);
        xml::attr_opt(xml, "isVideo", &self.is_video);
        xml::attr_opt(xml, "userRating", &self.user_rating);
        xml::attr_opt(xml, "averageRating", &self.average_rating);
        xml::attr_opt(xml, "playCount", &self.play_count);
        xml::attr_opt(xml, "discNumber", &self.disc_number);
        xml::attr_opt(xml, "created", &self.disc_number);
        xml::attr_opt(xml, "starred", &self.starred);
        xml::attr_opt(xml, "albumId", &self.album_id);
        xml::attr_opt(xml, "artistId", &self.artist_id);
        xml::attr_opt(xml, "type", &self.media_type);
        xml::attr_opt(xml, "bookmarkPosition", &self.bookmark_position);
        xml::attr_opt(xml, "originalWidth", &self.original_width);
        xml::attr_opt(xml, "originalHeight", &self.original_height);
        xml::elem_begin_close_end(xml);
    }
}

impl XmlSerialize for Child {
    fn serialize(&self, xml: &mut xml::Xml) {
        self.serialize_as(xml, "child");
    }
}

impl XmlSerialize for SearchResult {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "searchResult");
        xml::attr(xml, "offset", &self.offset);
        xml::attr(xml, "totalHits", &self.total_hits);
        for m in &self.matches {
            m.serialize_as(xml, "match");
        }
        xml::elem_begin_close(xml);
    }
}

impl XmlSerialize for SearchResult2 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "searchResult2");
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        for album in &self.album {
            album.serialize_as(xml, "album");
        }
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for SearchResult3 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "searchResult3");
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        for album in &self.album {
            XmlSerialize::serialize(album, xml);
        }
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Playlists {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "playlists");
        xml::elem_begin_close(xml);
        for playlist in &self.playlist {
            XmlSerialize::serialize(playlist, xml);
        }
        xml::elem_end(xml);
    }
}

impl Playlist {
    fn serialize_attributes(&self, xml: &mut xml::Xml) {
        xml::attr(xml, "id", &self.id);
        xml::attr(xml, "name", &self.name);
        xml::attr_opt(xml, "comment", &self.comment);
        xml::attr_opt(xml, "owner", &self.owner);
        xml::attr_opt(xml, "public", &self.public);
        xml::attr(xml, "songCount", &self.song_count);
        xml::attr(xml, "duration", &self.duration);
        xml::attr(xml, "created", &self.created);
        xml::attr(xml, "changed", &self.changed);
        xml::attr_opt(xml, "coverArt", &self.cover_art);
    }

    fn serialize_allowed_users(&self, xml: &mut xml::Xml) {
        for allowed_user in &self.allowed_user {
            xml::elem_begin_open(xml, "allowedUser");
            xml::elem_begin_close(xml);
            xml::body_display(xml, allowed_user);
            xml::elem_end(xml);
        }
    }
}

impl XmlSerialize for Playlist {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "playlist");
        self.serialize_attributes(xml);
        xml::elem_begin_close(xml);
        self.serialize_allowed_users(xml);
        xml::elem_end(xml);
    }
}

impl XmlSerialize for PlaylistWithSongs {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "playlist");
        self.playlist.serialize_attributes(xml);
        xml::elem_begin_close(xml);
        self.playlist.serialize_allowed_users(xml);
        for entry in &self.entry {
            entry.serialize_as(xml, "entry");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for AlbumList {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "albumList");
        xml::elem_begin_close(xml);
        for album in &self.album {
            album.serialize_as(xml, "album");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for AlbumList2 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "albumList2");
        xml::elem_begin_close(xml);
        for album in &self.album {
            XmlSerialize::serialize(album, xml);
        }
        xml::elem_end(xml);
    }
}

impl Songs {
    fn serialize_as(&self, xml: &mut xml::Xml, element: &'static str) {
        xml::elem_begin_open(xml, element);
        xml::elem_begin_close(xml);
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Songs {
    fn serialize(&self, xml: &mut xml::Xml) {
        self.serialize_as(xml, "songs");
    }
}

impl XmlSerialize for Starred {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "starred");
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        for album in &self.album {
            album.serialize_as(xml, "album");
        }
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for AlbumInfo {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "albumInfo");
        xml::elem_begin_close(xml);

        xml::elem_begin_open(xml, "notes");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.notes);
        xml::elem_end(xml);

        xml::elem_begin_open(xml, "musicBrainzId");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.music_brainz_id);
        xml::elem_end(xml);

        xml::elem_begin_open(xml, "lastFmUrl");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.last_fm_url);
        xml::elem_end(xml);

        xml::elem_begin_open(xml, "smallImageUrl");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.small_image_url);
        xml::elem_end(xml);

        xml::elem_begin_open(xml, "mediumImageUrl");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.medium_image_url);
        xml::elem_end(xml);

        xml::elem_begin_open(xml, "largeImageUrl");
        xml::elem_begin_close(xml);
        xml::body_display(xml, &self.large_image_url);
        xml::elem_end(xml);

        xml::elem_end(xml);
    }
}

impl ArtistInfoBase {
    fn serialize(&self, xml: &mut xml::Xml) {
        if let Some(ref biography) = self.biography {
            xml::elem_begin_open(xml, "biography");
            xml::elem_begin_close(xml);
            xml::body_display(xml, biography);
            xml::elem_end(xml);
        }

        if let Some(ref musicbrainz_id) = self.music_brainz_id {
            xml::elem_begin_open(xml, "musicBrainzId");
            xml::elem_begin_close(xml);
            xml::body_display(xml, musicbrainz_id);
            xml::elem_end(xml);
        }

        if let Some(ref last_fm_url) = self.last_fm_url {
            xml::elem_begin_open(xml, "lastFmUrl");
            xml::elem_begin_close(xml);
            xml::body_display(xml, last_fm_url);
            xml::elem_end(xml);
        }

        if let Some(ref small_image_url) = self.small_image_url {
            xml::elem_begin_open(xml, "smallImageUrl");
            xml::elem_begin_close(xml);
            xml::body_display(xml, small_image_url);
            xml::elem_end(xml);
        }

        if let Some(ref medium_image_url) = self.medium_image_url {
            xml::elem_begin_open(xml, "mediumImageUrl");
            xml::elem_begin_close(xml);
            xml::body_display(xml, medium_image_url);
            xml::elem_end(xml);
        }

        if let Some(ref large_image_url) = self.large_image_url {
            xml::elem_begin_open(xml, "largeImageUrl");
            xml::elem_begin_close(xml);
            xml::body_display(xml, large_image_url);
            xml::elem_end(xml);
        }
    }
}

impl XmlSerialize for ArtistInfo {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "artistInfo");
        xml::elem_begin_close(xml);

        self.info.serialize(xml);
        for similar in &self.similar_artist {
            similar.serialize_as(xml, "similarArtist");
        }

        xml::elem_end(xml);
    }
}

impl XmlSerialize for ArtistInfo2 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "artistInfo2");
        xml::elem_begin_close(xml);

        self.info.serialize(xml);
        for similar in &self.similar_artist {
            similar.serialize_as(xml, "similarArtist");
        }

        xml::elem_end(xml);
    }
}

impl XmlSerialize for Starred2 {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "starred2");
        xml::elem_begin_close(xml);
        for artist in &self.artist {
            XmlSerialize::serialize(artist, xml);
        }
        for album in &self.album {
            XmlSerialize::serialize(album, xml);
        }
        for song in &self.song {
            song.serialize_as(xml, "song");
        }
        xml::elem_end(xml);
    }
}

impl XmlSerialize for Error {
    fn serialize(&self, xml: &mut xml::Xml) {
        xml::elem_begin_open(xml, "error");
        xml::attr(xml, "code", &u32::from(self.code));
        xml::attr_opt(xml, "message", &self.message);
        xml::elem_begin_close_end(xml);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_xml_license() {
        insta::assert_snapshot!(xml::serialize(&License {
            valid: true,
            email: Some("hello@world.com".to_string()),
            ..Default::default()
        }));
    }

    #[test]
    fn test_xml_music_folder() {
        insta::assert_snapshot!(xml::serialize(&MusicFolder {
            id: 5,
            name: Some("folder name".to_string())
        }));
    }

    #[test]
    fn test_xml_music_folders() {
        insta::assert_snapshot!(xml::serialize(&MusicFolders {
            music_folder: vec![
                MusicFolder {
                    id: 1,
                    name: Some("folder 1 name".to_string())
                },
                MusicFolder {
                    id: 2,
                    name: Some("folder 2 name".to_string())
                }
            ]
        }));
    }
}
