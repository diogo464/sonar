use crate::{
    async_trait, Album, AlbumId, Artist, ArtistId, Playlist, PlaylistId, Result, Track, TrackId, UserId,
};

mod builtin;
pub use builtin::BuiltInSearchEngine;

pub type SearchFlags = u32;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<u32>,
    pub flags: SearchFlags,
}

impl SearchQuery {
    pub const FLAG_NONE: SearchFlags = 0;
    pub const FLAG_ARTIST: SearchFlags = 1 << 0;
    pub const FLAG_ALBUM: SearchFlags = 1 << 1;
    pub const FLAG_TRACK: SearchFlags = 1 << 2;
    pub const FLAG_PLAYLIST: SearchFlags = 1 << 3;
    pub const FLAG_ALL: SearchFlags =
        Self::FLAG_ARTIST | Self::FLAG_ALBUM | Self::FLAG_TRACK | Self::FLAG_PLAYLIST;
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Artist(Artist),
    Album(Album),
    Track(Track),
    Playlist(Playlist),
}

impl From<Artist> for SearchResult {
    fn from(artist: Artist) -> Self {
        Self::Artist(artist)
    }
}

impl From<Album> for SearchResult {
    fn from(album: Album) -> Self {
        Self::Album(album)
    }
}

impl From<Track> for SearchResult {
    fn from(track: Track) -> Self {
        Self::Track(track)
    }
}

impl From<Playlist> for SearchResult {
    fn from(playlist: Playlist) -> Self {
        Self::Playlist(playlist)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
}

#[async_trait]
pub trait SearchEngine: std::fmt::Debug + Send + Sync + 'static {
    async fn search(&self, user_id: UserId, query: &SearchQuery) -> Result<SearchResults>;
    async fn synchronize_artist(&self, artist: ArtistId);
    async fn synchronize_album(&self, album: AlbumId);
    async fn synchronize_track(&self, track: TrackId);
    async fn synchronize_playlist(&self, playlist: PlaylistId);
    async fn synchronize_all(&self);
}

impl SearchResults {
    pub fn artists(&self) -> impl Iterator<Item = &Artist> {
        self.results.iter().filter_map(|result| match result {
            SearchResult::Artist(artist) => Some(artist),
            _ => None,
        })
    }

    pub fn into_artists(self) -> impl Iterator<Item = Artist> {
        self.results.into_iter().filter_map(|result| match result {
            SearchResult::Artist(artist) => Some(artist),
            _ => None,
        })
    }

    pub fn albums(&self) -> impl Iterator<Item = &Album> {
        self.results.iter().filter_map(|result| match result {
            SearchResult::Album(album) => Some(album),
            _ => None,
        })
    }

    pub fn into_albums(self) -> impl Iterator<Item = Album> {
        self.results.into_iter().filter_map(|result| match result {
            SearchResult::Album(album) => Some(album),
            _ => None,
        })
    }

    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.results.iter().filter_map(|result| match result {
            SearchResult::Track(track) => Some(track),
            _ => None,
        })
    }

    pub fn into_tracks(self) -> impl Iterator<Item = Track> {
        self.results.into_iter().filter_map(|result| match result {
            SearchResult::Track(track) => Some(track),
            _ => None,
        })
    }

    pub fn playlists(&self) -> impl Iterator<Item = &Playlist> {
        self.results.iter().filter_map(|result| match result {
            SearchResult::Playlist(playlist) => Some(playlist),
            _ => None,
        })
    }

    pub fn into_playlists(self) -> impl Iterator<Item = Playlist> {
        self.results.into_iter().filter_map(|result| match result {
            SearchResult::Playlist(playlist) => Some(playlist),
            _ => None,
        })
    }
}
