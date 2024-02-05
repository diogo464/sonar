//! This module this with garbage collection of artists, albums and tracks.
//! This is necessary because we might download alot of data that we don't need anymore.
//! One example is when we download playlists like "Discover Weekly" and "Release Radar" from Spotify.
//! Most of the tracks downloaded from these playlists are only listened to once and if they are
//! not somehow marked as "liked" or "saved" they should be eventually deleted.
//! To do this we need will garbage collect artists, albums and tracks.
//! One of this items is a candidate for garbage collection if:
//! - It is not part of any playlist
//! - It is not pinned by any user
//! - None of its ancestors are pinned or part of any playlist
//! - None of its descendants are pinned or part of any playlist

use std::collections::{HashMap, HashSet};

use crate::{album, artist, db::DbC, pin, playlist, track, Result, SonarId};

pub async fn list_gc_candidates(db: &mut DbC) -> Result<Vec<SonarId>> {
    struct Rels {
        parent: Option<SonarId>,
        children: Vec<SonarId>,
    }

    let user_pinned: HashSet<SonarId> = pin::list_all(db).await?.into_iter().collect();
    let playlist_pinned: HashSet<SonarId> = playlist::list_tracks_in_all_playlists(db)
        .await?
        .into_iter()
        .map(From::from)
        .collect();

    let album_artist_pairs = album::list_artist_id_pairs(db).await?;
    let track_album_pairs = track::list_album_id_pairs(db).await?;
    let artists = artist::list_ids(db).await?;

    let mut candidates: HashSet<SonarId> = HashSet::new();
    let mut rels: HashMap<SonarId, Rels> = HashMap::new();
    for artist in artists {
        let artist = SonarId::from(artist);
        rels.insert(
            artist,
            Rels {
                parent: None,
                children: vec![],
            },
        );
    }
    for (album, artist) in album_artist_pairs {
        let album = SonarId::from(album);
        let artist = SonarId::from(artist);
        rels.entry(artist)
            .or_insert_with(|| Rels {
                parent: None,
                children: vec![],
            })
            .children
            .push(album);
        rels.entry(album)
            .or_insert_with(|| Rels {
                parent: None,
                children: vec![],
            })
            .parent = Some(artist);
    }
    for (track, album) in track_album_pairs {
        let track = SonarId::from(track);
        let album = SonarId::from(album);
        rels.entry(album)
            .or_insert_with(|| Rels {
                parent: None,
                children: vec![],
            })
            .children
            .push(track);
        rels.entry(track)
            .or_insert_with(|| Rels {
                parent: None,
                children: vec![],
            })
            .parent = Some(album);
    }
    candidates.extend(rels.keys().copied());

    let mut queue = Vec::new();
    for &sonar_id in user_pinned.iter().chain(playlist_pinned.iter()) {
        queue.clear();
        queue.push(sonar_id);

        while let Some(sonar_id) = queue.pop() {
            if candidates.remove(&sonar_id) {
                let r = &rels[&sonar_id];
                queue.extend(r.parent);
                queue.extend(r.children.iter());
            }
        }
    }

    Ok(candidates.into_iter().collect())
}
