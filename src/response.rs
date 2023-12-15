//! Module for Subsonic API responses.
//!
//! # Example
//! Building a response:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use subsonic_types::{common::Version, response::{Response, ResponseBody, License}};
//!     let response = Response::ok(
//!         Version::V1_16_1,
//!         ResponseBody::License(License {
//!             valid: true,
//!             ..Default::default()
//!         }),
//!     );
//!     assert_eq!(
//!         r#"{"subsonic-response":{"status":"ok","version":"1.16.1","license":{"valid":true}}}"#,
//!         Response::to_json(&response)?
//!     );
//! # Ok(())
//! # }
//! ```
//!
//! Parsing a response:
//! Deserialize a response from json
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use subsonic_types::{common::Version, response::{Response, ResponseBody, License}};
//!     let response = Response::ok(
//!         Version::V1_16_1,
//!         ResponseBody::License(License {
//!             valid: true,
//!             ..Default::default()
//!         }),
//!     );
//!     let serialized = r#"{"subsonic-response":{"status":"ok","version":"1.16.1","license":{"valid":true}}}"#;
//!     let deserialized = Response::from_json(serialized)?;
//!     assert_eq!(
//!         response,
//!         deserialized
//!     );
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::common::{
    AverageRating, DateTime, MediaType, Milliseconds, Seconds, UserRating, Version,
};

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
    Indexes(Indexes),
    Directory(Directory),
    Genres(Genres),
    Artist(Artist),
    #[serde(rename = "indexes")]
    Artists(ArtistsID3),
    ArtistWithAlbums(ArtistWithAlbumsID3),
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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumID3 {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub song_count: u32,
    pub duration: u32,
    pub play_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
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
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub public: Option<bool>,
    pub song_count: u32,
    pub duration: Seconds,
    pub created: DateTime,
    pub changed: DateTime,
    pub cover_art: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub notes: Vec<String>,
    pub music_brainz_id: Vec<String>,
    pub last_fm_url: Vec<String>,
    pub small_image_url: Vec<String>,
    pub medium_image_url: Vec<String>,
    pub large_image_url: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfoBase {
    pub biography: Vec<String>,
    pub music_brainz_id: Vec<String>,
    pub last_fm_url: Vec<String>,
    pub small_image_url: Vec<String>,
    pub medium_image_url: Vec<String>,
    pub large_image_url: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo {
    #[serde(flatten)]
    pub info: ArtistInfoBase,
    pub similar_artist: Vec<Artist>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
