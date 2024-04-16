#![feature(async_closure)]
use clap::Parser;
use opensubsonic::{
    common::Version,
    request::{
        annotation::*, bookmark::*, browsing::*, chat::*, jukebox::*, lists::*, playlists::*,
        podcast::*, radio::*, retrieval::*, scan::*, search::*, sharing::*, system::*, user::*,
        Authentication, Request, SubsonicRequest,
    },
};

#[derive(Debug, Parser)]
struct Args {
    /// The server to connect to
    #[clap(
        long,
        default_value = "http://localhost:3000",
        env = "OPENSUBSONIC_SERVER"
    )]
    server: String,

    /// The client string to use
    #[clap(long, default_value = "opensubsonic-cli", env = "OPENSUBSONIC_CLIENT")]
    client: String,

    /// The username to use
    #[clap(long, env = "OPENSUBSONIC_USERNAME")]
    username: Option<String>,

    /// The password to use
    #[clap(long, env = "OPENSUBSONIC_PASSWORD")]
    password: Option<String>,

    /// The format to use
    #[clap(long, default_value = "json", env = "OPENSUBSONIC_FORMAT")]
    format: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    AddChatMessage(AddChatMessage),
    ChangePassword(ChangePassword),
    CreateBookmark(CreateBookmark),
    CreateInternetRadioStation(CreateInternetRadioStation),
    CreatePlaylist(CreatePlaylist),
    CreatePodcastChannel(CreatePodcastChannel),
    CreateShare(CreateShare),
    CreateUser(CreateUser),
    DeleteBookmark(DeleteBookmark),
    DeleteInternetRadioStation(DeleteInternetRadioStation),
    DeletePlaylist(DeletePlaylist),
    DeletePodcastChannel(DeletePodcastChannel),
    DeletePodcastEpisode(DeletePodcastEpisode),
    DeleteShare(DeleteShare),
    DeleteUser(DeleteUser),
    Download(Download),
    DownloadPodcastEpisode(DownloadPodcastEpisode),
    GetAlbum(GetAlbum),
    GetAlbumInfo(GetAlbumInfo),
    GetAlbumInfo2(GetAlbumInfo2),
    GetAlbumList(GetAlbumList),
    GetAlbumList2(GetAlbumList2),
    GetArtist(GetArtist),
    GetArtistInfo(GetArtistInfo),
    GetArtistInfo2(GetArtistInfo2),
    GetArtists(GetArtists),
    GetAvatar(GetAvatar),
    GetBookmarks(GetBookmarks),
    GetCaptions(GetCaptions),
    GetChatMessages(GetChatMessages),
    GetCoverArt(GetCoverArt),
    GetGenres(GetGenres),
    GetIndexes(GetIndexes),
    GetInternetRadioStations(GetInternetRadioStations),
    GetLicense(GetLicense),
    GetLyrics(GetLyrics),
    GetMusicDirectory(GetMusicDirectory),
    GetMusicFolders(GetMusicFolders),
    GetNewestPodcasts(GetNewestPodcasts),
    GetNowPlaying(GetNowPlaying),
    GetPlaylist(GetPlaylist),
    GetPlaylists(GetPlaylists),
    GetPlayQueue(GetPlayQueue),
    GetPodcasts(GetPodcasts),
    GetRandomSongs(GetRandomSongs),
    GetScanStatus(GetScanStatus),
    GetShares(GetShares),
    GetSimilarSongs(GetSimilarSongs),
    GetSimilarSongs2(GetSimilarSongs2),
    GetSong(GetSong),
    GetSongsByGenre(GetSongsByGenre),
    GetStarred(GetStarred),
    GetStarred2(GetStarred2),
    GetTopSongs(GetTopSongs),
    GetUser(GetUser),
    GetUsers(GetUsers),
    GetVideoInfo(GetVideoInfo),
    GetVideos(GetVideos),
    Hls(Hls),
    JubeboxControl(JukeboxControl),
    Ping(Ping),
    RefreshPodcasts(RefreshPodcasts),
    SavePlayQueue(SavePlayQueue),
    Scrobble(Scrobble),
    Search(Search),
    Search2(Search2),
    Search3(Search3),
    SetRating(SetRating),
    Star(Star),
    StartScan(StartScan),
    Unstar(Unstar),
    UpdateInternetRadioStation(UpdateInternetRadioStation),
    UpdatePlaylist(UpdatePlaylist),
    UpdateShare(UpdateShare),
    UpdateUser(UpdateUser),
}

