use std::{borrow::Cow, str::FromStr};

use crate::Result;

#[derive(Debug)]
pub struct InvalidGenreError {
    message: Cow<'static, str>,
}

impl InvalidGenreError {
    fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for InvalidGenreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a valid genre", self.message)
    }
}

impl std::error::Error for InvalidGenreError {}

/// A genre of music.
///
/// A genre is an ASCII string of up to 24 lowercase characters.
/// Only the characters `a-z`, `0-9` and ` ` are allowed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Genre([u8; 24]);

impl std::fmt::Debug for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Genre").field(&self.as_str()).finish()
    }
}

impl FromStr for Genre {
    type Err = InvalidGenreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 24 {
            return Err(InvalidGenreError::new("genre is too long"));
        }

        if !s.is_ascii() {
            return Err(InvalidGenreError::new("genre is not ASCII"));
        }

        if !s.bytes().all(is_valid_genre_char) {
            return Err(InvalidGenreError::new("genre contains invalid characters"));
        }

        let mut genre = [0; 24];
        genre[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self(genre))
    }
}

const fn is_valid_genre_char(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b' ')
}

impl std::fmt::Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Genre {
    pub fn new(genre: impl AsRef<str>) -> Result<Self, InvalidGenreError> {
        Self::from_str(genre.as_ref())
    }

    pub fn new_unchecked(genre: impl AsRef<str>) -> Self {
        Self::from_str(genre.as_ref()).expect("invalid genre")
    }

    pub fn as_bytes(&self) -> &[u8] {
        let first_zero = self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len());
        &self.0[..first_zero]
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).expect("genre is not valid UTF-8")
    }
}

#[derive(Debug, Default, Clone, Hash)]
pub struct Genres(Vec<Genre>);

impl Genres {
    pub fn new(
        genres: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, InvalidGenreError> {
        let mut genres = genres
            .into_iter()
            .map(|genre| Genre::from_str(genre.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        genres.sort_unstable();
        genres.dedup();
        Ok(Self(genres))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn set(&mut self, genre: &Genre) {
        self.0.push(*genre);
    }

    pub fn unset(&mut self, genre: &Genre) {
        self.0.retain(|g| g != genre);
    }

    pub fn contains(&self, genre: &Genre) -> bool {
        self.0.contains(genre)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Genre> {
        self.0.iter()
    }
}

impl std::ops::Deref for Genres {
    type Target = [Genre];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Genres {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Index<usize> for Genres {
    type Output = Genre;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Genres {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl IntoIterator for Genres {
    type Item = Genre;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'s> IntoIterator for &'s Genres {
    type Item = &'s Genre;
    type IntoIter = std::slice::Iter<'s, Genre>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::fmt::Display for Genres {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut genres = self.0.iter();
        if let Some(genre) = genres.next() {
            write!(f, "{}", genre)?;
            for genre in genres {
                write!(f, ",{}", genre)?;
            }
        }
        Ok(())
    }
}

impl FromStr for Genres {
    type Err = InvalidGenreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.split(','))
    }
}

impl TryFrom<Vec<String>> for Genres {
    type Error = InvalidGenreError;

    fn try_from(genres: Vec<String>) -> Result<Self, Self::Error> {
        Self::new(genres)
    }
}

impl From<Genres> for Vec<String> {
    fn from(genres: Genres) -> Self {
        genres.into_iter().map(|genre| genre.to_string()).collect()
    }
}

impl From<Genre> for Genres {
    fn from(genre: Genre) -> Self {
        Self(vec![genre])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenreUpdateAction {
    Set,
    Unset,
}

#[derive(Debug, Clone)]
pub struct GenreUpdate {
    pub genre: Genre,
    pub action: GenreUpdateAction,
}

impl GenreUpdate {
    pub fn set(genre: Genre) -> Self {
        Self {
            genre,
            action: GenreUpdateAction::Set,
        }
    }

    pub fn unset(genre: Genre) -> Self {
        Self {
            genre,
            action: GenreUpdateAction::Unset,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn genre_parse() {
        "heavy metal".parse::<Genre>().unwrap();
        "electronic".parse::<Genre>().unwrap();

        "Heavy Metal".parse::<Genre>().unwrap_err();
        "EDM".parse::<Genre>().unwrap_err();
    }

    #[test]
    fn genre_as_str() {
        let genre = Genre::new_unchecked("heavy metal");
        assert_eq!(genre.as_str(), "heavy metal");
    }

    #[test]
    fn genres_display() {
        let genres = Genres::new(vec!["heavy metal", "electronic"]).unwrap();
        assert_eq!(genres.to_string(), "electronic,heavy metal");
    }
}
