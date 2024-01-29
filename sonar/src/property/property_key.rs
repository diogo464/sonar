use std::{borrow::Cow, str::FromStr};

use serde::{Deserialize, Serialize};

const PROPERTY_KEY_MAX_LENGTH: usize = 64;

#[derive(Debug)]
pub struct InvalidPropertyKeyError {
    key: Cow<'static, str>,
    message: &'static str,
}

impl InvalidPropertyKeyError {
    fn new(key: impl Into<Cow<'static, str>>, message: &'static str) -> Self {
        Self {
            key: key.into(),
            message,
        }
    }
}

impl std::fmt::Display for InvalidPropertyKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`{}` is not a valid property key: {}",
            self.key, self.message
        )
    }
}

impl std::error::Error for InvalidPropertyKeyError {}

/// A property key.
///
/// A property key is a lower case asccii string with a maximum length of 64 characters.
/// Only the characters `a-z`, `0-9`, `-`, `_` , `/` and `.` are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyKey(pub(super) Cow<'static, str>);

impl FromStr for PropertyKey {
    type Err = InvalidPropertyKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(error_message) = property_key_check(s) {
            return Err(InvalidPropertyKeyError::new(s.to_owned(), error_message));
        }
        Ok(Self(From::from(s.to_owned())))
    }
}

impl std::fmt::Display for PropertyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for PropertyKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PropertyKey> for String {
    fn from(key: PropertyKey) -> Self {
        key.0.into_owned()
    }
}

impl PropertyKey {
    pub fn new(key: impl AsRef<str>) -> Result<Self, InvalidPropertyKeyError> {
        Self::from_str(key.as_ref())
    }

    pub const fn new_const(key: &'static str) -> Self {
        if let Some(_) = property_key_check(key) {
            panic!("invalid property key");
        }
        Self(Cow::Borrowed(key))
    }

    pub fn new_uncheked(key: impl Into<Cow<'static, str>>) -> Self {
        Self::from_str(key.into().as_ref()).expect("invalid property key")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const fn property_key_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '/' || c == '.'
}

const fn property_key_check(s: &str) -> Option<&'static str> {
    if s.len() > PROPERTY_KEY_MAX_LENGTH {
        return Some("property key is too long");
    }

    if !s.is_ascii() {
        return Some("property key is not ASCII");
    }

    let bytes = s.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !property_key_char(bytes[index] as char) {
            return Some("property key contains invalid characters");
        }
        index += 1;
    }

    None
}
