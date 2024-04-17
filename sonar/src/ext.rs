use std::collections::HashMap;

use crate::{
    Album, AlbumId, Artist, ArtistId, Audio, AudioId, Context, PlaylistId, Properties, Result,
    SonarId, Track, TrackId, UserId,
};

#[derive(Debug, Clone)]
pub struct SplitIds {
    pub artist_ids: Vec<ArtistId>,
    pub album_ids: Vec<AlbumId>,
    pub track_ids: Vec<TrackId>,
    pub playlist_ids: Vec<PlaylistId>,
}

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

pub fn audios_map(audios: impl IntoIterator<Item = Audio>) -> HashMap<AudioId, Audio> {
    audios.into_iter().map(|audio| (audio.id, audio)).collect()
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

pub async fn audio_bulk(
    context: &Context,
    audio_ids: impl IntoIterator<Item = AudioId>,
) -> Result<Vec<Audio>> {
    let audio_ids = audio_ids.into_iter().collect::<Vec<_>>();
    let audios = crate::audio_get_bulk(context, &audio_ids).await?;
    Ok(audios)
}

pub async fn audio_bulk_map(
    context: &Context,
    audio_ids: impl IntoIterator<Item = AudioId>,
) -> Result<HashMap<AudioId, Audio>> {
    let audios = audio_bulk(context, audio_ids).await?;
    Ok(audios_map(audios))
}

pub async fn user_property_bulk_map(
    context: &Context,
    user_id: UserId,
    ids: impl IntoIterator<Item = SonarId>,
) -> Result<HashMap<SonarId, Properties>> {
    let ids = ids.into_iter().collect::<Vec<_>>();
    let properties = crate::user_property_get_bulk(context, user_id, &ids).await?;
    Ok(ids.into_iter().zip(properties).collect())
}

pub async fn get_albums_artists_map(
    context: &Context,
    albums: impl IntoIterator<Item = &Album>,
) -> Result<HashMap<ArtistId, Artist>> {
    let mut artist_ids = Vec::new();
    for album in albums.into_iter() {
        artist_ids.push(album.artist);
    }
    let artists = artist_bulk(context, artist_ids).await?;
    Ok(artists_map(artists))
}

pub async fn get_tracks_artists_map(
    context: &Context,
    tracks: impl IntoIterator<Item = &Track>,
) -> Result<HashMap<ArtistId, Artist>> {
    let mut artist_ids = Vec::new();
    for track in tracks.into_iter() {
        artist_ids.push(track.artist);
    }
    let artists = artist_bulk(context, artist_ids).await?;
    Ok(artists_map(artists))
}

pub async fn get_tracks_albums_map(
    context: &Context,
    tracks: impl IntoIterator<Item = &Track>,
) -> Result<HashMap<AlbumId, Album>> {
    let mut album_ids = Vec::new();
    for track in tracks.into_iter() {
        album_ids.push(track.album);
    }
    let albums = album_bulk(context, album_ids).await?;
    Ok(albums_map(albums))
}

pub async fn get_tracks_audios_map(
    context: &Context,
    tracks: impl IntoIterator<Item = &Track>,
) -> Result<HashMap<AudioId, Audio>> {
    let audio_ids = tracks
        .into_iter()
        .filter_map(|t| t.audio)
        .collect::<Vec<_>>();
    let audios = audio_bulk(context, audio_ids).await?;
    Ok(audios_map(audios))
}

pub fn split_sonar_ids(ids: impl IntoIterator<Item = SonarId>) -> SplitIds {
    let mut artist_ids = Vec::new();
    let mut album_ids = Vec::new();
    let mut track_ids = Vec::new();
    let mut playlist_ids = Vec::new();
    for id in ids.into_iter() {
        match id {
            SonarId::Artist(id) => artist_ids.push(id),
            SonarId::Album(id) => album_ids.push(id),
            SonarId::Track(id) => track_ids.push(id),
            SonarId::Playlist(id) => playlist_ids.push(id),
            _ => {}
        }
    }
    SplitIds {
        artist_ids,
        album_ids,
        track_ids,
        playlist_ids,
    }
}
