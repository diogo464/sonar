#![feature(str_split_whitespace_remainder)]
use std::{net::SocketAddr, str::FromStr};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};

const MPD_DEFAULT_PORT: u16 = 6600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Invalid,
    Internal,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid => write!(f, "invalid input"),
            Self::Internal => write!(f, "internal error"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error + Send + 'static>>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl Error {
    fn new(
        kind: ErrorKind,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + 'static>>,
    ) -> Self {
        Self {
            kind,
            message,
            source,
        }
    }

    fn new_message(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::new(kind, message.into(), None)
    }

    fn wrap(kind: ErrorKind, source: impl std::error::Error + Send + 'static) -> Self {
        Self::new(kind, Default::default(), Some(Box::new(source)))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::wrap(ErrorKind::Internal, value)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn run() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], MPD_DEFAULT_PORT + 1));
    tracing::info!("binding to {}", addr);
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, remote) = listener.accept().await?;
        tracing::info!("accepted connection from {}", remote);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                tracing::error!("connection error: {}", e);
            }
        });
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// MPD range. The range is not inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: u32,
    end: Option<u32>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepeatMode {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomMode {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingleMode {
    On,
    Off,
    OneShot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumeMode {
    On,
    Off,
    OneShot,
}

fn write_key_value(buf: &mut Vec<u8>, key: &str, value: impl std::fmt::Display) {
    use std::io::Write;
    buf.extend_from_slice(key.as_bytes());
    buf.extend_from_slice(b": ");
    write!(buf, "{}", value).unwrap();
    buf.extend_from_slice(b"\n");
}

fn write_ok(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"OK\n");
}

fn write_list_ok(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"list_OK\n");
}

async fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut stream = BufStream::new(stream);
    let mut buffer = Vec::new();
    let mut response = Vec::new();
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
            .map_err(|_| Error::new_message(ErrorKind::Invalid, "client sent invalid UTF-8"))?
            .trim();
        tracing::debug!("received line: {}", line);

        let command = match parse_command(line) {
            Ok(command) => command,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("failed to parse command '{}': {}", line, e),
                    Some(Box::new(e)),
                ))
            }
        };

        match command {
            Command::CommandListBegin => {
                queueing = true;
                queueing_ok = false;
            }
            Command::CommandListOkBegin => {
                queueing = true;
                queueing_ok = true;
            }
            Command::CommandListEnd => {
                for command in queue.drain(..) {
                    process_command(command, &mut response).await?;
                    if queueing_ok {
                        write_list_ok(&mut response);
                    }
                }
                write_ok(&mut response);
            }
            _ => {
                if queueing {
                    queue.push(command);
                } else {
                    process_command(command, &mut response).await?;
                    write_ok(&mut response);
                }
            }
        }

        stream.write_all(&response).await?;
        stream.flush().await?;
        response.clear();
    }
    Ok(())
}

