use crate::{Genre, PropertyKey};

pub const DESCRIPTION: PropertyKey = PropertyKey::new_const("sonar.io/description");
pub const RELEASE_DATE: PropertyKey = PropertyKey::new_const("sonar.io/release-date");

pub const TRACK_NUMBER: PropertyKey = PropertyKey::new_const("sonar.io/track-number");
pub const DISC_NUMBER: PropertyKey = PropertyKey::new_const("sonar.io/disc-number");

pub const EXTERNAL_SPOTIFY_ID: PropertyKey = PropertyKey::new_const("external.sonar.io/spotify-id");
pub const EXTERNAL_MUSICBRAINZ_ID: PropertyKey =
    PropertyKey::new_const("external.sonar.io/musicbrainz-id");

pub fn genre_key(genre: &Genre) -> PropertyKey {
    PropertyKey::new(format!("sonar.io/genre/{}", genre.as_str()))
        .expect("invalid property key for genre")
}
