use std::{borrow::Cow, collections::HashMap, str::FromStr};

use crate::{DbC, Result};

const PROPERTY_KEY_MAX_LENGTH: usize = 64;
const PROPERT_VALUE_MAX_LENGTH: usize = 128;

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

/// A property key.
///
/// A property key is a lower case asccii string with a maximum length of 64 characters.
/// Only the characters `a-z`, `0-9`, `-`, `_` , `/` and `.` are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyKey(Cow<'static, str>);

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

/// A property value.
///
/// A property value is an ascii string with a maximum length of 128 characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyValue(Cow<'static, str>);

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

#[derive(Debug, Default, Clone)]
pub struct Properties(HashMap<Cow<'static, str>, Cow<'static, str>>);

impl Properties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.0.contains_key(key.as_ref())
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<PropertyValue> {
        self.0.get(key.as_ref()).map(|v| PropertyValue(v.clone()))
    }

    pub fn insert(&mut self, key: PropertyKey, value: PropertyValue) -> Option<PropertyValue> {
        self.0.insert(key.0, value.0).map(|v| PropertyValue(v))
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Option<PropertyValue> {
        self.0.remove(key.as_ref()).map(|v| PropertyValue(v))
    }

    pub fn keys(&self) -> impl Iterator<Item = PropertyKey> + '_ {
        self.0.keys().map(|k| PropertyKey(k.clone()))
    }

    pub fn values(&self) -> impl Iterator<Item = PropertyValue> + '_ {
        self.0.values().map(|v| PropertyValue(v.clone()))
    }
}

pub struct PropertiesIntoIter {
    inner: std::collections::hash_map::IntoIter<Cow<'static, str>, Cow<'static, str>>,
}

impl Iterator for PropertiesIntoIter {
    type Item = (PropertyKey, PropertyValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (PropertyKey(k), PropertyValue(v)))
    }
}

impl IntoIterator for Properties {
    type Item = (PropertyKey, PropertyValue);
    type IntoIter = PropertiesIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        PropertiesIntoIter {
            inner: self.0.into_iter(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PropertyUpdateAction {
    Set(PropertyValue),
    Remove,
}

#[derive(Debug, Clone)]
pub struct PropertyUpdate {
    pub key: PropertyKey,
    pub action: PropertyUpdateAction,
}

impl PropertyUpdate {
    pub fn set(key: PropertyKey, value: PropertyValue) -> Self {
        Self {
            key,
            action: PropertyUpdateAction::Set(value),
        }
    }

    pub fn remove(key: PropertyKey) -> Self {
        Self {
            key,
            action: PropertyUpdateAction::Remove,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Artist = 0,
    Album = 1,
    Track = 2,
    Playlist = 3,
    Scrobble = 4,
}

pub async fn set(
    db: &mut DbC,
    namespace: Namespace,
    id: i64,
    properties: &Properties,
) -> Result<()> {
    debug_assert_eq!(id & (crate::id::ID_TYPE_MASK as i64), 0);
    clear(db, namespace, id).await?;

    let namespace = namespace as i64;
    for (key, value) in properties.0.iter() {
        let key = key.as_ref();
        let value = value.as_ref();
        sqlx::query!(
            "INSERT INTO property (namespace, id, key, value) VALUES (?, ?, ?, ?)",
            namespace,
            id,
            key,
            value
        )
        .execute(&mut *db)
        .await?;
    }

    Ok(())
}

pub async fn get(db: &mut DbC, namespace: Namespace, id: i64) -> Result<Properties> {
    debug_assert_eq!(id & (crate::id::ID_TYPE_MASK as i64), 0);
    let namespace = namespace as i64;
    let rows = sqlx::query!(
        "SELECT key, value FROM property WHERE namespace = ? AND id = ?",
        namespace,
        id
    )
    .fetch_all(&mut *db)
    .await?;

    let mut properties = Properties::with_capacity(rows.len());
    for row in rows {
        properties.insert(
            PropertyKey::new_uncheked(row.key),
            PropertyValue::new_uncheked(row.value),
        );
    }

    Ok(properties)
}

pub async fn update(
    db: &mut DbC,
    namespace: Namespace,
    id: i64,
    update: &[PropertyUpdate],
) -> Result<()> {
    debug_assert_eq!(id & (crate::id::ID_TYPE_MASK as i64), 0);
    let namespace = namespace as i64;
    for update in update {
        let key = update.key.as_ref();
        match update.action {
            PropertyUpdateAction::Set(ref value) => {
                let value = value.as_ref();
                sqlx::query!(
                    "INSERT INTO property (namespace, id, key, value) VALUES (?, ?, ?, ?)",
                    namespace,
                    id,
                    key,
                    value
                )
                .execute(&mut *db)
                .await?;
            }
            PropertyUpdateAction::Remove => {
                sqlx::query!(
                    "DELETE FROM property WHERE namespace = ? AND id = ? AND key = ?",
                    namespace,
                    id,
                    key
                )
                .execute(&mut *db)
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn clear(db: &mut DbC, namespace: Namespace, id: i64) -> Result<()> {
    debug_assert_eq!(id & (crate::id::ID_TYPE_MASK as i64), 0);
    let namespace = namespace as i64;
    sqlx::query!(
        "DELETE FROM property WHERE namespace = ? AND id = ?",
        namespace,
        id
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}
