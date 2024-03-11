use async_trait::async_trait;
use sonar_grpc::{Album, Artist, Track};
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};

const MPD_DEFAULT_PORT: u16 = 6600;

#[derive(Debug)]
pub struct Error {}

impl Error {
    fn todo() -> Self {
        Error {}
    }

    fn unsupported() -> Self {
        Error {}
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::todo()
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct Arguments {
    args: VecDeque<String>,
}

impl Arguments {
    pub fn consume(&mut self) -> Result<String> {
        self.args.pop_front().ok_or_else(|| Error::todo())
    }

    pub fn consume_parse<T: std::str::FromStr>(&mut self) -> Result<T> {
        self.consume()?.parse().map_err(|_| Error::todo())
    }

    pub fn consume_optional(&mut self) -> Result<Option<String>> {
        Ok(self.args.pop_front())
    }

    pub fn consume_optional_parse<T: std::str::FromStr>(&mut self) -> Result<Option<T>> {
        match self.args.front() {
            Some(arg) => {
                let arg = self.consume()?;
                arg.parse().map(Some).map_err(|_| Error::todo())
            }
            None => Ok(None),
        }
    }

    pub fn consume_all(self) -> Vec<String> {
        self.args.into_iter().collect()
    }

    pub fn expect_empty(self) -> Result<()> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(Error::todo())
        }
    }
}

#[derive(Debug, Default)]
struct Responder {
    buffer: Vec<u8>,
}

impl Responder {
    pub fn key_value(
        &mut self,
        key: impl std::fmt::Display,
        value: impl std::fmt::Display,
    ) -> Result<()> {
        use std::io::Write;
        write!(self.buffer, "{}: {}\n", key, value).map_err(|_| Error::todo())?;
        Ok(())
    }

    fn ok(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(b"OK\n");
        Ok(())
    }

    fn list_ok(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(b"list_OK\n");
        Ok(())
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Debug)]
pub struct InvalidTagError;

impl std::fmt::Display for InvalidTagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid tag")
    }
}

impl std::error::Error for InvalidTagError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    Artist,
    ArtistSort,
    Album,
    AlbumSort,
    AlbumArtist,
    AlbumArtistSort,
    Title,
    Titlesort,
    Track,
    Name,
    Genre,
    Mood,
    Date,
    OriginalDate,
    Composer,
    ComposerSort,
    Performer,
    Conductor,
    Work,
    Ensemble,
    Movement,
    Movementnumber,
    Location,
    Grouping,
    Comment,
    Disc,
    Label,
    MusicbrainzArtistid,
    MusicbrainzAlbumid,
    MusicbrainzAlbumartistid,
    MusicbrainzTrackid,
    MusicbrainzReleasegroupid,
    MusicbrainzReleasetrackid,
    MusicbrainzWorkid,
}

