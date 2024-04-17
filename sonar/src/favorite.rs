use std::collections::HashSet;

use crate::{db::DbC, Error, Result, SonarId, SonarIdentifier, Timestamp, UserId};

use sqlx::Row;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Favorite {
    pub id: SonarId,
    pub favorite_at: Timestamp,
}

#[derive(Debug)]
struct FavoriteView {
    user: u32,
    namespace: u32,
    identifier: u32,
    created_at: u32,
}

pub(crate) async fn user_list(db: &mut DbC, user_id: UserId) -> Result<Vec<Favorite>> {
    let rows = sqlx::query("SELECT * FROM favorite WHERE user = ?")
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let mut favorites = Vec::with_capacity(rows.len());
    for row in rows {
        let namespace = row.get::<u32, _>("namespace");
        let identifier = row.get::<u32, _>("identifier");
        let sonar_id = SonarId::from_namespace_and_id(namespace, identifier)
            .expect("invalid sonar_id in database");
        let unix_timestamp = row.get::<u32, _>("created_at");
        let timestamp = Timestamp::from_seconds(u64::from(unix_timestamp));
        favorites.push(Favorite {
            id: sonar_id,
            favorite_at: timestamp,
        });
    }
    Ok(favorites)
}

pub(crate) async fn user_get_bulk(
    db: &mut DbC,
    user_id: UserId,
    ids: &[SonarId],
) -> Result<Vec<Favorite>> {
    // TODO: improve this
    let favorites = user_list(db, user_id).await?;
    let id_set = ids.iter().collect::<HashSet<_>>();
    Ok(favorites
        .into_iter()
        .filter(|f| id_set.contains(&f.id))
        .collect())
}

pub(crate) async fn user_put(db: &mut DbC, user_id: UserId, id: SonarId) -> Result<()> {
    if !std::matches!(
        id,
        SonarId::Artist(_) | SonarId::Album(_) | SonarId::Track(_)
    ) {
        return Err(Error::new(
            crate::ErrorKind::Invalid,
            "cannot favorite item type",
        ));
    }

    let namespace = id.namespace();
    let identifier = id.identifier();
    sqlx::query("INSERT OR IGNORE INTO favorite(user, namespace, identifier) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(namespace)
        .bind(identifier)
        .execute(db)
        .await?;

    Ok(())
}

pub(crate) async fn user_remove(db: &mut DbC, user_id: UserId, id: SonarId) -> Result<()> {
    let namespace = id.namespace();
    let identifier = id.identifier();
    sqlx::query("DELETE FROM favorite WHERE user = ? AND namespace = ? AND identifier = ?")
        .bind(user_id)
        .bind(namespace)
        .bind(identifier)
        .execute(db)
        .await?;
    Ok(())
}
