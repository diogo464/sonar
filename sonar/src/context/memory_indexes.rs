use std::{collections::HashMap, sync::Arc};

use crate::{genre::GenreStats, AlbumId, ArtistId, Context, Genre, ListParams, Result, TrackId};

#[derive(Debug, Default, Clone)]
pub struct MemoryIndexes {
    genres: Arc<MemoryGenresIndex>,
}

impl MemoryIndexes {
    pub fn genres(&self) -> &MemoryGenresIndex {
        &self.genres
    }
}

#[derive(Debug, Default)]
pub struct MemoryGenresIndex {
    stats: HashMap<Genre, GenreStats>,
    artists: HashMap<Genre, Vec<ArtistId>>,
    albums: HashMap<Genre, Vec<AlbumId>>,
    tracks: HashMap<Genre, Vec<TrackId>>,
}

impl MemoryGenresIndex {
    async fn new(context: &Context) -> Result<Self> {
        let artists = super::artist_list(context, Default::default());
        let albums = super::album_list(context, Default::default());
        let tracks = super::track_list(context, Default::default());
        let (artists, albums, tracks) = tokio::try_join!(artists, albums, tracks)?;

        let artists = crate::ext::artists_map(artists);
        let albums = crate::ext::albums_map(albums);

        let mut stats_map: HashMap<Genre, GenreStats> = HashMap::default();
        let mut artists_map: HashMap<Genre, Vec<ArtistId>> = HashMap::default();
        let mut albums_map: HashMap<Genre, Vec<AlbumId>> = HashMap::default();
        let mut tracks_map: HashMap<Genre, Vec<TrackId>> = HashMap::default();

        for artist in artists.values() {
            for genre in artist.genres.iter() {
                let entry = artists_map.entry(*genre).or_default();
                entry.push(artist.id);
            }
        }

        for album in albums.values() {
            let artist = &artists[&album.artist];
            for genre in artist.genres.iter().chain(album.genres.iter()) {
                let entry = albums_map.entry(*genre).or_default();
                entry.push(album.id);
            }
        }

        for track in tracks {
            let artist = &artists[&track.artist];
            let album = &albums[&track.album];
            for genre in artist.genres.iter().chain(album.genres.iter()) {
                let entry = tracks_map.entry(*genre).or_default();
                entry.push(track.id);
            }
        }

        // initialize genre stats to zero
        for genre in artists_map.keys().chain(albums_map.keys()) {
            stats_map.entry(*genre).or_insert(GenreStats {
                genre: *genre,
                num_artists: 0,
                num_albums: 0,
                num_tracks: 0,
            });
        }

        for (genre, artists) in artists_map.iter() {
            let stats = stats_map.get_mut(genre).unwrap();
            stats.num_artists += artists.len() as u32;
        }
        for (genre, albums) in albums_map.iter() {
            let stats = stats_map.get_mut(genre).unwrap();
            stats.num_albums += albums.len() as u32;
        }
        for (genre, tracks) in tracks_map.iter() {
            let stats = stats_map.get_mut(genre).unwrap();
            stats.num_tracks += tracks.len() as u32;
        }

        Ok(Self {
            stats: stats_map,
            artists: artists_map,
            albums: albums_map,
            tracks: tracks_map,
        })
    }

    pub fn list_genres(&self) -> Vec<GenreStats> {
        self.stats.values().cloned().collect()
    }

    pub fn list_albums_by_genre(&self, genre: &Genre, params: ListParams) -> Vec<AlbumId> {
        let albums = match self.albums.get(genre) {
            Some(albums) => albums,
            None => return Default::default(),
        };
        albums
            .iter()
            .skip(params.offset.unwrap_or(0) as usize)
            .take(params.limit.unwrap_or(u32::MAX) as usize)
            .copied()
            .collect()
    }
}

async fn try_memory_indexes_rebuild(context: &Context) -> Result<()> {
    let genres = MemoryGenresIndex::new(context).await?;

    let indexes = MemoryIndexes {
        genres: Arc::new(genres),
    };

    *context.memory_indexes.lock().unwrap() = indexes;
    Ok(())
}

pub async fn memory_indexes_rebuild(context: &Context) {
    match try_memory_indexes_rebuild(context).await {
        Ok(_) => tracing::info!("finished rebuilding memory indexes"),
        Err(err) => tracing::error!("failed to rebuild memory indexes: {}", err),
    }
}