impl FromStr for Tag {
    type Err = InvalidTagError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("Artist") {
            Ok(Self::Artist)
        } else if s.eq_ignore_ascii_case("Artistsort") {
            Ok(Self::ArtistSort)
        } else if s.eq_ignore_ascii_case("Album") {
            Ok(Self::Album)
        } else if s.eq_ignore_ascii_case("Albumsort") {
            Ok(Self::AlbumSort)
        } else if s.eq_ignore_ascii_case("Albumartist") {
            Ok(Self::AlbumArtist)
        } else if s.eq_ignore_ascii_case("Albumartistsort") {
            Ok(Self::AlbumArtistSort)
        } else if s.eq_ignore_ascii_case("Title") {
            Ok(Self::Title)
        } else if s.eq_ignore_ascii_case("Titlesort") {
            Ok(Self::Titlesort)
        } else if s.eq_ignore_ascii_case("Track") {
            Ok(Self::Track)
        } else if s.eq_ignore_ascii_case("Name") {
            Ok(Self::Name)
        } else if s.eq_ignore_ascii_case("Genre") {
            Ok(Self::Genre)
        } else if s.eq_ignore_ascii_case("Mood") {
            Ok(Self::Mood)
        } else if s.eq_ignore_ascii_case("Date") {
            Ok(Self::Date)
        } else if s.eq_ignore_ascii_case("Originaldate") {
            Ok(Self::OriginalDate)
        } else if s.eq_ignore_ascii_case("Composer") {
            Ok(Self::Composer)
        } else if s.eq_ignore_ascii_case("Composersort") {
            Ok(Self::ComposerSort)
        } else if s.eq_ignore_ascii_case("Performer") {
            Ok(Self::Performer)
        } else if s.eq_ignore_ascii_case("Conductor") {
            Ok(Self::Conductor)
        } else if s.eq_ignore_ascii_case("Work") {
            Ok(Self::Work)
        } else if s.eq_ignore_ascii_case("Ensemble") {
            Ok(Self::Ensemble)
        } else if s.eq_ignore_ascii_case("Movement") {
            Ok(Self::Movement)
        } else if s.eq_ignore_ascii_case("Movementnumber") {
            Ok(Self::Movementnumber)
        } else if s.eq_ignore_ascii_case("Location") {
            Ok(Self::Location)
        } else if s.eq_ignore_ascii_case("Grouping") {
            Ok(Self::Grouping)
        } else if s.eq_ignore_ascii_case("Comment") {
            Ok(Self::Comment)
        } else if s.eq_ignore_ascii_case("Disc") {
            Ok(Self::Disc)
        } else if s.eq_ignore_ascii_case("Label") {
            Ok(Self::Label)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Artistid") {
            Ok(Self::MusicbrainzArtistid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Albumid") {
            Ok(Self::MusicbrainzAlbumid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Albumartistid") {
            Ok(Self::MusicbrainzAlbumartistid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Trackid") {
            Ok(Self::MusicbrainzTrackid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Releasegroupid") {
            Ok(Self::MusicbrainzReleasegroupid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Releasetrackid") {
            Ok(Self::MusicbrainzReleasetrackid)
        } else if s.eq_ignore_ascii_case("Musicbrainz_Workid") {
            Ok(Self::MusicbrainzWorkid)
        } else {
            Err(InvalidTagError)
        }
    }
}

#[derive(Debug)]
struct InvalidRangeError;

impl std::fmt::Display for InvalidRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid range")
    }
}

impl std::error::Error for InvalidRangeError {}

/// MPD range. The range is not inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: u32,
    end: Option<u32>,
}

impl FromStr for Range {
    type Err = InvalidRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((left, right)) => {
                let start = left.parse().map_err(|_| InvalidRangeError)?;
                if right.is_empty() {
                    Ok(Self { start, end: None })
                } else {
                    let end = right.parse().map_err(|_| InvalidRangeError)?;
                    Ok(Self {
                        start,
                        end: Some(end),
                    })
                }
            }
            None => {
                let start = s.parse().map_err(|_| InvalidRangeError)?;
                Ok(Self {
                    start,
                    end: Some(start + 1),
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Subsystem {
    Audio,
    Database,
    Mixer,
    Options,
    Playlist,
    Sticker,
    Update,
    StoredPlaylist,
    Partition,
    StickerCache,
    Subscription,
    Message,
    Neighbor,
    Output,
    Reflection,
    Stats,
    Status,
    Mount,
    Command,
    Notifier,
    Unknown(String),
}

trait MpdRequest: Sized {
    const COMMAND: &'static str;

    fn parse_arguments(args: Arguments) -> Result<Self>;
}

#[derive(Debug)]
struct CommandListBegin;

impl MpdRequest for CommandListBegin {
    const COMMAND: &'static str = "command_list_begin";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct CommandListOkBegin;

impl MpdRequest for CommandListOkBegin {
    const COMMAND: &'static str = "command_list_ok_begin";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct CommandListEnd;

impl MpdRequest for CommandListEnd {
    const COMMAND: &'static str = "command_list_end";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

// status commands

#[derive(Debug)]
struct ClearError;

impl MpdRequest for ClearError {
    const COMMAND: &'static str = "clearerror";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct CurrentSong;

impl MpdRequest for CurrentSong {
    const COMMAND: &'static str = "currentsong";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Idle {
    subsystems: Vec<Subsystem>,
}

impl MpdRequest for Idle {
    const COMMAND: &'static str = "idle";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        todo!()
    }
}

#[derive(Debug)]
struct Status;

impl MpdRequest for Status {
    const COMMAND: &'static str = "status";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Stats;

impl MpdRequest for Stats {
    const COMMAND: &'static str = "stats";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Ok(Self)
    }
}

// playback commands

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ConsumeMode {
    On,
    Off,
    OneShot,
}

#[derive(Debug)]
struct Consume {
    mode: ConsumeMode,
}

impl MpdRequest for Consume {
    const COMMAND: &'static str = "consume";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume()?.as_str() {
            "0" => ConsumeMode::Off,
            "1" => ConsumeMode::On,
            "oneshot" => ConsumeMode::OneShot,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug)]
struct Crossfade {
    duration: Duration,
}

impl MpdRequest for Crossfade {
    const COMMAND: &'static str = "crossfade";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let duration = Duration::from_secs_f64(args.consume_parse()?);
        args.expect_empty()?;
        Ok(Self { duration })
    }
}

#[derive(Debug)]
struct MixRampDb {
    value: f64,
}

impl MpdRequest for MixRampDb {
    const COMMAND: &'static str = "mixrampdb";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let value = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { value })
    }
}

#[derive(Debug)]
struct MixRampDelay {
    duration: Duration,
}

impl MpdRequest for MixRampDelay {
    const COMMAND: &'static str = "mixrampdelay";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let duration = Duration::from_secs_f64(args.consume_parse()?);
        args.expect_empty()?;
        Ok(Self { duration })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RandomMode {
    On,
    Off,
}

#[derive(Debug)]
struct Random {
    mode: RandomMode,
}

impl MpdRequest for Random {
    const COMMAND: &'static str = "random";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume()?.as_str() {
            "0" => RandomMode::Off,
            "1" => RandomMode::On,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RepeatMode {
    On,
    Off,
}

#[derive(Debug)]
struct Repeat {
    mode: RepeatMode,
}

impl MpdRequest for Repeat {
    const COMMAND: &'static str = "repeat";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume()?.as_str() {
            "0" => RepeatMode::Off,
            "1" => RepeatMode::On,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug)]
struct SetVol {
    volume: u8,
}

impl MpdRequest for SetVol {
    const COMMAND: &'static str = "setvol";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let volume = args.consume_parse()?;
        if volume > 100 {
            return Err(Error::todo());
        }
        args.expect_empty()?;
        Ok(Self { volume })
    }
}

#[derive(Debug)]
struct GetVol;

impl MpdRequest for GetVol {
    const COMMAND: &'static str = "getvol";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SingleMode {
    On,
    Off,
    OneShot,
}

#[derive(Debug)]
struct Single {
    mode: SingleMode,
}

impl MpdRequest for Single {
    const COMMAND: &'static str = "single";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume()?.as_str() {
            "0" => SingleMode::Off,
            "1" => SingleMode::On,
            "oneshot" => SingleMode::OneShot,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum ReplayGainMode {
    Off,
    Track,
    Album,
    Auto,
}

#[derive(Debug)]
struct ReplayGain {
    mode: ReplayGainMode,
}

impl MpdRequest for ReplayGain {
    const COMMAND: &'static str = "replay_gain_mode";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume()?.as_str() {
            "off" => ReplayGainMode::Off,
            "track" => ReplayGainMode::Track,
            "album" => ReplayGainMode::Album,
            "auto" => ReplayGainMode::Auto,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug)]
struct ReplayGainStatus;

impl MpdRequest for ReplayGainStatus {
    const COMMAND: &'static str = "replay_gain_status";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Volume {
    change: i32,
}

impl MpdRequest for Volume {
    const COMMAND: &'static str = "volume";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let change = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { change })
    }
}

// controll commands

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseMode {
    Pause,
    Resume,
    Toggle,
}

#[derive(Debug)]
struct Next;

impl MpdRequest for Next {
    const COMMAND: &'static str = "next";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Pause {
    mode: PauseMode,
}

impl MpdRequest for Pause {
    const COMMAND: &'static str = "pause";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let mode = match args.consume_optional()?.as_deref() {
            Some("0") => PauseMode::Resume,
            Some("1") => PauseMode::Pause,
            Some(_) => return Err(Error::todo()),
            None => PauseMode::Toggle,
        };
        args.expect_empty()?;
        Ok(Self { mode })
    }
}

#[derive(Debug)]
struct Play {
    position: u32,
}

impl MpdRequest for Play {
    const COMMAND: &'static str = "play";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let position = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { position })
    }
}

#[derive(Debug)]
struct PlayId {
    id: u32,
}

impl MpdRequest for PlayId {
    const COMMAND: &'static str = "playid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { id })
    }
}

