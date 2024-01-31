use std::{borrow::Cow, collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

mod property_key;
pub use property_key::*;

mod property_value;
pub use property_value::*;

mod property_update;
pub use property_update::*;

use crate::{db::DbC, Result, SonarIdentifier};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

    pub fn get_parsed<T>(&self, key: impl AsRef<str>) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let value = self.get(key.as_ref())?;
        match T::from_str(value.as_ref()) {
            Ok(value) => Some(value),
            Err(err) => {
                tracing::warn!(
                    "property {} existed but failed to parse: {}",
                    key.as_ref(),
                    err
                );
                None
            }
        }
    }

    pub fn insert(&mut self, key: PropertyKey, value: PropertyValue) -> Option<PropertyValue> {
        self.0.insert(key.0, value.0).map(PropertyValue)
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Option<PropertyValue> {
        self.0.remove(key.as_ref()).map(PropertyValue)
    }

    pub fn keys(&self) -> impl Iterator<Item = PropertyKey> + '_ {
        self.0.keys().map(|k| PropertyKey(k.clone()))
    }

    pub fn values(&self) -> impl Iterator<Item = PropertyValue> + '_ {
        self.0.values().map(|v| PropertyValue(v.clone()))
    }

    pub fn into_property_updates(self) -> Vec<PropertyUpdate> {
        self.into_iter()
            .map(|(key, value)| PropertyUpdate::set(key, value))
            .collect()
    }

    pub fn merge(primary: &mut Self, secondary: &Self) {
        for (key, value) in secondary {
            if !primary.contains_key(&key) {
                primary.insert(key.clone(), value.clone());
            }
        }
    }

    pub(crate) fn apply_updates(&mut self, updates: &[PropertyUpdate]) {
        for update in updates {
            match &update.action {
                PropertyUpdateAction::Set(value) => {
                    self.insert(update.key.clone(), value.clone());
                }
                PropertyUpdateAction::Remove => {
                    self.remove(&update.key);
                }
            }
        }
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

pub struct PropertiesIter<'a> {
    inner: std::collections::hash_map::Iter<'a, Cow<'static, str>, Cow<'static, str>>,
}

impl<'a> Iterator for PropertiesIter<'a> {
    type Item = (PropertyKey, PropertyValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (PropertyKey(k.clone()), PropertyValue(v.clone())))
    }
}

impl<'a> IntoIterator for &'a Properties {
    type Item = (PropertyKey, PropertyValue);
    type IntoIter = PropertiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PropertiesIter {
            inner: self.0.iter(),
        }
    }
}

pub async fn set(db: &mut DbC, id: impl SonarIdentifier, properties: &Properties) -> Result<()> {
    clear(db, id).await?;

    let namespace = id.namespace();
    let identifier = id.identifier();
    for (key, value) in properties {
        let key = key.as_str();
        let value = value.as_str();
        sqlx::query!(
            "INSERT INTO property (namespace, identifier, key, value) VALUES (?, ?, ?, ?)",
            namespace,
            identifier,
            key,
            value
        )
        .execute(&mut *db)
        .await?;
    }

    Ok(())
}

pub async fn get(db: &mut DbC, id: impl SonarIdentifier) -> Result<Properties> {
    let namespace = id.namespace();
    let identifier = id.identifier();
    let rows = sqlx::query!(
        "SELECT key, value FROM property WHERE namespace = ? AND identifier = ?",
        namespace,
        identifier
    )
    .fetch_all(&mut *db)
    .await?;
    let mut properties = Properties::with_capacity(rows.len());
    for row in rows {
        properties.insert(PropertyKey(row.key.into()), PropertyValue(row.value.into()));
    }
    Ok(properties)
}

pub async fn get_bulk(
    db: &mut DbC,
    ids: impl Iterator<Item = impl SonarIdentifier>,
) -> Result<Vec<Properties>> {
    let mut properties = Vec::new();
    for id in ids {
        properties.push(get(db, id).await?);
    }
    Ok(properties)
}

pub async fn update(
    db: &mut DbC,
    id: impl SonarIdentifier,
    updates: &[PropertyUpdate],
) -> Result<()> {
    let namespace = id.namespace();
    let identifier = id.identifier();
    for update in updates {
        match &update.action {
            PropertyUpdateAction::Set(value) => {
                let key = update.key.as_str();
                let value = value.as_str();
                sqlx::query!(
                    "INSERT OR REPLACE INTO property (namespace, identifier, key, value) VALUES (?, ?, ?, ?)",
                    namespace,
                    identifier,
                    key,
                    value
                )
                .execute(&mut *db)
                .await?;
            }
            PropertyUpdateAction::Remove => {
                let key = update.key.as_str();
                sqlx::query!(
                    "DELETE FROM property WHERE namespace = ? AND identifier = ? AND key = ?",
                    namespace,
                    identifier,
                    key
                )
                .execute(&mut *db)
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn clear(db: &mut DbC, id: impl SonarIdentifier) -> Result<()> {
    let namespace = id.namespace();
    let identifier = id.identifier();
    sqlx::query!(
        "DELETE FROM property WHERE namespace = ? AND identifier = ?",
        namespace,
        identifier
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}