async fn process_command(command: Command, response: &mut Vec<u8>) -> Result<()> {
    match command {
        Command::CommandListBegin => {}
        Command::CommandListOkBegin => {}
        Command::CommandListEnd => {}
        Command::ListAll(_) => {}
        Command::Status(_) => {
            write_key_value(response, "volume", 100);
            write_key_value(response, "repeat", 0);
            write_key_value(response, "random", 0);
            write_key_value(response, "single", 0);
            write_key_value(response, "consume", 0);
            write_key_value(response, "playlist", 0);
            write_key_value(response, "playlistlength", 0);
            write_key_value(response, "state", "stop");
        }
        Command::PlChanges(_) => {}
        Command::Outputs(_) => {
            write_key_value(response, "outputid", 0);
            write_key_value(response, "outputname", "dummy");
            write_key_value(response, "outputenabled", 1);
        }
        Command::Decoders(_) => {}
        Command::Idle(_) => {}
        Command::NoIdle(_) => {}
        Command::LsInfo(_) => {
            write_key_value(response, "file", "music.mp3");
            write_key_value(response, "file", "music2.mp3");
            write_key_value(response, "file", "music3.mp3");
        }
        Command::AddId(cmd) => {
            tracing::info!("adding file: {}", cmd.uri);
        }
        Command::List(cmd) => {
            if cmd.list_tag == Tag::Artist {
                write_key_value(response, "Artist", "Artist1");
                write_key_value(response, "Artist", "Artist2");
                write_key_value(response, "Artist", "Artist3");
            } else if cmd.list_tag == Tag::Album {
                write_key_value(response, "Album", "Album1");
                write_key_value(response, "Album", "Album2");
                write_key_value(response, "Album", "Album3");
            }
        }
        Command::Find(_) => {
            write_key_value(response, "file", "music.mp3");
        }
        Command::Search(_) => {}
        Command::ListPlaylists(_) => {
            write_key_value(response, "playlist", "playlist1");
        }
        Command::ListPlaylistInfo(_) => {
            write_key_value(response, "file", "music.mp3");
            write_key_value(response, "file", "music2.mp3");
            write_key_value(response, "file", "music3.mp3");
        }
        Command::ListAllInfo(_) => {}
        Command::Volume(_) => {}
        Command::Random(_) => {}
        Command::Repeat(_) => {}
        Command::Consume(_) => {}
        Command::Single(_) => {}
        Command::CurrentSong(_) => {
            write_key_value(response, "file", "music.mp3");
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Command {
    CommandListBegin,
    CommandListOkBegin,
    CommandListEnd,
    ListAll(ListAllCommand),
    Status(StatusCommand),
    PlChanges(PlChangesCommand),
    Outputs(OutputsCommand),
    Decoders(DecodersCommand),
    Idle(IdleCommand),
    NoIdle(NoIdleCommand),
    LsInfo(LsInfoCommand),
    AddId(AddIdCommand),
    List(ListCommand),
    Find(FindCommand),
    Search(SearchCommand),
    ListPlaylists(ListPlaylistsCommand),
    ListPlaylistInfo(ListPlaylistInfoCommand),
    ListAllInfo(ListAllInfoCommand),
    Volume(VolumeCommand),
    Random(RandomCommand),
    Repeat(RepeatCommand),
    Consume(ConsumeCommand),
    Single(SingleCommand),
    CurrentSong(CurrentSongCommand),
}

#[derive(Debug, Clone)]
struct ListAllCommand {
    uri: Option<String>,
}

fn parse_command_listall(input: &str) -> Result<ListAllCommand> {
    let (uri, remainder) = consume_argument_opt(input)?;
    consume_empty(remainder)?;
    Ok(ListAllCommand { uri })
}

#[derive(Debug, Clone)]
struct StatusCommand;

#[derive(Debug, Clone)]
struct PlChangesCommand {
    version: u32,
    range: Option<Range>,
}

fn parse_command_plchanges(input: &str) -> Result<PlChangesCommand> {
    let (version, remaider) = consume_u32(input)?;
    let (range, remainder) = consume_range_opt(remaider)?;
    consume_empty(remainder)?;
    Ok(PlChangesCommand { version, range })
}

#[derive(Debug, Clone)]
struct OutputsCommand;

#[derive(Debug, Clone)]
struct DecodersCommand;

#[derive(Debug, Clone)]
struct IdleCommand {
    subsystems: Vec<Subsystem>,
}

fn parse_command_idle(input: &str) -> Result<IdleCommand> {
    let mut input = input.trim_start();
    let mut subsystems = Vec::new();
    while !input.is_empty() {
        let (subsystem, remainder) = consume_subsystem(input)?;
        input = remainder;
        subsystems.push(subsystem);
    }
    Ok(IdleCommand { subsystems })
}

#[derive(Debug, Clone)]
struct NoIdleCommand;

#[derive(Debug, Clone)]
struct LsInfoCommand {
    uri: Option<String>,
}

fn parse_command_lsinfo(input: &str) -> Result<LsInfoCommand> {
    let (uri, remainder) = consume_argument_opt(input)?;
    consume_empty(remainder)?;
    Ok(LsInfoCommand { uri })
}

#[derive(Debug, Clone)]
struct AddIdCommand {
    uri: String,
    position: Option<u32>,
}

fn parse_command_addid(input: &str) -> Result<AddIdCommand> {
    let (uri, remainder) = consume_argument_opt(input)?;
    let (position, remainder) = consume_u32_opt(remainder)?;
    consume_empty(remainder)?;
    Ok(AddIdCommand {
        uri: uri.unwrap(),
        position,
    })
}

#[derive(Debug, Clone)]
struct ListCommand {
    list_tag: Tag,
    // TODO: this should be an expression
    filter: Option<String>,
    group_tag: Vec<Tag>,
}

fn parse_list_command(input: &str) -> Result<ListCommand> {
    let (list_tag, remainder) = consume_tag(input)?;
    let remainder = remainder.trim_start();
    if !remainder.starts_with("group") {}

    let mut group_tag = Vec::new();
    let mut remainder = remainder.trim_start();
    while remainder.starts_with("group") {
        remainder = &remainder[5..];
        let (tag, rem) = consume_tag(remainder)?;
        group_tag.push(tag);
        remainder = rem.trim_start();
    }

    Ok(ListCommand {
        list_tag,
        filter: None,
        group_tag,
    })
}

#[derive(Debug, Clone)]
struct FindCommand {
    // TODO: this should be an expression
    filter: String,
    // TODO: add the rest of the fields
}

fn parse_command_find(input: &str) -> Result<FindCommand> {
    let (filter, _remainder) = consume_argument_opt(input)?;
    Ok(FindCommand {
        filter: filter.unwrap(),
    })
}

#[derive(Debug, Clone)]
struct SearchCommand {
    // TODO: this should be an expression
    filter: String, // TODO: add the rest of the fields
}

fn parse_command_search(input: &str) -> Result<SearchCommand> {
    let (filter, _remainder) = consume_argument_opt(input)?;
    Ok(SearchCommand {
        filter: filter.unwrap(),
    })
}

#[derive(Debug, Clone)]
struct ListPlaylistsCommand;

#[derive(Debug, Clone)]
struct ListPlaylistInfoCommand {
    name: String,
    range: Option<Range>,
}

fn parse_command_listplaylistinfo(input: &str) -> Result<ListPlaylistInfoCommand> {
    let (name, remainder) = consume_argument(input)?;
    let (range, remainder) = consume_range_opt(remainder)?;
    consume_empty(remainder)?;
    Ok(ListPlaylistInfoCommand { name, range })
}

#[derive(Debug, Clone)]
struct ListAllInfoCommand {
    uri: Option<String>,
}

fn parse_command_listallinfo(input: &str) -> Result<ListAllInfoCommand> {
    let (uri, remainder) = consume_argument_opt(input)?;
    consume_empty(remainder)?;
    Ok(ListAllInfoCommand { uri })
}

#[derive(Debug, Clone)]
struct VolumeCommand {
    change: i32,
}

fn parse_command_volume(input: &str) -> Result<VolumeCommand> {
    let (change, remainder) = consume_i32(input)?;
    consume_empty(remainder)?;
    Ok(VolumeCommand { change })
}

#[derive(Debug, Clone)]
struct RandomCommand {
    mode: RandomMode,
}

fn parse_command_random(input: &str) -> Result<RandomCommand> {
    let (mode, remainder) = consume_random_mode(input)?;
    consume_empty(remainder)?;
    Ok(RandomCommand { mode })
}

#[derive(Debug, Clone)]
struct RepeatCommand {
    mode: RepeatMode,
}

fn parse_command_repeat(input: &str) -> Result<RepeatCommand> {
    let (mode, remainder) = consume_repeat_mode(input)?;
    consume_empty(remainder)?;
    Ok(RepeatCommand { mode })
}

#[derive(Debug, Clone)]
struct ConsumeCommand {
    mode: ConsumeMode,
}

fn parse_command_consume(input: &str) -> Result<ConsumeCommand> {
    let (mode, remainder) = consume_consume_mode(input)?;
    consume_empty(remainder)?;
    Ok(ConsumeCommand { mode })
}

#[derive(Debug, Clone)]
struct SingleCommand {
    mode: SingleMode,
}

fn parse_command_single(input: &str) -> Result<SingleCommand> {
    let (mode, remainder) = consume_single_mode(input)?;
    consume_empty(remainder)?;
    Ok(SingleCommand { mode })
}

#[derive(Debug, Clone)]
struct CurrentSongCommand;

fn parse_command(line: &str) -> Result<Command> {
    let (cmd, remainder) = consume_ident(line)?;
    match cmd {
        "command_list_begin" => parse_command_empty(remainder, Command::CommandListBegin),
        "command_list_ok_begin" => parse_command_empty(remainder, Command::CommandListOkBegin),
        "command_list_end" => parse_command_empty(remainder, Command::CommandListEnd),
        "listall" => parse_command_listall(remainder).map(Command::ListAll),
        "status" => parse_command_empty(remainder, StatusCommand).map(Command::Status),
        "plchanges" => parse_command_plchanges(remainder).map(Command::PlChanges),
        "outputs" => parse_command_empty(remainder, OutputsCommand).map(Command::Outputs),
        "decoders" => parse_command_empty(remainder, DecodersCommand).map(Command::Decoders),
        "idle" => parse_command_idle(remainder).map(Command::Idle),
        "noidle" => parse_command_empty(remainder, NoIdleCommand).map(Command::NoIdle),
        "lsinfo" => parse_command_lsinfo(remainder).map(Command::LsInfo),
        "addid" => parse_command_addid(remainder).map(Command::AddId),
        "list" => parse_list_command(remainder).map(Command::List),
        "find" => parse_command_find(remainder).map(Command::Find),
        "search" => parse_command_search(remainder).map(Command::Search),
        "listplaylists" => {
            parse_command_empty(remainder, ListPlaylistsCommand).map(Command::ListPlaylists)
        }
        "listplaylistinfo" => {
            parse_command_listplaylistinfo(remainder).map(Command::ListPlaylistInfo)
        }
        "listallinfo" => parse_command_listallinfo(remainder).map(Command::ListAllInfo),
        "volume" => parse_command_volume(remainder).map(Command::Volume),
        "random" => parse_command_random(remainder).map(|cmd| Command::Status(StatusCommand)),
        "repeat" => parse_command_repeat(remainder).map(|cmd| Command::Status(StatusCommand)),
        "consume" => parse_command_consume(remainder).map(|cmd| Command::Status(StatusCommand)),
        "single" => parse_command_single(remainder).map(|cmd| Command::Status(StatusCommand)),
        "currentsong" => {
            parse_command_empty(remainder, CurrentSongCommand).map(Command::CurrentSong)
        }
        _ => Err(Error::new_message(
            ErrorKind::Invalid,
            format!("unknown command: {}", cmd),
        )),
    }
}

fn parse_command_empty<T>(input: &str, cmd: T) -> Result<T> {
    consume_empty(input)?;
    Ok(cmd)
}

fn consume_ident(input: &str) -> Result<(&str, &str)> {
    let input = input.trim_start();
    let (ident, remainder) = consume_until_whitespace(input)?;
    if ident.is_empty() {
        return Err(Error::new_message(
            ErrorKind::Invalid,
            "expected identifier",
        ));
    }
    return Ok((ident, remainder));
}

fn consume_repeat_mode(input: &str) -> Result<(RepeatMode, &str)> {
    let (mode, remainder) = consume_argument(input)?;
    let mode = match mode.as_str() {
        "1" => RepeatMode::On,
        "0" => RepeatMode::Off,
        _ => {
            return Err(Error::new_message(
                ErrorKind::Invalid,
                "invalid repeat mode",
            ))
        }
    };
    Ok((mode, remainder))
}

fn consume_random_mode(input: &str) -> Result<(RandomMode, &str)> {
    let (mode, remainder) = consume_argument(input)?;
    let mode = match mode.as_str() {
        "1" => RandomMode::On,
        "0" => RandomMode::Off,
        _ => {
            return Err(Error::new_message(
                ErrorKind::Invalid,
                "invalid random mode",
            ))
        }
    };
    Ok((mode, remainder))
}

fn consume_single_mode(input: &str) -> Result<(SingleMode, &str)> {
    let (mode, remainder) = consume_argument(input)?;
    let mode = match mode.as_str() {
        "1" => SingleMode::On,
        "0" => SingleMode::Off,
        "oneshot" => SingleMode::OneShot,
        _ => {
            return Err(Error::new_message(
                ErrorKind::Invalid,
                "invalid single mode",
            ))
        }
    };
    Ok((mode, remainder))
}

fn consume_consume_mode(input: &str) -> Result<(ConsumeMode, &str)> {
    let (mode, remainder) = consume_argument(input)?;
    let mode = match mode.as_str() {
        "1" => ConsumeMode::On,
        "0" => ConsumeMode::Off,
        "oneshot" => ConsumeMode::OneShot,
        _ => {
            return Err(Error::new_message(
                ErrorKind::Invalid,
                "invalid consume mode",
            ))
        }
    };
    Ok((mode, remainder))
}

fn consume_tag(input: &str) -> Result<(Tag, &str)> {
    let (ident, remainder) = consume_ident(input)?;
    let tag = ident
        .parse()
        .map_err(|_| Error::new_message(ErrorKind::Invalid, "invalid tag"))?;
    Ok((tag, remainder))
}

fn consume_subsystem(input: &str) -> Result<(Subsystem, &str)> {
    let (ident, remainder) = consume_ident(input)?;
    let subsystem = match ident {
        "audio" => Subsystem::Audio,
        "database" => Subsystem::Database,
        "mixer" => Subsystem::Mixer,
        "options" => Subsystem::Options,
        "playlist" => Subsystem::Playlist,
        "sticker" => Subsystem::Sticker,
        "update" => Subsystem::Update,
        "stored_playlist" => Subsystem::StoredPlaylist,
        "partition" => Subsystem::Partition,
        "sticker_cache" => Subsystem::StickerCache,
        "subscription" => Subsystem::Subscription,
        "message" => Subsystem::Message,
        "neighbor" => Subsystem::Neighbor,
        "output" => Subsystem::Output,
        "reflection" => Subsystem::Reflection,
        "stats" => Subsystem::Stats,
        "status" => Subsystem::Status,
        "mount" => Subsystem::Mount,
        "command" => Subsystem::Command,
        "notifier" => Subsystem::Notifier,
        _ => Subsystem::Unknown(ident.to_string()),
    };
    Ok((subsystem, remainder))
}

fn consume_i32(input: &str) -> Result<(i32, &str)> {
    let input = input.trim_start();
    // it seems like numbers can be quoted.
    // ncmpcpp sends 'plchanges "0"' for example.
    if input.starts_with('"') {
        let input = &input[1..];
        let quote_idx = input
            .find('"')
            .ok_or_else(|| Error::new_message(ErrorKind::Invalid, "expected closing quote"))?;
        let value = &input[..quote_idx];
        let remainder = &input[quote_idx + 1..];
        let value = value
            .parse()
            .map_err(|e| Error::new(ErrorKind::Invalid, format!("invalid i32: {}", e), None))?;
        Ok((value, remainder))
    } else {
        let (value, remainder) = consume_until_whitespace(input)?;
        let value = value
            .parse()
            .map_err(|e| Error::new(ErrorKind::Invalid, format!("invalid u32: {}", e), None))?;
        Ok((value, remainder))
    }
}

fn consume_i32_opt(input: &str) -> Result<(Option<i32>, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return Ok((None, input));
    }

    let (value, remainder) = consume_i32(input)?;
    Ok((Some(value), remainder))
}

fn consume_u32(input: &str) -> Result<(u32, &str)> {
    let (value, remainder) = consume_i32(input)?;
    if value < 0 {
        return Err(Error::new_message(
            ErrorKind::Invalid,
            "expected positive integer",
        ));
    }
    Ok((value as u32, remainder))
}

fn consume_u32_opt(input: &str) -> Result<(Option<u32>, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return Ok((None, input));
    }

    let (value, remainder) = consume_u32(input)?;
    Ok((Some(value), remainder))
}

fn consume_range_opt(input: &str) -> Result<(Option<Range>, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return Ok((None, input));
    }

    let (start, remainder) = consume_u32(input)?;
    if !remainder.starts_with(':') {
        return Err(Error::new_message(
            ErrorKind::Invalid,
            "expected colon after start of range",
        ));
    }

    let remainder = &remainder[1..];
    if remainder.is_empty() || !remainder.as_bytes()[0].is_ascii_digit() {
        return Ok((Some(Range { start, end: None }), remainder));
    }

    let (end, remainder) = consume_u32(remainder)?;
    Ok((
        Some(Range {
            start,
            end: Some(end),
        }),
        remainder,
    ))
}