#[derive(Debug)]
struct Previous;

impl MpdRequest for Previous {
    const COMMAND: &'static str = "previous";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Seek {
    position: u32,
    time: Duration,
}

impl MpdRequest for Seek {
    const COMMAND: &'static str = "seek";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let position = args.consume_parse()?;
        let time = Duration::from_secs_f64(args.consume_parse()?);
        args.expect_empty()?;
        Ok(Self { position, time })
    }
}

#[derive(Debug)]
struct SeekId {
    id: String,
    time: Duration,
}

impl MpdRequest for SeekId {
    const COMMAND: &'static str = "seekid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        let time = Duration::from_secs_f64(args.consume_parse()?);
        args.expect_empty()?;
        Ok(Self { id, time })
    }
}

#[derive(Debug)]
struct SeekCur {
    time: Duration,
}

impl MpdRequest for SeekCur {
    const COMMAND: &'static str = "seekcur";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let time = Duration::from_secs_f64(args.consume_parse()?);
        args.expect_empty()?;
        Ok(Self { time })
    }
}

#[derive(Debug)]
struct Stop;

impl MpdRequest for Stop {
    const COMMAND: &'static str = "stop";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

// queue commands

#[derive(Debug)]
struct Add {
    uri: String,
    position: Option<u32>,
}

impl MpdRequest for Add {
    const COMMAND: &'static str = "add";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        let position = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { uri, position })
    }
}

#[derive(Debug)]
struct AddId {
    uri: String,
    position: Option<u32>,
}

impl MpdRequest for AddId {
    const COMMAND: &'static str = "addid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        let position = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { uri, position })
    }
}

#[derive(Debug)]
struct Clear;

impl MpdRequest for Clear {
    const COMMAND: &'static str = "clear";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Delete {
    range: Range,
}

impl MpdRequest for Delete {
    const COMMAND: &'static str = "delete";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let range = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { range })
    }
}

#[derive(Debug)]
struct DeleteId {
    id: String,
}

impl MpdRequest for DeleteId {
    const COMMAND: &'static str = "deleteid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        args.expect_empty()?;
        Ok(Self { id })
    }
}

#[derive(Debug)]
struct Move {
    range: Range,
    // TODO: maybe add Position type
    to: u32,
}

impl MpdRequest for Move {
    const COMMAND: &'static str = "move";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let range = args.consume_parse()?;
        let to = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { range, to })
    }
}

#[derive(Debug)]
struct MoveId {
    id: String,
    // TODO: maybe add Position type
    to: u32,
}

impl MpdRequest for MoveId {
    const COMMAND: &'static str = "moveid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        let to = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { id, to })
    }
}

#[derive(Debug)]
struct Playlist;

impl MpdRequest for Playlist {
    const COMMAND: &'static str = "playlist";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct PlaylistFind {
    // TODO
}

impl MpdRequest for PlaylistFind {
    const COMMAND: &'static str = "playlistfind";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct PlaylistId {
    song_id: Option<String>,
}

impl MpdRequest for PlaylistId {
    const COMMAND: &'static str = "playlistid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let song_id = args.consume_optional()?;
        args.expect_empty()?;
        Ok(Self { song_id })
    }
}

#[derive(Debug)]
struct PlaylistInfo {
    range: Option<Range>,
}

impl MpdRequest for PlaylistInfo {
    const COMMAND: &'static str = "playlistinfo";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { range })
    }
}

#[derive(Debug)]
struct PlaylistSearch {
    // TODO
}

impl MpdRequest for PlaylistSearch {
    const COMMAND: &'static str = "playlistsearch";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct PlChanges {
    version: u32,
    range: Option<Range>,
}

impl MpdRequest for PlChanges {
    const COMMAND: &'static str = "plchanges";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let version = args.consume_parse()?;
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { version, range })
    }
}

#[derive(Debug)]
struct PlChangesPosId {
    version: u32,
    range: Option<Range>,
}

