use std::{borrow::Cow, str::FromStr};

use serde::{Deserialize, Serialize};

const PROPERT_VALUE_MAX_LENGTH: usize = 128;

#[derive(Debug)]
pub struct InvalidPropertyValueError {
    value: Cow<'static, str>,
    message: &'static str,
}

impl InvalidPropertyValueError {
    fn new(value: impl Into<Cow<'static, str>>, message: &'static str) -> Self {
        Self {
            value: value.into(),
            message,
        }
    }
}

impl std::fmt::Display for InvalidPropertyValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`{}` is not a valid property value: {}",
            self.value, self.message
        )
    }
}

impl std::error::Error for InvalidPropertyValueError {}

/// A property value.
///
/// A property value is an ascii string with a maximum length of 128 characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyValue(pub(super) Cow<'static, str>);

impl FromStr for PropertyValue {
    type Err = InvalidPropertyValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > PROPERT_VALUE_MAX_LENGTH {
            return Err(InvalidPropertyValueError::new(
                s.to_owned(),
                "property value is too long",
            ));
        }

        if !s.is_ascii() {
            return Err(InvalidPropertyValueError::new(
                s.to_owned(),
                "property value is not ASCII",
            ));
        }

        Ok(Self(From::from(s.to_owned())))
    }
}

impl std::fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for PropertyValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PropertyValue> for String {
    fn from(value: PropertyValue) -> Self {
        value.0.into_owned()
    }
}

impl From<u16> for PropertyValue {
    fn from(value: u16) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl From<i16> for PropertyValue {
    fn from(value: i16) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl From<u32> for PropertyValue {
    fn from(value: u32) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl From<i32> for PropertyValue {
    fn from(value: i32) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl From<u64> for PropertyValue {
    fn from(value: u64) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl From<i64> for PropertyValue {
    fn from(value: i64) -> Self {
        Self::new_uncheked(value.to_string())
    }
}

impl PropertyValue {
    pub fn new(value: impl AsRef<str>) -> Result<Self, InvalidPropertyValueError> {
        Self::from_str(value.as_ref())
    }

    pub fn new_uncheked(value: impl Into<Cow<'static, str>>) -> Self {
        Self::from_str(value.into().as_ref()).expect("invalid property value")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