fn consume_argument(input: &str) -> Result<(String, &str)> {
    let (argument, remainder) = consume_argument_opt(input)?;
    match argument {
        Some(argument) => Ok((argument, remainder)),
        None => Err(Error::new_message(ErrorKind::Invalid, "expected argument")),
    }
}

fn consume_argument_opt(input: &str) -> Result<(Option<String>, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return Ok((None, input));
    }

    if !input.starts_with('"') {
        let (value, remainder) = consume_until_whitespace(input)?;
        return Ok((Some(value.to_string()), remainder));
    }

    let mut input = &input[1..];
    let mut buffer = String::with_capacity(8);
    let mut it = input.chars();
    let mut escaped = false;
    while let Some(c) = it.next() {
        match c {
            '"' if !escaped => {
                break;
            }
            '\\' if !escaped => {
                escaped = true;
            }
            _ => {
                buffer.push(c);
                escaped = false;
            }
        }
    }
    input = it.as_str();
    Ok((Some(buffer), input))
}

fn consume_until_whitespace(input: &str) -> Result<(&str, &str)> {
    let mut it = input.split_ascii_whitespace();
    let value = it.next().unwrap_or_default();
    let remainder = it.remainder().unwrap_or_default();
    Ok((value, remainder))
}

fn consume_empty(input: &str) -> Result<()> {
    let input = input.trim_start();
    if input.is_empty() {
        Ok(())
    } else {
        Err(Error::new_message(
            ErrorKind::Invalid,
            format!("unexpected trailing input: '{}'", input),
        ))
    }
}