impl MpdRequest for PlChangesPosId {
    const COMMAND: &'static str = "plchangesposid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let version = args.consume_parse()?;
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { version, range })
    }
}

#[derive(Debug)]
struct Prio {
    priority: u32,
    range: Range,
}

impl MpdRequest for Prio {
    const COMMAND: &'static str = "prio";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let priority = args.consume_parse()?;
        let range = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { priority, range })
    }
}

#[derive(Debug)]
struct PrioId {
    priority: u32,
    ids: Vec<String>,
}

impl MpdRequest for PrioId {
    const COMMAND: &'static str = "prioid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let priority = args.consume_parse()?;
        let ids = args.consume_all();
        Ok(Self { priority, ids })
    }
}

#[derive(Debug)]
struct RangeId {
    id: String,
    range: Range,
}

impl MpdRequest for RangeId {
    const COMMAND: &'static str = "rangeid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        let range = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { id, range })
    }
}

#[derive(Debug)]
struct Shuffle {
    range: Option<Range>,
}

impl MpdRequest for Shuffle {
    const COMMAND: &'static str = "shuffle";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { range })
    }
}

#[derive(Debug)]
struct Swap {
    song1: u32,
    song2: u32,
}

impl MpdRequest for Swap {
    const COMMAND: &'static str = "swap";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let song1 = args.consume_parse()?;
        let song2 = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { song1, song2 })
    }
}

#[derive(Debug)]
struct SwapId {
    id1: String,
    id2: String,
}

impl MpdRequest for SwapId {
    const COMMAND: &'static str = "swapid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id1 = args.consume()?;
        let id2 = args.consume()?;
        args.expect_empty()?;
        Ok(Self { id1, id2 })
    }
}

#[derive(Debug)]
struct AddTagId {
    id: String,
    tag: Tag,
    value: String,
}

impl MpdRequest for AddTagId {
    const COMMAND: &'static str = "addtagid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        let tag = args.consume_parse()?;
        let value = args.consume()?;
        args.expect_empty()?;
        Ok(Self { id, tag, value })
    }
}

#[derive(Debug)]
struct ClearTagId {
    id: String,
    tag: Tag,
}

impl MpdRequest for ClearTagId {
    const COMMAND: &'static str = "cleartagid";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let id = args.consume()?;
        let tag = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { id, tag })
    }
}

// playlist commands

#[derive(Debug)]
struct InvalidPlaylistSaveMode;

impl std::fmt::Display for InvalidPlaylistSaveMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid playlist save mode")
    }
}

impl std::error::Error for InvalidPlaylistSaveMode {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlaylistSaveMode {
    Create,
    Append,
    Replace,
}

impl FromStr for PlaylistSaveMode {
    type Err = InvalidPlaylistSaveMode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Self::Create),
            "append" => Ok(Self::Append),
            "replace" => Ok(Self::Replace),
            _ => Err(InvalidPlaylistSaveMode),
        }
    }
}

#[derive(Debug)]
struct ListPlaylist {
    name: String,
    range: Option<Range>,
}

impl MpdRequest for ListPlaylist {
    const COMMAND: &'static str = "listplaylist";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { name, range })
    }
}

#[derive(Debug)]
struct ListPlaylistInfo {
    name: String,
    range: Option<Range>,
}

impl MpdRequest for ListPlaylistInfo {
    const COMMAND: &'static str = "listplaylistinfo";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let range = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { name, range })
    }
}

#[derive(Debug)]
struct ListPlaylists;

impl MpdRequest for ListPlaylists {
    const COMMAND: &'static str = "listplaylists";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Load {
    name: String,
    range: Option<Range>,
    position: Option<u32>,
}

impl MpdRequest for Load {
    const COMMAND: &'static str = "load";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let range = args.consume_optional_parse()?;
        let position = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self {
            name,
            range,
            position,
        })
    }
}

#[derive(Debug)]
struct PlaylistAdd {
    name: String,
    uri: String,
    position: Option<u32>,
}

impl MpdRequest for PlaylistAdd {
    const COMMAND: &'static str = "playlistadd";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let uri = args.consume()?;
        let position = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self {
            name,
            uri,
            position,
        })
    }
}

#[derive(Debug)]
struct PlaylistClear {
    name: String,
}

impl MpdRequest for PlaylistClear {
    const COMMAND: &'static str = "playlistclear";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        args.expect_empty()?;
        Ok(Self { name })
    }
}

#[derive(Debug)]
struct PlaylistDelete {
    name: String,
    position: u32,
}

impl MpdRequest for PlaylistDelete {
    const COMMAND: &'static str = "playlistdelete";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let position = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { name, position })
    }
}

#[derive(Debug)]
struct PlaylistLength {
    name: String,
}

impl MpdRequest for PlaylistLength {
    const COMMAND: &'static str = "playlistlength";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        args.expect_empty()?;
        Ok(Self { name })
    }
}

#[derive(Debug)]
struct PlaylistMove {
    name: String,
    range: Range,
    to: u32,
}

impl MpdRequest for PlaylistMove {
    const COMMAND: &'static str = "playlistmove";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let range = args.consume_parse()?;
        let to = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { name, range, to })
    }
}

#[derive(Debug)]
struct Rename {
    name: String,
    new_name: String,
}

impl MpdRequest for Rename {
    const COMMAND: &'static str = "rename";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let new_name = args.consume()?;
        Ok(Self { name, new_name })
    }
}

#[derive(Debug)]
struct Rm {
    name: String,
}

