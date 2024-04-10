use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use meilisearch_sdk::{Client, Index};
use serde::{Deserialize, Serialize};

use crate::{
    album, artist, db::Db, ext, playlist, track, AlbumId, ArtistId, Error, PlaylistId, Result,
    SearchQuery, SearchResult, TrackId, UserId,
};

use super::{SearchEngine, SearchResults};

// TODO: synchronize playlists.
// TODO: take care of user id when searching for playlists

const DEFAULT_SEARCH_LIMIT: u32 = 50;
const INDEX_NAME: &'static str = "items";
const INDEX_KEY: &'static str = "key";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum DocumentKind {
    Artist,
    Album,
    Track,
    Playlist,
}

impl std::fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DocumentKind::Artist => "artist",
            DocumentKind::Album => "album",
            DocumentKind::Track => "track",
            DocumentKind::Playlist => "playlist",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    kind: DocumentKind,
    artist_name: Option<String>,
    album_name: Option<String>,
    track_name: Option<String>,
    playlist_name: Option<String>,
    lyrics: Option<String>,
}

#[derive(Debug)]
pub struct MeiliSearchEngine {
    db: Db,
    client: Client,
    index: Index,
}

impl MeiliSearchEngine {
    pub async fn new(
        db: Db,
        meilisearch_url: impl AsRef<str>,
        meilisearch_key: impl AsRef<str>,
    ) -> Result<Self> {
        let client = Client::new(meilisearch_url.as_ref(), Some(meilisearch_key.as_ref()));
        let mut index = client.index(INDEX_NAME);

        index
            .set_primary_key(INDEX_KEY)
            .await
            .map_err(Error::wrap)?;

        Ok(Self { db, client, index })
    }

    async fn insert_document(&self, document: Document) {
        self.insert_documents(&[document]).await;
    }

    async fn insert_documents(&self, documents: &[Document]) {
        const MIN_TIMEOUT: Duration = Duration::from_secs(5);
        const EXTRA_PER_1K: Duration = Duration::from_secs(2); // arbitrary value with no basis on
                                                               // anything

        let timeout = MIN_TIMEOUT + EXTRA_PER_1K * (documents.len() as u32 / 1000);
        tracing::debug!(
            "inserting {} documents with timeout {:?}",
            documents.len(),
            timeout
        );
        tracing::trace!("documents: {:#?}", documents);
        match self.index.add_documents(documents, Some(INDEX_KEY)).await {
            Ok(task) => {
                tracing::debug!("waiting for completion of inserted documents");
                match task
                    .wait_for_completion(&self.client, None, Some(timeout))
                    .await
                {
                    Ok(task) => {
                        if task.is_success() {
                            tracing::debug!("inserted documents successfully");
                        } else {
                            let err = task.unwrap_failure();
                            tracing::error!("failed to insert documents: {}", err);
                        }
                    }
                    Err(err) => tracing::error!(
                        "failed to wait for completion of inserted documents: {}",
                        err
                    ),
                }
            }
            Err(err) => tracing::error!("failed to insert document: {}", err),
        }
    }

