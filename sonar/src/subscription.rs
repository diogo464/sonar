use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    db::Db, download::DownloadController, external::SonarExternalService, ExternalMediaId, Result,
    UserId,
};

#[derive(Debug, Clone)]
pub struct Subscription {
    pub user: UserId,
    pub external_id: ExternalMediaId,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreate {
    pub user: UserId,
    pub external_id: ExternalMediaId,
}

#[derive(Debug, Clone)]
pub struct SubscriptionDelete {
    pub user: UserId,
    pub external_id: ExternalMediaId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SubscriptionKey {
    user: UserId,
    external_id: ExternalMediaId,
}

impl SubscriptionKey {
    fn new(user: UserId, external_id: ExternalMediaId) -> Self {
        Self { user, external_id }
    }
}

#[derive(Debug)]
struct SubscriptionState {
    last_download: Instant,
    description: String,
}

// TODO: fetch descriptions
#[derive(Debug)]
struct State {
    db: Db,
    downloads: DownloadController,
    _services: Vec<SonarExternalService>,
    subscriptions: Mutex<HashMap<SubscriptionKey, SubscriptionState>>,
}

// NOTE: this never gets dropped
#[derive(Debug, Clone)]
pub(crate) struct SubscriptionController(Arc<State>);

impl SubscriptionController {
    pub async fn new(
        db: Db,
        mut services: Vec<SonarExternalService>,
        downloads: DownloadController,
    ) -> Self {
        services.sort_by_key(|s| s.priority());

        let controller = Self(Arc::new(State {
            db,
            downloads,
            _services: services,
            subscriptions: Default::default(),
        }));
        tokio::spawn({
            let controller = controller.clone();
            async move {
                controller.update_loop().await;
            }
        });
        controller
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<Subscription>> {
        let rows = sqlx::query!("SELECT * FROM subscription WHERE user = ?", user_id)
            .fetch_all(&self.0.db)
            .await?;

        let mut subscriptions = Vec::with_capacity(rows.len());
        let states = self.0.subscriptions.lock().unwrap();
        for row in rows {
            let external_id = ExternalMediaId::from(row.external_id);
            let user_id = UserId::from_db(row.user);
            let key = SubscriptionKey {
                user: user_id,
                external_id,
            };
            let state = states.get(&key);
            subscriptions.push(Subscription {
                user: key.user,
                external_id: key.external_id,
                description: state.map(|s| s.description.clone()),
            });
        }

        Ok(subscriptions)
    }

    pub async fn create(&self, create: SubscriptionCreate) -> Result<()> {
        let external_id = create.external_id.as_str();
        sqlx::query!(
            "INSERT OR IGNORE INTO subscription (user, external_id) VALUES (?, ?)",
            create.user,
            external_id
        )
        .execute(&self.0.db)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, delete: SubscriptionDelete) -> Result<()> {
        let external_id = delete.external_id.as_str();
        sqlx::query!(
            "DELETE FROM subscription WHERE user = ? AND external_id = ?",
            delete.user,
            external_id
        )
        .execute(&self.0.db)
        .await?;
        Ok(())
    }

    async fn update_loop(&self) {
        loop {
            if let Err(err) = self.try_update().await {
                tracing::error!("subscription update error: {}", err);
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
    async fn try_update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = &self.0;
        let rows = sqlx::query!("SELECT * FROM subscription")
            .fetch_all(&state.db)
            .await?;

        let download_queue = {
            let mut subscriptions = state.subscriptions.lock().unwrap();
            for row in rows {
                let user_id = UserId::from_db(row.user);
                let external_id = ExternalMediaId::from(row.external_id);
                let key = SubscriptionKey::new(user_id, external_id);
                if !subscriptions.contains_key(&key) {
                    let state = SubscriptionState {
                        last_download: Instant::now(),
                        description: Default::default(),
                    };
                    subscriptions.insert(key, state);
                }
            }

            let sync_interval = std::time::Duration::from_secs(24 * 60 * 60);
            let mut download_queue = Vec::new();
            for (key, state) in subscriptions.iter_mut() {
                let elapsed = state.last_download.elapsed();
                if elapsed > sync_interval {
                    state.last_download = Instant::now();
                    download_queue.push(key.clone());
                } else {
                    tracing::debug!("skipping download for {:?}. elapsed: {:?}", key, elapsed);
                }
            }
            download_queue
        };

        for key in download_queue {
            tracing::info!("requesting download for {:?}", key);
            state.downloads.request(key.user, key.external_id).await;
        }

        Ok(())
    }
}