impl MpdRequest for Rm {
    const COMMAND: &'static str = "rm";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        args.expect_empty()?;
        Ok(Self { name })
    }
}

#[derive(Debug)]
struct Save {
    name: String,
    mode: Option<PlaylistSaveMode>,
}

impl MpdRequest for Save {
    const COMMAND: &'static str = "save";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let name = args.consume()?;
        let mode = args.consume_optional_parse()?;
        args.expect_empty()?;
        Ok(Self { name, mode })
    }
}

// database commands

#[derive(Debug)]
struct AlbumArt {
    uri: String,
    offset: u32,
}

impl MpdRequest for AlbumArt {
    const COMMAND: &'static str = "albumart";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        let offset = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { uri, offset })
    }
}

#[derive(Debug)]
struct Count {
    // TODO
}

impl MpdRequest for Count {
    const COMMAND: &'static str = "count";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct GetFingerPrint {
    uri: String,
}

impl MpdRequest for GetFingerPrint {
    const COMMAND: &'static str = "getfingerprint";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct Find {
    // TODO
}

impl MpdRequest for Find {
    const COMMAND: &'static str = "find";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct FindAdd {
    // TODO
}

impl MpdRequest for FindAdd {
    const COMMAND: &'static str = "findadd";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct List {
    // TODO
}

impl MpdRequest for List {
    const COMMAND: &'static str = "list";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct ListAll {
    uri: Option<String>,
}

impl MpdRequest for ListAll {
    const COMMAND: &'static str = "listall";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume_optional()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct ListAllInfo {
    uri: Option<String>,
}

impl MpdRequest for ListAllInfo {
    const COMMAND: &'static str = "listallinfo";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume_optional()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct ListFiles {
    uri: String,
}

impl MpdRequest for ListFiles {
    const COMMAND: &'static str = "listfiles";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct LsInfo {
    uri: String,
}

impl MpdRequest for LsInfo {
    const COMMAND: &'static str = "lsinfo";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct ReadComments {
    uri: String,
}

impl MpdRequest for ReadComments {
    const COMMAND: &'static str = "readcomments";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct ReadPicture {
    uri: String,
    offset: u32,
}

impl MpdRequest for ReadPicture {
    const COMMAND: &'static str = "readpicture";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume()?;
        let offset = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { uri, offset })
    }
}

#[derive(Debug)]
struct Search {
    // TODO
}

impl MpdRequest for Search {
    const COMMAND: &'static str = "search";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct SearchAdd {
    // TODO
}

impl MpdRequest for SearchAdd {
    const COMMAND: &'static str = "searchadd";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct SearchAddPl {
    // TODO
}

impl MpdRequest for SearchAddPl {
    const COMMAND: &'static str = "searchaddpl";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct SearchCount {
    // TODO
}

impl MpdRequest for SearchCount {
    const COMMAND: &'static str = "searchcount";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        Err(Error::todo())
    }
}

#[derive(Debug)]
struct Update {
    uri: Option<String>,
}

impl MpdRequest for Update {
    const COMMAND: &'static str = "update";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume_optional()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

#[derive(Debug)]
struct Rescan {
    uri: Option<String>,
}

impl MpdRequest for Rescan {
    const COMMAND: &'static str = "rescan";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let uri = args.consume_optional()?;
        args.expect_empty()?;
        Ok(Self { uri })
    }
}

// connection commands

#[derive(Debug)]
struct Close;

impl MpdRequest for Close {
    const COMMAND: &'static str = "close";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Kill;

impl MpdRequest for Kill {
    const COMMAND: &'static str = "kill";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct Password {
    password: String,
}

impl MpdRequest for Password {
    const COMMAND: &'static str = "password";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let password = args.consume()?;
        args.expect_empty()?;
        Ok(Self { password })
    }
}

#[derive(Debug)]
struct Ping;

impl MpdRequest for Ping {
    const COMMAND: &'static str = "ping";

    fn parse_arguments(args: Arguments) -> Result<Self> {
        args.expect_empty()?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct BinaryLimit {
    limit: u32,
}

impl MpdRequest for BinaryLimit {
    const COMMAND: &'static str = "binarylimit";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let limit = args.consume_parse()?;
        args.expect_empty()?;
        Ok(Self { limit })
    }
}

#[derive(Debug, Clone)]
enum TagTypesCommand {
    List,
    Clear,
    EnableAll,
    Enable(Vec<Tag>),
    Disable(Vec<Tag>),
}

#[derive(Debug)]
struct TagTypes {
    command: TagTypesCommand,
}

impl MpdRequest for TagTypes {
    const COMMAND: &'static str = "tagtypes";

