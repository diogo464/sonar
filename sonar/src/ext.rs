use std::collections::HashMap;

use crate::{Album, AlbumId, Artist, ArtistId, Context, Result, Track, TrackId};

pub fn artists_map(artists: impl IntoIterator<Item = Artist>) -> HashMap<ArtistId, Artist> {
    artists
        .into_iter()
        .map(|artist| (artist.id, artist))
        .collect()
}

pub fn albums_map(albums: impl IntoIterator<Item = Album>) -> HashMap<AlbumId, Album> {
    albums.into_iter().map(|album| (album.id, album)).collect()
}

pub fn tracks_map(tracks: impl IntoIterator<Item = Track>) -> HashMap<TrackId, Track> {
    tracks.into_iter().map(|track| (track.id, track)).collect()
}

pub async fn artist_bulk(
    context: &Context,
    artist_ids: impl IntoIterator<Item = ArtistId>,
) -> Result<Vec<Artist>> {
    let artist_ids = artist_ids.into_iter().collect::<Vec<_>>();
    let artists = crate::artist_get_bulk(context, &artist_ids).await?;
    Ok(artists)
}

pub async fn artist_bulk_map(
    context: &Context,
    artist_ids: impl IntoIterator<Item = ArtistId>,
) -> Result<HashMap<ArtistId, Artist>> {
    let artists = artist_bulk(context, artist_ids).await?;
    Ok(artists_map(artists))
}

pub async fn album_bulk(
    context: &Context,
    album_ids: impl IntoIterator<Item = AlbumId>,
) -> Result<Vec<Album>> {
    let album_ids = album_ids.into_iter().collect::<Vec<_>>();
    let albums = crate::album_get_bulk(context, &album_ids).await?;
    Ok(albums)
}

pub async fn album_bulk_map(
    context: &Context,
    album_ids: impl IntoIterator<Item = AlbumId>,
) -> Result<HashMap<AlbumId, Album>> {
    let albums = album_bulk(context, album_ids).await?;
    Ok(albums_map(albums))
}

pub async fn track_bulk(
    context: &Context,
    track_ids: impl IntoIterator<Item = TrackId>,
) -> Result<Vec<Track>> {
    let track_ids = track_ids.into_iter().collect::<Vec<_>>();
    let tracks = crate::track_get_bulk(context, &track_ids).await?;
    Ok(tracks)
}

pub async fn track_bulk_map(
    context: &Context,
    track_ids: impl IntoIterator<Item = TrackId>,
) -> Result<HashMap<TrackId, Track>> {
    let tracks = track_bulk(context, track_ids).await?;
    Ok(tracks_map(tracks))
}
