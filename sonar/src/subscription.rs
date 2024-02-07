use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{db::Db, external::SonarExternalService, ExternalMediaId, Result, UserId};

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

// TODO: fetch descriptions
#[derive(Debug)]
struct State {
    db: Db,
    _services: Vec<SonarExternalService>,
    descriptions: Mutex<HashMap<SubscriptionKey, String>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubscriptionController(Arc<State>);

impl SubscriptionController {
    pub async fn new(db: Db, mut services: Vec<SonarExternalService>) -> Self {
        services.sort_by_key(|s| s.priority());

        Self(Arc::new(State {
            db,
            _services: services,
            descriptions: Default::default(),
        }))
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<Subscription>> {
        let rows = sqlx::query!("SELECT * FROM subscription WHERE user = ?", user_id)
            .fetch_all(&self.0.db)
            .await?;

        let mut subscriptions = Vec::with_capacity(rows.len());
        let descriptions = self.0.descriptions.lock().unwrap();
        for row in rows {
            let external_id = ExternalMediaId::from(row.external_id);
            let user_id = UserId::from_db(row.user);
            let key = SubscriptionKey {
                user: user_id,
                external_id,
            };
            let description = descriptions.get(&key).cloned();
            subscriptions.push(Subscription {
                user: key.user,
                external_id: key.external_id,
                description,
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
}
