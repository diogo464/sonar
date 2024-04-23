use std::time::Duration;

use crate::{Context, Result};

pub(super) async fn run(context: &Context) {
    loop {
        if let Err(err) = iteration(context).await {
            tracing::error!("failed to run playlist cover process iteration: {err}");
        }
        tokio::time::sleep(Duration::from_mins(30)).await;
    }
}

async fn iteration(context: &Context) -> Result<()> {
    let playlists = super::playlist_list(context, Default::default()).await?;
    for playlist in playlists {
        if playlist.cover_art.is_some() {
            tracing::debug!("playlist {} already has cover art, skipping", playlist.id);
            continue;
        }
        if let Err(err) = super::playlist_generate_cover(context, playlist.id).await {
            tracing::error!(
                "failed to update playlist {} cover art: {}",
                playlist.id,
                err
            );
        }
    }
    Ok(())
}
