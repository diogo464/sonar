use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

mod property_key;
pub use property_key::*;

mod property_value;
pub use property_value::*;

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

    pub(crate) fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("failed to serialize properties")
    }

    pub(crate) fn deserialize(value: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(value)
    }

    pub(crate) fn deserialize_unchecked(value: &[u8]) -> Self {
        Self::deserialize(value).expect("failed to deserialize properties")
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
