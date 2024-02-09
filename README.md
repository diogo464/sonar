```rust
fn main() {
    let MIGRATION_000 = migrator::include!("migrations/000_init.sql");
    let MIGRATION_001 = migrator::include!("migrations/001_add_blob_hash.sql");
    let MIGRATION_002 = migrator::include!("migrations/002_add_modified_timestamp.sql");

    let db = todo!();

    migrator::run(&db, &MIGRATION_000).await?;
    migrator::run_with(&db, &MIGRATION_001, |tx| async {
        let rows = sqlx::query!("SELECT * FROM blob").await;
        for row in rows {
            let path = row.path;
            let content = tokio::fs::read(&path).await?;
        }
    })
    .await?;
    migrator::run_with(&db, &MIGRATION_002, |tx| async {
        
    })
    .await?;
}
```
