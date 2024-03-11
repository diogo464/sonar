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

pub async fn track_list_all(client: &mut Client) -> Result<Vec<Track>> {
    let mut offset = 0;
    let limit = 1000;
    let mut tracks = Vec::new();

    loop {
        let response = client
            .track_list(super::TrackListRequest {
                offset: Some(offset),
                count: Some(limit),
            })
            .await?;

        let response = response.into_inner();
        let tracks_len = response.tracks.len();
        tracks.extend(response.tracks);
        offset += limit;

        if tracks_len < limit as usize {
            break;
        }
    }

    Ok(tracks)
}