    async fn synchronize_artists(&self, artists: Vec<ArtistId>) {
        let mut conn = self.db.acquire().await.unwrap();
        let artists = artist::get_bulk(&mut conn, &artists).await.unwrap();
        let mut documents = Vec::with_capacity(artists.len());
        for artist in artists {
            documents.push(Document {
                id: artist.id.to_string(),
                kind: DocumentKind::Artist,
                artist_name: Some(artist.name),
                album_name: None,
                track_name: None,
                playlist_name: None,
                lyrics: None,
            });
        }
        self.insert_documents(&documents).await;
    }
    async fn synchronize_albums(&self, albums: Vec<AlbumId>) {
        let mut conn = self.db.acquire().await.unwrap();
        let albums = album::get_bulk(&mut conn, &albums).await.unwrap();

        let artist_ids = albums.iter().map(|a| a.artist).collect::<Vec<_>>();
        let artists = artist::get_bulk(&mut conn, &artist_ids).await.unwrap();
        let artists = ext::artists_map(artists);

        let mut documents = Vec::with_capacity(albums.len());
        for album in albums {
            let artist = &artists[&album.artist];
            documents.push(Document {
                id: album.id.to_string(),
                kind: DocumentKind::Artist,
                artist_name: Some(artist.name.clone()),
                album_name: Some(album.name),
                track_name: None,
                playlist_name: None,
                lyrics: None,
            });
        }
        self.insert_documents(&documents).await;
    }
    async fn synchronize_tracks(&self, tracks: Vec<TrackId>) {
        let mut conn = self.db.acquire().await.unwrap();
        let tracks = track::get_bulk(&mut conn, &tracks).await.unwrap();

        let album_ids = tracks.iter().map(|t| t.album).collect::<Vec<_>>();
        let albums = album::get_bulk(&mut conn, &album_ids).await.unwrap();
        let albums = ext::albums_map(albums);

        let artist_ids = tracks.iter().map(|t| t.artist).collect::<Vec<_>>();
        let artists = artist::get_bulk(&mut conn, &artist_ids).await.unwrap();
        let artists = ext::artists_map(artists);

        let mut documents = Vec::with_capacity(albums.len());
        for track in tracks {
            let artist = &artists[&track.artist];
            let album = &albums[&track.album];
            let lyrics = track::get_lyrics(&mut conn, track.id)
                .await
                .map(|l| {
                    l.lines
                        .into_iter()
                        .map(|x| x.text)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .ok();

            documents.push(Document {
                id: track.id.to_string(),
                kind: DocumentKind::Artist,
                artist_name: Some(artist.name.clone()),
                album_name: Some(album.name.clone()),
                track_name: Some(track.name),
                playlist_name: None,
                lyrics,
            });
        }
        self.insert_documents(&documents).await;
    }
    async fn synchronize_playlists(&self, playlists: Vec<PlaylistId>) {
        let mut conn = self.db.acquire().await.unwrap();
        let playlists = playlist::get_bulk(&mut conn, &playlists).await.unwrap();
        let mut documents = Vec::with_capacity(playlists.len());
        for playlist in playlists {
            documents.push(Document {
                id: playlist.id.to_string(),
                kind: DocumentKind::Playlist,
                artist_name: None,
                album_name: None,
                track_name: None,
                playlist_name: Some(playlist.name),
                lyrics: None,
            });
        }
        self.insert_documents(&documents).await;
    }
}

#[async_trait]
impl SearchEngine for MeiliSearchEngine {
    async fn search(&self, _user_id: UserId, query: &SearchQuery) -> Result<SearchResults> {
        tracing::info!("searching for {:?}", query);
        let mut q = self.index.search();
        q.with_query(&query.query);
        q.with_limit(query.limit.unwrap_or(DEFAULT_SEARCH_LIMIT) as usize);
        let results = q.execute::<Document>().await.map_err(Error::wrap)?;
        tracing::debug!("found {} results", results.hits.len());
        tracing::trace!("results: {:#?}", results);

        let mut artist_ids = Vec::new();
        let mut album_ids = Vec::new();
        let mut track_ids = Vec::new();
        let mut playlist_ids = Vec::new();

        for document in results.hits.iter() {
            match document.result.kind {
                DocumentKind::Artist => artist_ids.push(document.result.id.parse().unwrap()),
                DocumentKind::Album => album_ids.push(document.result.id.parse().unwrap()),
                DocumentKind::Track => track_ids.push(document.result.id.parse().unwrap()),
                DocumentKind::Playlist => playlist_ids.push(document.result.id.parse().unwrap()),
            }
        }

        let mut conn = self.db.acquire().await.unwrap();
        let artists = artist::get_bulk(&mut conn, &artist_ids).await.unwrap();
        let albums = album::get_bulk(&mut conn, &album_ids).await.unwrap();
        let tracks = track::get_bulk(&mut conn, &track_ids).await.unwrap();
        let playlists = playlist::get_bulk(&mut conn, &playlist_ids).await.unwrap();

        // TODO: maybe add some trait 'Entity' with associated type 'Key' and add a function to
        // ks to transform a vector of entities into a map of keys to entities
        let artists = artists
            .into_iter()
            .map(|x| (x.id, x))
            .collect::<HashMap<_, _>>();
        let albums = albums
            .into_iter()
            .map(|x| (x.id, x))
            .collect::<HashMap<_, _>>();
        let tracks = tracks
            .into_iter()
            .map(|x| (x.id, x))
            .collect::<HashMap<_, _>>();
        let playlists = playlists
            .into_iter()
            .map(|x| (x.id, x))
            .collect::<HashMap<_, _>>();

        let mut search_results = Vec::new();
        for document in results.hits {
            match document.result.kind {
                DocumentKind::Artist => search_results.push(SearchResult::Artist(
                    artists[&document.result.id.parse().unwrap()].clone(),
                )),
                DocumentKind::Album => search_results.push(SearchResult::Album(
                    albums[&document.result.id.parse().unwrap()].clone(),
                )),
                DocumentKind::Track => search_results.push(SearchResult::Track(
                    tracks[&document.result.id.parse().unwrap()].clone(),
                )),
                DocumentKind::Playlist => search_results.push(SearchResult::Playlist(
                    playlists[&document.result.id.parse().unwrap()].clone(),
                )),
            }
        }

        Ok(SearchResults {
            results: search_results,
        })
    }
    async fn synchronize_artist(&self, artist: ArtistId) {
        self.synchronize_artists(vec![artist]).await;
    }
    async fn synchronize_album(&self, album: AlbumId) {
        self.synchronize_albums(vec![album]).await;
    }
    async fn synchronize_track(&self, track: TrackId) {
        self.synchronize_tracks(vec![track]).await;
    }
    async fn synchronize_playlist(&self, playlist: PlaylistId) {
        self.synchronize_playlists(vec![playlist]).await;
    }
    async fn synchronize_all(&self) {
        let mut conn = self.db.acquire().await.unwrap();
        let track_ids = track::list_ids(&mut conn).await.unwrap();
        let album_ids = album::list_ids(&mut conn).await.unwrap();
        let artist_ids = artist::list_ids(&mut conn).await.unwrap();

        self.synchronize_tracks(track_ids).await;
        self.synchronize_albums(album_ids).await;
        self.synchronize_artists(artist_ids).await;
    }
}
