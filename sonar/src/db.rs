use crate::{
    Error, ErrorKind, Genres, ListParams, Properties, Result, SonarIdentifier, ValueUpdate,
};

pub type Db = sqlx::SqlitePool;
pub type DbC = sqlx::SqliteConnection;

pub fn query_builder_push_id_tuple<I, ID, DB>(builder: &mut sqlx::QueryBuilder<'_, DB>, ids: I)
where
    I: IntoIterator<Item = ID>,
    ID: SonarIdentifier,
    DB: sqlx::Database,
{
    builder.push(" (");
    for (i, id) in ids.into_iter().enumerate() {
        if i > 0 {
            builder.push(", ");
        }
        builder.push(id.identifier());
    }
    builder.push(") ");
}

pub async fn get_by_id<T, ID>(db: &mut DbC, table: &str, id: ID) -> Result<T>
where
    T: for<'a> sqlx::FromRow<'a, sqlx::sqlite::SqliteRow> + Send + Unpin,
    ID: SonarIdentifier,
{
    Ok(sqlx::QueryBuilder::new("SELECT * FROM ")
        .push(table)
        .push(" WHERE id = ?")
        .build_query_as::<T>()
        .bind(id.identifier())
        .fetch_one(&mut *db)
        .await?)
}

pub async fn list<T>(db: &mut DbC, view: &str, params: ListParams) -> Result<Vec<T>>
where
    T: for<'a> sqlx::FromRow<'a, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    let (offset, limit) = params.to_db_offset_limit();
    Ok(sqlx::QueryBuilder::new("SELECT * FROM ")
        .push(view)
        .push(" ORDER BY id ASC LIMIT ? OFFSET ?")
        .build_query_as::<T>()
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *db)
        .await?)
}

pub async fn list_where_field_eq<T, V>(
    db: &mut DbC,
    view: &str,
    field: &str,
    value: V,
    params: ListParams,
) -> Result<Vec<T>>
where
    T: for<'a> sqlx::FromRow<'a, sqlx::sqlite::SqliteRow> + Send + Unpin,
    V: sqlx::Type<sqlx::sqlite::Sqlite>
        + for<'a> sqlx::Encode<'a, sqlx::sqlite::Sqlite>
        + Send
        + 'static,
{
    let (offset, limit) = params.to_db_offset_limit();
    Ok(sqlx::QueryBuilder::new("SELECT * FROM ")
        .push(view)
        .push(" WHERE ")
        .push(field)
        .push(" = ?")
        .push(" ORDER BY id ASC LIMIT ? OFFSET ?")
        .build_query_as::<T>()
        .bind(value)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *db)
        .await?)
}

pub async fn list_bulk<T, ID>(
    db: &mut DbC,
    table: &str,
    ids: impl IntoIterator<Item = ID>,
) -> Result<Vec<T>>
where
    T: for<'a> sqlx::FromRow<'a, sqlx::sqlite::SqliteRow> + Send + Unpin,
    ID: SonarIdentifier,
{
    let mut query = sqlx::QueryBuilder::new("SELECT * FROM ");
    query.push(table).push(" WHERE id IN ");
    query_builder_push_id_tuple(&mut query, ids);
    Ok(query.build_query_as::<T>().fetch_all(&mut *db).await?)
}

pub async fn value_update_string_non_null(
    db: &mut DbC,
    table: &str,
    field: &str,
    id: impl SonarIdentifier,
    update: ValueUpdate<String>,
) -> Result<()> {
    if let Some(new_value) = match update {
        ValueUpdate::Set(value) => Some(value),
        ValueUpdate::Unset => Some("".to_owned()),
        ValueUpdate::Unchanged => None,
    } {
        sqlx::QueryBuilder::new("UPDATE ")
            .push(table)
            .push(" SET ")
            .push(field)
            .push(" = ? WHERE id = ?")
            .build()
            .bind(new_value)
            .bind(id.identifier())
            .execute(&mut *db)
            .await?;
    }
    Ok(())
}

pub async fn value_update_id_non_null(
    db: &mut DbC,
    table: &str,
    field: &str,
    id: impl SonarIdentifier,
    update: ValueUpdate<impl SonarIdentifier>,
) -> Result<()> {
    if let Some(new_value) = match update {
        ValueUpdate::Set(value) => Some(value.identifier()),
        ValueUpdate::Unset => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("cannot unsed {field} on {table} update"),
            ))
        }
        ValueUpdate::Unchanged => None,
    } {
        sqlx::QueryBuilder::new("UPDATE ")
            .push(table)
            .push(" SET ")
            .push(field)
            .push(" = ? WHERE id = ?")
            .build()
            .bind(new_value)
            .bind(id.identifier())
            .execute(db)
            .await?;
    }
    Ok(())
}

pub async fn value_update_id_nullable(
    db: &mut DbC,
    table: &str,
    field: &str,
    id: impl SonarIdentifier,
    update: ValueUpdate<impl SonarIdentifier>,
) -> Result<()> {
    match update {
        ValueUpdate::Set(value) => {
            sqlx::QueryBuilder::new("UPDATE ")
                .push(table)
                .push(" SET ")
                .push(field)
                .push(" = ? WHERE id = ?")
                .build()
                .bind(value.identifier())
                .bind(id.identifier())
                .execute(db)
                .await?;
        }
        ValueUpdate::Unset => {
            sqlx::QueryBuilder::new("UPDATE ")
                .push(table)
                .push(" SET ")
                .push(field)
                .push(" = NULL WHERE id = ?")
                .build()
                .bind(id.identifier())
                .execute(db)
                .await?;
        }
        ValueUpdate::Unchanged => {}
    }
    Ok(())
}

pub fn merge_view_properties<T, R>(views: Vec<T>, properties: Vec<Properties>) -> Vec<R>
where
    R: From<(T, Properties)>,
{
    views
        .into_iter()
        .zip(properties.into_iter())
        .map(From::from)
        .collect()
}

pub fn merge_view_genres_properties<T, R>(
    views: Vec<T>,
    genres: Vec<Genres>,
    properties: Vec<Properties>,
) -> Vec<R>
where
    R: From<(T, Genres, Properties)>,
{
    if views.len() != genres.len() || views.len() != properties.len() {
        panic!("merge_view_genres_properties: input vectors must have the same length");
    }

    views
        .into_iter()
        .zip(genres.into_iter())
        .zip(properties.into_iter())
        .map(|((view, genres), properties)| From::from((view, genres, properties)))
        .collect()
}
