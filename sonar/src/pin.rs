use sqlx::Row;

use crate::{db::DbC, Result, SonarId, SonarIdentifier, UserId};

pub async fn list(db: &mut DbC, user_id: UserId) -> Result<Vec<SonarId>> {
    let rows = sqlx::query("SELECT namespace, identifier FROM pin WHERE user = ?")
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let mut sonar_ids = Vec::<SonarId>::with_capacity(rows.len());
    for row in rows {
        let namespace = row.get::<i64, _>(0) as u32;
        let identifier = row.get::<i64, _>(1) as u32;
        let sonar_id = SonarId::from_namespace_and_id(namespace, identifier)
            .expect("invalid identifier in database");
        sonar_ids.push(sonar_id);
    }

    Ok(sonar_ids)
}

pub async fn list_all(db: &mut DbC) -> Result<Vec<SonarId>> {
    let rows = sqlx::query("SELECT namespace, identifier FROM pin")
        .fetch_all(db)
        .await?;

    let mut sonar_ids = Vec::<SonarId>::with_capacity(rows.len());
    for row in rows {
        let namespace = row.get::<i64, _>(0) as u32;
        let identifier = row.get::<i64, _>(1) as u32;
        let sonar_id = SonarId::from_namespace_and_id(namespace, identifier)
            .expect("invalid identifier in database");
        sonar_ids.push(sonar_id);
    }

    Ok(sonar_ids)
}

pub async fn set(db: &mut DbC, user_id: UserId, sonar_id: SonarId) -> Result<()> {
    set_bulk(db, user_id, &[sonar_id]).await
}

pub async fn set_bulk(db: &mut DbC, user_id: UserId, sonar_ids: &[SonarId]) -> Result<()> {
    for sonar_id in sonar_ids {
        let namespace = sonar_id.namespace() as i64;
        let identifier = sonar_id.identifier() as i64;
        sqlx::query("INSERT OR IGNORE INTO pin (user, namespace, identifier) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(namespace)
            .bind(identifier)
            .execute(&mut *db)
            .await?;
    }
    Ok(())
}

pub async fn unset(db: &mut DbC, user_id: UserId, sonar_id: SonarId) -> Result<()> {
    unset_bulk(db, user_id, &[sonar_id]).await
}

pub async fn unset_bulk(db: &mut DbC, user_id: UserId, sonar_ids: &[SonarId]) -> Result<()> {
    for sonar_id in sonar_ids {
        sqlx::query("DELETE FROM pin WHERE user = ? AND namespace = ? AND identifier = ?")
            .bind(user_id)
            .bind(sonar_id.namespace() as i64)
            .bind(sonar_id.identifier() as i64)
            .execute(&mut *db)
            .await?;
    }
    Ok(())
}
