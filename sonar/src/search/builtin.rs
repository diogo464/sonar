use super::*;
use crate::async_trait;

#[derive(Debug)]
pub struct BuiltinSearchEngine;

#[async_trait]
impl SearchEngine for BuiltinSearchEngine {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResults> {
        todo!()
    }
    async fn synchronize_artist(&self, artist: ArtistId) {
        todo!()
    }
    async fn syncrhonize_album(&self, album: AlbumId) {
        todo!()
    }
    async fn synchronize_track(&self, track: TrackId) {
        todo!()
    }
    async fn synchronize_playlist(&self, playlist: PlaylistId) {
        todo!()
    }
    async fn synchronize_all(&self) {
        todo!()
    }
}