struct RequestContext {
    server: String,
    version: Version,
    username: String,
    authentication: Authentication,
    client: String,
    format: String,
}

impl RequestContext {
    async fn request<B: SubsonicRequest>(
        self,
        method: &'static str,
        body: B,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = Request {
            username: Some(self.username),
            authentication: self.authentication,
            version: self.version,
            client: self.client,
            format: Some(self.format),
            body,
        };
        let url = format!("{}/rest/{}?{}", self.server, method, request.to_query());
        let response = reqwest::Client::new().get(url).send().await?;
        let response = response.text().await?;
        println!("{}", response);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ctx = RequestContext {
        server: args.server,
        version: Version::V1_16_1,
        username: args.username.unwrap_or_default(),
        authentication: Authentication::Password(args.password.unwrap_or_default()),
        client: args.client,
        format: args.format,
    };

    match args.command {
        Command::AddChatMessage(body) => ctx.request("addChatMessage", body).await?,
        Command::ChangePassword(body) => ctx.request("changePassword", body).await?,
        Command::CreateBookmark(body) => ctx.request("createBookmark", body).await?,
        Command::CreateInternetRadioStation(body) => {
            ctx.request("createInternetRadioStation", body).await?
        }
        Command::CreatePlaylist(body) => ctx.request("createPlaylist", body).await?,
        Command::CreatePodcastChannel(body) => ctx.request("createPodcastChannel", body).await?,
        Command::CreateShare(body) => ctx.request("createShare", body).await?,
        Command::CreateUser(body) => ctx.request("createUser", body).await?,
        Command::DeleteBookmark(body) => ctx.request("deleteBookmark", body).await?,
        Command::DeleteInternetRadioStation(body) => {
            ctx.request("deleteInternetRadioStation", body).await?
        }
        Command::DeletePlaylist(body) => ctx.request("deletePlaylist", body).await?,
        Command::DeletePodcastChannel(body) => ctx.request("deletePodcastChannel", body).await?,
        Command::DeletePodcastEpisode(body) => ctx.request("deletePodcastEpisode", body).await?,
        Command::DeleteShare(body) => ctx.request("deleteShare", body).await?,
        Command::DeleteUser(body) => ctx.request("deleteUser", body).await?,
        Command::Download(body) => ctx.request("download", body).await?,
        Command::DownloadPodcastEpisode(body) => {
            ctx.request("downloadPodcastEpisode", body).await?
        }
        Command::GetAlbum(body) => ctx.request("getAlbum", body).await?,
        Command::GetAlbumInfo(body) => ctx.request("getAlbumInfo", body).await?,
        Command::GetAlbumInfo2(body) => ctx.request("getAlbumInfo2", body).await?,
        Command::GetAlbumList(body) => ctx.request("getAlbumList", body).await?,
        Command::GetAlbumList2(body) => ctx.request("getAlbumList2", body).await?,
        Command::GetArtist(body) => ctx.request("getArtist", body).await?,
        Command::GetArtistInfo(body) => ctx.request("getArtistInfo", body).await?,
        Command::GetArtistInfo2(body) => ctx.request("getArtistInfo2", body).await?,
        Command::GetArtists(body) => ctx.request("getArtists", body).await?,
        Command::GetAvatar(body) => ctx.request("getAvatar", body).await?,
        Command::GetBookmarks(body) => ctx.request("getBookmarks", body).await?,
        Command::GetCaptions(body) => ctx.request("getCaptions", body).await?,
        Command::GetChatMessages(body) => ctx.request("getChatMessages", body).await?,
        Command::GetCoverArt(body) => ctx.request("getCoverArt", body).await?,
        Command::GetGenres(body) => ctx.request("getGenres", body).await?,
        Command::GetIndexes(body) => ctx.request("getIndexes", body).await?,
        Command::GetInternetRadioStations(body) => {
            ctx.request("getInternetRadioStations", body).await?
        }
        Command::GetLicense(body) => ctx.request("getLicense", body).await?,
        Command::GetLyrics(body) => ctx.request("getLyrics", body).await?,
        Command::GetMusicDirectory(body) => ctx.request("getMusicDirectory", body).await?,
        Command::GetMusicFolders(body) => ctx.request("getMusicFolders", body).await?,
        Command::GetNewestPodcasts(body) => ctx.request("getNewestPodcasts", body).await?,
        Command::GetNowPlaying(body) => ctx.request("getNowPlaying", body).await?,
        Command::GetPlaylist(body) => ctx.request("getPlaylist", body).await?,
        Command::GetPlaylists(body) => ctx.request("getPlaylists", body).await?,
        Command::GetPlayQueue(body) => ctx.request("getPlayQueue", body).await?,
        Command::GetPodcasts(body) => ctx.request("getPodcasts", body).await?,
        Command::GetRandomSongs(body) => ctx.request("getRandomSongs", body).await?,
        Command::GetScanStatus(body) => ctx.request("getScanStatus", body).await?,
        Command::GetShares(body) => ctx.request("getShares", body).await?,
        Command::GetSimilarSongs(body) => ctx.request("getSimilarSongs", body).await?,
        Command::GetSimilarSongs2(body) => ctx.request("getSimilarSongs2", body).await?,
        Command::GetSong(body) => ctx.request("getSong", body).await?,
        Command::GetSongsByGenre(body) => ctx.request("getSongsByGenre", body).await?,
        Command::GetStarred(body) => ctx.request("getStarred", body).await?,
        Command::GetStarred2(body) => ctx.request("getStarred2", body).await?,
        Command::GetTopSongs(body) => ctx.request("getTopSongs", body).await?,
        Command::GetUser(body) => ctx.request("getUser", body).await?,
        Command::GetUsers(body) => ctx.request("getUsers", body).await?,
        Command::GetVideoInfo(body) => ctx.request("getVideoInfo", body).await?,
        Command::GetVideos(body) => ctx.request("getVideos", body).await?,
        Command::Hls(body) => ctx.request("hls", body).await?,
        Command::JubeboxControl(body) => ctx.request("jubeboxControl", body).await?,
        Command::Ping(body) => ctx.request("ping", body).await?,
        Command::RefreshPodcasts(body) => ctx.request("refreshPodcasts", body).await?,
        Command::SavePlayQueue(body) => ctx.request("savePlayQueue", body).await?,
        Command::Scrobble(body) => ctx.request("scrobble", body).await?,
        Command::Search(body) => ctx.request("search", body).await?,
        Command::Search2(body) => ctx.request("search2", body).await?,
        Command::Search3(body) => ctx.request("search3", body).await?,
        Command::SetRating(body) => ctx.request("setRating", body).await?,
        Command::Star(body) => ctx.request("star", body).await?,
        Command::StartScan(body) => ctx.request("startScan", body).await?,
        Command::Unstar(body) => ctx.request("unstar", body).await?,
        Command::UpdateInternetRadioStation(body) => {
            ctx.request("updateInternetRadioStation", body).await?
        }
        Command::UpdatePlaylist(body) => ctx.request("updatePlaylist", body).await?,
        Command::UpdateShare(body) => ctx.request("updateShare", body).await?,
        Command::UpdateUser(body) => ctx.request("updateUser", body).await?,
    }

    Ok(())
}

