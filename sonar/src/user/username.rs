use std::{borrow::Cow, str::FromStr};

const USERNAME_MAX_LENGTH: usize = 24;

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

impl From<Username> for String {
    fn from(username: Username) -> Self {
        username.0.into_owned()
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
