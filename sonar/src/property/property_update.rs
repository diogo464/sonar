use super::{PropertyKey, PropertyValue};

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
    pub fn from_option(key: PropertyKey, value: Option<PropertyValue>) -> Self {
        Self {
            key,
            action: match value {
                Some(value) => PropertyUpdateAction::Set(value),
                None => PropertyUpdateAction::Remove,
            },
        }
    }

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
