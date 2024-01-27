#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueUpdate<T> {
    Set(T),
    Unset,
    #[default]
    Unchanged,
}

impl<T> ValueUpdate<T> {
    pub fn set(value: T) -> Self {
        Self::Set(value)
    }

    pub fn unset() -> Self {
        Self::Unset
    }

    pub fn unchanged() -> Self {
        Self::Unchanged
    }

    /// Creates a new `ValueUpdate` from an `Option`.
    /// If the `Option` is `Some`, the `ValueUpdate` will be `Set`.
    /// If the `Option` is `None`, the `ValueUpdate` will be `Unset`.
    pub fn from_option_unset(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Set(value),
            None => Self::Unset,
        }
    }

    /// Creates a new `ValueUpdate` from an `Option`.
    /// If the `Option` is `Some`, the `ValueUpdate` will be `Set`.
    /// If the `Option` is `None`, the `ValueUpdate` will be `Unchanged`.
    pub fn from_option_unchanged(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Set(value),
            None => Self::Unchanged,
        }
    }
}