    fn parse_arguments(mut args: Arguments) -> Result<Self> {
        let command = match args.consume()?.as_str() {
            "list" => TagTypesCommand::List,
            "clear" => TagTypesCommand::Clear,
            "enable" => {
                TagTypesCommand::Enable(args.args.iter().map(|s| s.parse().unwrap()).collect())
            }
            "disable" => {
                TagTypesCommand::Disable(args.args.iter().map(|s| s.parse().unwrap()).collect())
            }
            "all" => TagTypesCommand::EnableAll,
            _ => return Err(Error::todo()),
        };
        args.expect_empty()?;
        Ok(Self { command })
    }
}

macro_rules! impl_enum_command {
    ($($v:ident),*) => {
        #[derive(Debug)]
        enum Command {
            $($v($v)),*
        }

        $(
            impl From<$v> for Command {
                fn from(v: $v) -> Self {
                    Self::$v(v)
                }
            }
        )*

        impl Command {
            fn parse_line(line: &str) -> Result<Self> {
                let (cmd, remain) = match line.split_once(' ') {
                    Some((cmd, remain)) => (cmd, remain),
                    None => (line, ""),
                };
                let args = parse_arguments(remain);
                tracing::debug!("parsing command: {} with {:?}", cmd, args);

                match cmd {
                    $(
                        $v::COMMAND => $v::parse_arguments(args).map(Into::into),
                    )*
                    _ => Err(Error::todo()),
                }
            }
        }
    };
}
impl_enum_command!(
    CommandListBegin,
    CommandListOkBegin,
    CommandListEnd,
    // status
    ClearError,
    CurrentSong,
    Idle,
    Status,
    Stats,
    // playback
    Consume,
    Crossfade,
    MixRampDb,
    MixRampDelay,
    Random,
    Repeat,
    SetVol,
    GetVol,
    Single,
    ReplayGain,
    ReplayGainStatus,
    Volume,
    // control
    Next,
    Pause,
    Play,
    PlayId,
    Previous,
    Seek,
    SeekId,
    SeekCur,
    Stop,
    // queue
    Add,
    AddId,
    Clear,
    Delete,
    DeleteId,
    Move,
    MoveId,
    Playlist,
    PlaylistFind,
    PlaylistId,
    PlaylistInfo,
    PlaylistSearch,
    PlChanges,
    PlChangesPosId,
    Prio,
    PrioId,
    RangeId,
    Shuffle,
    Swap,
    SwapId,
    AddTagId,
    ClearTagId,
    // playlist
    ListPlaylist,
    ListPlaylistInfo,
    ListPlaylists,
    Load,
    PlaylistAdd,
    PlaylistClear,
    PlaylistDelete,
    PlaylistLength,
    PlaylistMove,
    Rename,
    Rm,
    Save,
    // database
    AlbumArt,
    Count,
    GetFingerPrint,
    Find,
    FindAdd,
    List,
    ListAll,
    ListAllInfo,
    ListFiles,
    LsInfo,
    ReadComments,
    ReadPicture,
    Search,
    SearchAdd,
    SearchAddPl,
    SearchCount,
    Update,
    Rescan,
    // mounts
    // sticker
    // connection
    Close,
    Kill,
    Password,
    Ping,
    BinaryLimit,
    TagTypes // partition
             // audio
             // reflection
             // client
);

#[async_trait]
trait Server: Send + Sync + 'static {
    // command list
    async fn on_command_list_ok_begin(&self) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_command_list_end(&self) -> Result<()> {
        Err(Error::unsupported())
    }

    // query
    async fn on_currentsong(&self, responder: &mut Responder) -> Result<()> {
        // file: Avenged Sevenfold/Nightmare/8 - Victim.mp3
        // Last-Modified: 2024-03-02T10:28:44Z
        // Format: 44100:24:2
        // Artist: Avenged Sevenfold
        // AlbumArtist: Avenged Sevenfold
        // Title: Victim
        // Album: Nightmare
        // Track: 8
        // Genre: alternative metal
        // Disc: 1
        // Time: 450
        // duration: 449.750
        // Pos: 0
        // Id: 1
        Err(Error::unsupported())
    }
    async fn on_status(&self, responder: &mut Responder) -> Result<()> {
        // volume: 100
        // repeat: 0
        // random: 0
        // single: 0
        // consume: 0
        // partition: default
        // playlist: 2
        // playlistlength: 1
        // mixrampdb: 0
        // state: pause
        // song: 0
        // songid: 1
        // time: 7:450
        // elapsed: 6.875
        // bitrate: 320
        // duration: 449.750
        // audio: 44100:24:2
        Err(Error::unsupported())
    }
    // playback
    async fn on_consume(&self, consume: Consume) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_crossfade(&self, crossfade: Crossfade) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_mixrampdb(&self, mixrampdb: MixRampDb) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_mixrampdelay(&self, mixrampdelay: MixRampDelay) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_random(&self, random: Random) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_repeat(&self, repeat: Repeat) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_setvol(&self, setvol: SetVol) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_getvol(&self, getvol: GetVol) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_single(&self, single: Single) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_replay_gain(&self, replay_gain: ReplayGain) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_replay_gain_status(&self, replay_gain_status: ReplayGainStatus) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_volume(&self, volume: Volume) -> Result<()> {
        Err(Error::unsupported())
    }
    // control
    async fn on_next(&self, next: Next) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_pause(&self, pause: Pause) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_play(&self, play: Play) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_play_id(&self, play_id: PlayId) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_previous(&self, previous: Previous) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_seek(&self, seek: Seek) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_seek_id(&self, seek_id: SeekId) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_seek_cur(&self, seek_cur: SeekCur) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_stop(&self, stop: Stop) -> Result<()> {
        Err(Error::unsupported())
    }
    // queue
    async fn on_add(&self, add: Add) -> Result<()> {
        Err(Error::unsupported())
    }
    // playlist
    // database
    async fn on_albumart(&self, album_art: AlbumArt) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_count(&self, count: Count) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_getfingerprint(&self, get_fingerprint: GetFingerPrint) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_find(&self, find: Find) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_find_add(&self, find_add: FindAdd) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_list(&self, list: List) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_list_all(&self, list_all: ListAll) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_list_all_info(&self, list_all_info: ListAllInfo) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_list_files(&self, list_files: ListFiles) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_ls_info(&self, ls_info: LsInfo) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_read_comments(&self, read_comments: ReadComments) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_read_picture(&self, read_picture: ReadPicture) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_search(&self, search: Search) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_search_add(&self, search_add: SearchAdd) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_search_add_pl(&self, search_add_pl: SearchAddPl) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_search_count(&self, search_count: SearchCount) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_update(&self, update: Update) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_rescan(&self, rescan: Rescan) -> Result<()> {
        Err(Error::unsupported())
    }
    // mounts
    // sticker
    // connection
    async fn on_close(&self, close: Close) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_kill(&self, kill: Kill) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_password(&self, password: Password) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_ping(&self, ping: Ping) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_binary_limit(&self, binary_limit: BinaryLimit) -> Result<()> {
        Err(Error::unsupported())
    }
    async fn on_tag_types(&self, tag_types: TagTypes) -> Result<()> {
        Err(Error::unsupported())
    }
    // partition
    // audio
    // reflection
    // client
}

