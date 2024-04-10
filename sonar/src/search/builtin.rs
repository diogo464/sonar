use super::*;
use crate::{album, artist, async_trait, db::Db, playlist, track};

#[derive(Debug)]
pub struct BuiltInSearchEngine {
    db: Db,
}

impl BuiltInSearchEngine {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SearchEngine for BuiltInSearchEngine {
    #[tracing::instrument]
    async fn search(&self, user_id: UserId, query: &SearchQuery) -> Result<SearchResults> {
        let mut conn = self.db.acquire().await?;
        let mut results = SearchResults::default();
        let query_term = format!("%{}%", query.query);

        if query.flags & SearchQuery::FLAG_ARTIST != 0 {
            let artist_ids = sqlx::query_scalar("SELECT id FROM artist WHERE name LIKE ?")
                .bind(&query_term)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(ArtistId::from_db)
                .collect::<Vec<_>>();
            let artists = artist::get_bulk(&mut conn, &artist_ids).await?;
            results.results.extend(artists.into_iter().map(From::from));
        }

        if query.flags & SearchQuery::FLAG_ALBUM != 0 {
            let album_ids = sqlx::query_scalar("SELECT id FROM album WHERE name LIKE ?")
                .bind(&query_term)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(AlbumId::from_db)
                .collect::<Vec<_>>();
            let albums = album::get_bulk(&mut conn, &album_ids).await?;
            results.results.extend(albums.into_iter().map(From::from));
        }

        if query.flags & SearchQuery::FLAG_TRACK != 0 {
            let track_ids = sqlx::query_scalar("SELECT id FROM track WHERE name LIKE ?")
                .bind(&query_term)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(TrackId::from_db)
                .collect::<Vec<_>>();
            let tracks = track::get_bulk(&mut conn, &track_ids).await?;
            results.results.extend(tracks.into_iter().map(From::from));
        }

        if query.flags & SearchQuery::FLAG_PLAYLIST != 0 {
            let playlist_ids =
                sqlx::query_scalar("SELECT id FROM playlist WHERE owner = ? AND name LIKE ?")
                    .bind(user_id)
                    .bind(query_term)
                    .fetch_all(&mut *conn)
                    .await?
                    .into_iter()
                    .map(PlaylistId::from_db)
                    .collect::<Vec<_>>();
            let playlists = playlist::get_bulk(&mut conn, &playlist_ids).await?;
            results
                .results
                .extend(playlists.into_iter().map(From::from));
        }

        Ok(results)
    }
    async fn synchronize_artist(&self, _artist: ArtistId) {}
    async fn synchronize_album(&self, _album: AlbumId) {}
    async fn synchronize_track(&self, _track: TrackId) {}
    async fn synchronize_playlist(&self, _playlist: PlaylistId) {}
    async fn synchronize_all(&self) {}
}
