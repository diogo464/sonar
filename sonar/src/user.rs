use std::{borrow::Cow, str::FromStr};

use crate::{DbC, Error, ErrorKind, ImageId, Result, User, UserCreate, UserId};

const USERNAME_MAX_LENGTH: usize = 24;
const PASSWORD_MIN_LENGTH: usize = 8;
const PASSWORD_MAX_LENGTH: usize = 48;

#[derive(Debug)]
pub struct InvalidUsernameError {
    message: &'static str,
}

impl InvalidUsernameError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for InvalidUsernameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a valid username", self.message)
    }
}

impl std::error::Error for InvalidUsernameError {}

/// A username.
///
/// A username is an ASCII string of up to 24 lowercase characters.
/// Only the characters `a-z`, `A-Z`, `0-9`, `_`, `-` and `.` are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(Cow<'static, str>);

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Username {
    type Err = InvalidUsernameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > USERNAME_MAX_LENGTH {
            return Err(InvalidUsernameError::new("username is too long"));
        }

        if !s.is_ascii() {
            return Err(InvalidUsernameError::new("username is not ASCII"));
        }

        if !s.bytes().all(is_valid_username_char) {
            return Err(InvalidUsernameError::new(
                "username contains invalid characters",
            ));
        }

        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl Username {
    pub fn new(username: impl AsRef<str>) -> Result<Self, InvalidUsernameError> {
        Self::from_str(username.as_ref())
    }

    pub fn new_uncheked(username: impl Into<Cow<'static, str>>) -> Self {
        Self::from_str(username.into().as_ref()).expect("invalid username")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_username_char(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.')
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
            avatar: self.avatar.map(|id| ImageId::from_db(id)),
        }
    }
}

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

pub async fn delete(db: &mut DbC, user_id: UserId) -> Result<()> {
    let user_id = user_id.to_db();
    sqlx::query!("DELETE FROM user WHERE id = ?", user_id)
        .execute(db)
        .await?;
    Ok(())
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