#[derive(Debug, Default)]
struct State {
    artists: HashMap<String, Artist>,
    albums: HashMap<String, Album>,
    tracks: HashMap<String, Track>,
}

#[derive(Debug, Clone)]
struct SonarMpdServer {
    client: sonar_grpc::Client,
    state: Arc<Mutex<State>>,
}

impl SonarMpdServer {
    pub async fn new(client: sonar_grpc::Client) -> Result<Self> {
        let client = Self {
            client,
            state: Default::default(),
        };
        client.update_state().await?;
        Ok(client)
    }

    async fn update_state(&self) -> Result<()> {
        let mut client = self.client.clone();
        let artists = sonar_grpc::ext::artist_list_all(&mut client).await.unwrap();
        let albums = sonar_grpc::ext::album_list_all(&mut client).await.unwrap();
        let tracks = sonar_grpc::ext::track_list_all(&mut client).await.unwrap();

        let mut state = self.state.lock().unwrap();
        for artist in artists {
            state.artists.insert(artist.id.clone(), artist);
        }
        for album in albums {
            state.albums.insert(album.id.clone(), album);
        }
        for track in tracks {
            state.tracks.insert(track.id.clone(), track);
        }
        Ok(())
    }
}

#[async_trait]
impl Server for SonarMpdServer {
    async fn on_currentsong(&self, responder: &mut Responder) -> Result<()> {
        responder.key_value("file", "test.mp3")?;
        responder.key_value("artist", "artist")?;
        responder.key_value("album", "album")?;
        responder.key_value("title", "title")?;
        responder.key_value("name", "name")?;
        responder.key_value("track", 1)?;
        Ok(())
    }
    async fn on_status(&self, responder: &mut Responder) -> Result<()> {
        responder.key_value("volume", 100)?;
        responder.key_value("repeat", 0)?;
        responder.key_value("random", 0)?;
        responder.key_value("single", 0)?;
        responder.key_value("consume", 0)?;
        responder.key_value("playlist", 0)?;
        responder.key_value("playlistlength", 0)?;
        responder.key_value("state", "play")?;
        responder.key_value("elapsed", "50")?;
        responder.key_value("duration", "100")?;
        Ok(())
    }

