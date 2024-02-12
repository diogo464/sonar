use std::{future::Future, pin::Pin};

use sqlx::Row;

use crate::{
    db::{Db, DbC},
    Error, Result,
};

#[derive(Debug)]
struct Migration {
    filename: &'static str,
    content: &'static str,
}

macro_rules! migration {
    ($filename:literal) => {
        Migration {
            filename: $filename,
            content: include_str!($filename),
        }
    };
}

pub async fn run(db: &Db) -> Result<()> {
    tracing::info!("running migrations");
    run_migration(db, migration!("000_init.sql")).await?;
    tracing::info!("migrations complete");
    Ok(())
}

async fn run_migration(db: &Db, migration: Migration) -> Result<()> {
    run_migration_with(db, migration, |_| Box::pin(async move { Ok(()) })).await
}

async fn run_migration_with<F>(db: &Db, migration: Migration, f: F) -> Result<()>
where
    for<'a> F: FnOnce(&'a mut DbC) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>,
{
    let filename = if migration.filename.ends_with(".sql") {
        &migration.filename[..migration.filename.len() - 4]
    } else {
        migration.filename
    };
    let migration_name = filename
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(filename);

    tracing::info!("applying migration {}", migration_name);

    sqlx::query("CREATE TABLE IF NOT EXISTS migration (name TEXT PRIMARY KEY, content TEXT)")
        .execute(db)
        .await?;

    let existing = sqlx::query("SELECT content FROM migration WHERE name = ?")
        .bind(migration_name)
        .fetch_optional(db)
        .await?;
    if let Some(existing) = existing {
        let existing_content = existing.get::<String, _>(0);
        if existing_content == migration.content {
            tracing::info!("migration {} already applied", migration_name);
            return Ok(());
        } else {
            tracing::error!(
                "migration {} already applied with different content\n{}",
                migration_name,
                existing_content
            );
            return Err(Error::internal(format!(
                "migration {} already applied with different content",
                migration_name
            )));
        }
    }

    let (pre, post) = match migration.content.split_once("--@code") {
        Some((pre, post)) => (pre, post),
        None => (migration.content, ""),
    };

    let mut tx = db.begin().await?;
    tracing::debug!("running pre-migration code\n{}", pre);
    sqlx::query(pre).execute(&mut *tx).await?;
    tracing::debug!("running migration code");
    f(&mut tx).await?;
    tracing::debug!("running post-migration code\n{}", post);
    sqlx::query(post).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO migration (name, content) VALUES (?, ?)")
        .bind(migration_name)
        .bind(migration.content)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    tracing::info!("applied migration {}", migration_name);
    Ok(())
}
