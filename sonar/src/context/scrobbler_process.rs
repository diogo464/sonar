use crate::{scrobbler::SonarScrobbler, Context};

pub(super) async fn run(context: Context, scrobbler: SonarScrobbler) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        let scrobbles = if let Some(username) = scrobbler.username() {
            let user_id = match super::user_lookup(&context, username).await {
                Ok(Some(user_id)) => user_id,
                Ok(None) => {
                    tracing::warn!("user not found: {:?}", username);
                    continue;
                }
                Err(e) => {
                    tracing::error!("failed to look up user: {:?}", e);
                    continue;
                }
            };
            super::scrobble_list_unsubmitted_for_user(&context, scrobbler.identifier(), user_id)
                .await
        } else {
            super::scrobble_list_unsubmitted(&context, scrobbler.identifier()).await
        };

        let scrobbles = match scrobbles {
            Ok(scrobbles) => scrobbles,
            Err(e) => {
                tracing::error!("failed to list unsubmitted scrobbles: {:?}", e);
                continue;
            }
        };

        for scrobble in scrobbles {
            tracing::info!("scrobbling with {}: {:?}", scrobbler.identifier(), scrobble);
            match scrobbler.scrobble(&context, scrobble.clone()).await {
                Ok(_) => {
                    if let Err(err) = super::scrobble_register_submission(
                        &context,
                        scrobble.id,
                        scrobbler.identifier(),
                    )
                    .await
                    {
                        tracing::error!("failed to register submission: {:?}", err);
                        break;
                    }

                    tracing::info!("scrobbled: {:?}", scrobble);
                }
                Err(e) => tracing::error!("failed to scrobble: {:?}", e),
            }
        }
    }
}