    async fn on_add(&self, add: Add) -> Result<()> {
        tracing::info!("add: {:?}", add);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let endpoint = "http://localhost:3000";
    let client = sonar_grpc::client(endpoint).await?;
    let server = SonarMpdServer::new(client).await?;

    serve(server).await?;

    Ok(())
}

async fn serve(server: impl Server) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], MPD_DEFAULT_PORT + 1));
    tracing::info!("binding to {}", addr);
    let listener = TcpListener::bind(addr).await?;
    let server = Arc::new(server) as Arc<dyn Server>;
    loop {
        let server = server.clone();
        let (stream, remote) = listener.accept().await?;
        tracing::info!("accepted connection from {}", remote);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, server).await {
                tracing::error!("connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, server: Arc<dyn Server>) -> Result<()> {
    let mut stream = BufStream::new(stream);
    let mut buffer = Vec::new();
    let mut response = Responder::default();
    let mut queue = Vec::new();
    let mut queueing = false;
    let mut queueing_ok = false;

    stream.write_all(b"OK MPD 0.23.5\n").await?;
    stream.flush().await?;
    loop {
        buffer.clear();
        let n = stream.read_until(b'\n', &mut buffer).await?;
        if n == 0 {
            break;
        }

        let line = std::str::from_utf8(&buffer)
            //.map_err(|_| Error::new_message(ErrorKind::Invalid, "client sent invalid UTF-8"))?
            .map_err(|_| Error::todo())?
            .trim();
        tracing::debug!("received line: {}", line);

        let command = Command::parse_line(line)?;
        tracing::debug!("parsed command: {:?}", command);

        match command {
            Command::CommandListBegin(_) => {
                if queueing {
                    return Err(Error::todo());
                }
                queueing = true;
            }
            Command::CommandListOkBegin(_) => {
                if queueing {
                    return Err(Error::todo());
                }
                queueing = true;
                queueing_ok = true;
            }
            Command::CommandListEnd(_) => {
                queueing = false;
            }
            _ => queue.push(command),
        }

        if !queueing {
            let queue_len = queue.len();

            for command in queue.drain(..) {
                match command {
                    Command::CommandListBegin(_) => todo!(),
                    Command::CommandListOkBegin(_) => todo!(),
                    Command::CommandListEnd(_) => todo!(),
                    Command::Status(_) => server.on_status(&mut response).await?,
                    Command::CurrentSong(_) => server.on_currentsong(&mut response).await?,
                    Command::Consume(v) => server.on_consume(v).await?,
                    Command::Crossfade(v) => server.on_crossfade(v).await?,
                    Command::MixRampDb(v) => server.on_mixrampdb(v).await?,
                    Command::MixRampDelay(v) => server.on_mixrampdelay(v).await?,
                    Command::Random(v) => server.on_random(v).await?,
                    Command::Repeat(v) => server.on_repeat(v).await?,
                    Command::SetVol(v) => server.on_setvol(v).await?,
                    Command::GetVol(v) => server.on_getvol(v).await?,
                    Command::Single(v) => server.on_single(v).await?,
                    Command::ReplayGain(v) => server.on_replay_gain(v).await?,
                    Command::ReplayGainStatus(v) => server.on_replay_gain_status(v).await?,
                    Command::Volume(v) => server.on_volume(v).await?,
                    Command::Next(v) => server.on_next(v).await?,
                    Command::Pause(v) => server.on_pause(v).await?,
                    Command::Play(v) => server.on_play(v).await?,
                    Command::PlayId(v) => server.on_play_id(v).await?,
                    Command::Previous(v) => server.on_previous(v).await?,
                    Command::Seek(v) => server.on_seek(v).await?,
                    Command::SeekId(v) => server.on_seek_id(v).await?,
                    Command::SeekCur(v) => server.on_seek_cur(v).await?,
                    Command::Stop(v) => server.on_stop(v).await?,
                    Command::Add(v) => server.on_add(v).await?,
                    Command::AlbumArt(v) => server.on_albumart(v).await?,
                    Command::Count(v) => server.on_count(v).await?,
                    Command::GetFingerPrint(v) => server.on_getfingerprint(v).await?,
                    Command::Find(v) => server.on_find(v).await?,
                    Command::FindAdd(v) => server.on_find_add(v).await?,
                    Command::List(v) => server.on_list(v).await?,
                    Command::ListAll(v) => server.on_list_all(v).await?,
                    Command::ListAllInfo(v) => server.on_list_all_info(v).await?,
                    Command::ListFiles(v) => server.on_list_files(v).await?,
                    Command::LsInfo(v) => server.on_ls_info(v).await?,
                    Command::ReadComments(v) => server.on_read_comments(v).await?,
                    Command::ReadPicture(v) => server.on_read_picture(v).await?,
                    Command::Search(v) => server.on_search(v).await?,
                    Command::SearchAdd(v) => server.on_search_add(v).await?,
                    Command::SearchAddPl(v) => server.on_search_add_pl(v).await?,
                    Command::SearchCount(v) => server.on_search_count(v).await?,
                    Command::Update(v) => server.on_update(v).await?,
                    Command::Rescan(v) => server.on_rescan(v).await?,
                    Command::AddId(_) => todo!(),
                    Command::Clear(_) => todo!(),
                    Command::Delete(_) => todo!(),
                    Command::DeleteId(_) => todo!(),
                    Command::Move(_) => todo!(),
                    Command::MoveId(_) => todo!(),
                    Command::Playlist(_) => todo!(),
                    Command::PlaylistFind(_) => todo!(),
                    Command::PlaylistId(_) => todo!(),
                    Command::PlaylistInfo(_) => todo!(),
                    Command::PlaylistSearch(_) => todo!(),
                    Command::PlChanges(_) => todo!(),
                    Command::PlChangesPosId(_) => todo!(),
                    Command::Prio(_) => todo!(),
                    Command::PrioId(_) => todo!(),
                    Command::RangeId(_) => todo!(),
                    Command::Shuffle(_) => todo!(),
                    Command::Swap(_) => todo!(),
                    Command::SwapId(_) => todo!(),
                    Command::AddTagId(_) => todo!(),
                    Command::ClearTagId(_) => todo!(),
                    Command::ListPlaylist(_) => todo!(),
                    Command::ListPlaylistInfo(_) => todo!(),
                    Command::ListPlaylists(_) => todo!(),
                    Command::Load(_) => todo!(),
                    Command::PlaylistAdd(_) => todo!(),
                    Command::PlaylistClear(_) => todo!(),
                    Command::PlaylistDelete(_) => todo!(),
                    Command::PlaylistLength(_) => todo!(),
                    Command::PlaylistMove(_) => todo!(),
                    Command::Rename(_) => todo!(),
                    Command::Rm(_) => todo!(),
                    Command::Save(_) => todo!(),
                    Command::Close(_) => todo!(),
                    Command::Kill(_) => todo!(),
                    Command::Password(_) => todo!(),
                    Command::Ping(_) => todo!(),
                    Command::BinaryLimit(_) => todo!(),
                    Command::TagTypes(_) => todo!(),
                    Command::ClearError(_) => todo!(),
                    Command::Idle(_) => todo!(),
                    Command::Stats(_) => todo!(),
                }

                if queueing_ok {
                    response.list_ok()?;
                }
            }

            queueing_ok = false;
            response.ok()?;

            stream.write_all(response.as_bytes()).await?;
            stream.flush().await?;
            response.clear();
        }
    }
    Ok(())
}

fn parse_arguments(line: &str) -> Arguments {
    let mut args = VecDeque::new();
    let mut line = line.trim();

    while !line.is_empty() {
        tracing::debug!("line = [{}]", line);
        if line.starts_with('"') {
            line = &line[1..];
            let end = line.find('"').unwrap_or(line.len());
            let arg = line[..end].to_string();
            line = &line[(end + 1).min(line.len())..];
            line = line.trim();
            tracing::debug!("pushing arg: '{}'", arg);
            args.push_back(arg);
        } else {
            let (arg, rem) = match line.split_once(char::is_whitespace) {
                Some((arg, rem)) => (arg, rem.trim()),
                None => (line, ""),
            };
            tracing::debug!("pushing arg2: '{}'", arg);
            args.push_back(arg.to_string());
            line = rem;
        }
    }

    Arguments { args }
}
