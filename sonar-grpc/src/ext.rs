use crate::{Album, Artist, Client, Track};
use eyre::Result;

pub async fn artist_list_all(client: &mut Client) -> Result<Vec<Artist>> {
    let mut offset = 0;
    let limit = 1000;
    let mut artists = Vec::new();

    loop {
        let response = client
            .artist_list(super::ArtistListRequest {
                offset: Some(offset),
                count: Some(limit),
            })
            .await?;

        let response = response.into_inner();
        let artists_len = response.artists.len();
        artists.extend(response.artists);
        offset += limit;

        if artists_len < limit as usize {
            break;
        }
    }

    Ok(artists)
}

pub async fn album_list_all(client: &mut Client) -> Result<Vec<Album>> {
    let mut offset = 0;
    let limit = 1000;
    let mut albums = Vec::new();

    loop {
        let response = client
            .album_list(super::AlbumListRequest {
                offset: Some(offset),
                count: Some(limit),
            })
            .await?;

        let response = response.into_inner();
        let albums_len = response.albums.len();
        albums.extend(response.albums);
        offset += limit;

        if albums_len < limit as usize {
            break;
        }
    }

    Ok(albums)
}

pub async fn track_list(
    client: &mut Client,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<Track>> {
    let mut offset = offset.unwrap_or(0);
    let mut limit = limit.unwrap_or(u32::MAX);
    let limit_per_req = 1000;
    let mut tracks = Vec::new();

    loop {
        let response = client
            .track_list(super::TrackListRequest {
                offset: Some(offset),
                count: Some(limit_per_req.min(limit)),
            })
            .await?;

        let response = response.into_inner();
        let tracks_len = response.tracks.len();
        offset += limit_per_req;
        limit -= response.tracks.len() as u32;
        tracks.extend(response.tracks);

        if tracks_len < limit_per_req as usize {
            break;
        }
    }

    Ok(tracks)
}

pub async fn track_list_all(client: &mut Client) -> Result<Vec<Track>> {
    track_list(client, None, None).await
}
