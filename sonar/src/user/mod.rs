use crate::{
    db::{self, DbC},
    Error, ErrorKind, ImageId, ListParams, Result, UserId, ValueUpdate,
};

mod username;
use sqlx::Row;
pub use username::*;

mod token;
pub use token::*;

const PASSWORD_MIN_LENGTH: usize = 8;
const PASSWORD_MAX_LENGTH: usize = 48;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub avatar: Option<ImageId>,
    pub admin: bool,
}

#[derive(Clone)]
pub struct UserCreate {
    pub username: Username,
    pub password: String,
    pub avatar: Option<ImageId>,
    pub admin: bool,
}

impl std::fmt::Debug for UserCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserCreate")
            .field("username", &self.username)
            .field("password", &"****")
            .field("avatar", &self.avatar)
            .field("admin", &self.admin)
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct UserUpdate {
    pub password: ValueUpdate<String>,
    pub avatar: ValueUpdate<ImageId>,
    pub admin: ValueUpdate<bool>,
}

impl std::fmt::Debug for UserUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserUpdate")
            .field("password", &"****")
            .field("avatar", &self.avatar)
            .field("admin", &self.admin)
            .finish()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UserView {
    id: i64,
    username: String,
    avatar: Option<i64>,
    admin: bool,
}

impl From<UserView> for User {
    fn from(value: UserView) -> User {
        User {
            id: UserId::from_db(value.id),
            username: Username::new_uncheked(value.username),
            avatar: value.avatar.map(ImageId::from_db),
            admin: value.admin,
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<User>> {
    let views = db::list::<UserView>(db, "user", params).await?;
    Ok(views.into_iter().map(User::from).collect())
}

#[tracing::instrument(skip(db))]
pub async fn create(db: &mut DbC, create: UserCreate) -> Result<User> {
    validate_password(&create.password)?;
    let username = create.username.as_str();
    let password_hash = generate_initial_salt_and_hash(&create.password)?;
    let avatar_id = create.avatar.map(|id| id.to_db());
    let user_id = sqlx::query_scalar(
        r#"
INSERT INTO user (username, password_hash, avatar, admin)
VALUES (?, ?, ?, ?) RETURNING id
"#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(avatar_id)
    .bind(create.admin)
    .fetch_one(&mut *db)
    .await?;
    get(db, UserId::from_db(user_id)).await
}

#[tracing::instrument(skip(db))]
pub async fn get(db: &mut DbC, user_id: UserId) -> Result<User> {
    let user_view =
        sqlx::query_as::<_, UserView>("SELECT id, username, avatar, admin FROM user WHERE id = ?")
            .bind(user_id)
            .fetch_one(db)
            .await?;
    Ok(User::from(user_view))
}

#[tracing::instrument(skip(db))]
pub async fn update(db: &mut DbC, user_id: UserId, update: UserUpdate) -> Result<User> {
    tracing::info!("updating user {} with {:#?}", user_id, update);

    let password_update = match update.password {
        ValueUpdate::Set(password) => {
            let hash = generate_initial_salt_and_hash(&password)?;
            ValueUpdate::Set(hash)
        }
        ValueUpdate::Unset => return Err(Error::new(ErrorKind::Invalid, "cannot unset password")),
        ValueUpdate::Unchanged => ValueUpdate::Unchanged,
    };

    db::value_update_string_non_null(db, "user", "password_hash", user_id, password_update).await?;
    db::value_update_id_nullable(db, "user", "avatar", user_id, update.avatar).await?;
    db::value_update_bool_non_null(db, "user", "admin", user_id, update.admin).await?;

    get(db, user_id).await
}

#[tracing::instrument(skip(db))]
pub async fn lookup(db: &mut DbC, username: &Username) -> Result<Option<UserId>> {
    let id = sqlx::query_scalar("SELECT id FROM user WHERE username = ?")
        .bind(username.as_str())
        .fetch_optional(db)
        .await?;
    Ok(id.map(UserId::from_db))
}

#[tracing::instrument(skip(db))]
pub async fn delete(db: &mut DbC, user_id: UserId) -> Result<()> {
    sqlx::query("DELETE FROM user WHERE id = ?")
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn authenticate(db: &mut DbC, username: &Username, password: &str) -> Result<UserId> {
    let row = sqlx::query("SELECT id, password_hash FROM user WHERE username = ?")
        .bind(username.as_str())
        .fetch_optional(db)
        .await?;
    match row {
        Some(row) => {
            let id = row.get::<i64, _>(0);
            let password_hash = row.get::<String, _>(1);
            verify_password(&password_hash, password)?;
            Ok(UserId::from_db(id))
        }
        None => Err(Error::new(
            ErrorKind::Unauthorized,
            "invalid username or password",
        )),
    }
}

fn generate_initial_salt_and_hash(password: &str) -> Result<String> {
    use scrypt::password_hash::PasswordHasher;
    let salt =
        scrypt::password_hash::SaltString::generate(&mut scrypt::password_hash::rand_core::OsRng);
    let password_hash = scrypt::Scrypt
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| Error::with_source(ErrorKind::Internal, "failed to hash password", e))?;
    Ok(password_hash.to_string())
}

fn verify_password(password_hash: &str, password: &str) -> Result<()> {
    use scrypt::password_hash::PasswordVerifier;

    let parsed_hash =
        scrypt::password_hash::PasswordHash::new(password_hash).expect("invalid password hash");
    scrypt::Scrypt
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| Error::with_source(ErrorKind::Unauthorized, "invalid password", e))
}

fn validate_password(password: &str) -> Result<()> {
    if password.len() < PASSWORD_MIN_LENGTH {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "password is too short (minimum length is {} characters)",
                PASSWORD_MIN_LENGTH
            ),
        ));
    }

    if password.len() > PASSWORD_MAX_LENGTH {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "password is too long (maximum length is {} characters)",
                PASSWORD_MAX_LENGTH
            ),
        ));
    }

    if !password.is_ascii() {
        return Err(Error::new(ErrorKind::Invalid, "password is not ASCII"));
    }

    Ok(())
}
