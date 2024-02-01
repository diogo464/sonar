use crate::{db::DbC, Error, ErrorKind, ImageId, ListParams, Result, UserId, ValueUpdate};

mod username;
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
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub username: Username,
    pub password: String,
    pub avatar: Option<ImageId>,
}

#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub password: ValueUpdate<String>,
    pub avatar: ValueUpdate<ImageId>,
}

#[derive(Debug, sqlx::FromRow)]
struct UserView {
    id: i64,
    username: String,
    avatar: Option<i64>,
}

impl UserView {
    fn into_user(self) -> User {
        User {
            id: UserId::from_db(self.id),
            username: Username::new_uncheked(self.username),
            avatar: self.avatar.map(ImageId::from_db),
        }
    }
}

#[tracing::instrument]
pub async fn list(db: &mut DbC, params: ListParams) -> Result<Vec<User>> {
    let (offset, limit) = params.to_db_offset_limit();
    let views = sqlx::query_as!(
        UserView,
        "SELECT id, username, avatar FROM user ORDER BY id ASC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    let mut users = Vec::with_capacity(views.len());
    for view in views {
        users.push(view.into_user());
    }

    Ok(users)
}

#[tracing::instrument]
pub async fn create(db: &mut DbC, create: UserCreate) -> Result<User> {
    validate_password(&create.password)?;
    let username = create.username.as_str();
    let password_hash = generate_initial_salt_and_hash(&create.password)?;
    let avatar_id = create.avatar.map(|id| id.to_db());
    let user_id = sqlx::query!(
        r#"
INSERT INTO user (username, password_hash, avatar)
VALUES (?, ?, ?) RETURNING id
"#,
        username,
        password_hash,
        avatar_id
    )
    .fetch_one(&mut *db)
    .await?
    .id;
    get(db, UserId::from_db(user_id)).await
}

#[tracing::instrument]
pub async fn get(db: &mut DbC, user_id: UserId) -> Result<User> {
    let user_id = user_id.to_db();
    let user_view = sqlx::query_as!(
        UserView,
        "SELECT id, username, avatar FROM user WHERE id = ?",
        user_id
    )
    .fetch_one(db)
    .await?;
    Ok(user_view.into_user())
}

#[tracing::instrument]
pub async fn delete(db: &mut DbC, user_id: UserId) -> Result<()> {
    let user_id = user_id.to_db();
    sqlx::query!("DELETE FROM user WHERE id = ?", user_id)
        .execute(db)
        .await?;
    Ok(())
}

#[tracing::instrument]
pub async fn authenticate(db: &mut DbC, username: &Username, password: &str) -> Result<UserId> {
    let username = username.as_str();
    let row = sqlx::query!(
        "SELECT id, password_hash FROM user WHERE username = ?",
        username
    )
    .fetch_one(db)
    .await?;
    verify_password(&row.password_hash, password)?;
    Ok(UserId::from_db(row.id))
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
