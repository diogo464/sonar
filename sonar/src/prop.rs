use crate::{Genre, PropertyKey};

pub const DESCRIPTION: PropertyKey = PropertyKey::new_const("sonar.io/description");
pub const RELEASE_DATE: PropertyKey = PropertyKey::new_const("sonar.io/release-date");

pub const TRACK_NUMBER: PropertyKey = PropertyKey::new_const("sonar.io/track-number");
pub const DISC_NUMBER: PropertyKey = PropertyKey::new_const("sonar.io/disc-number");

pub const EXTERNAL_SPOTIFY_ID: PropertyKey = PropertyKey::new_const("external.sonar.io/spotify-id");
pub const EXTERNAL_MUSICBRAINZ_ID: PropertyKey =
    PropertyKey::new_const("external.sonar.io/musicbrainz-id");
// https://en.wikipedia.org/wiki/International_Standard_Recording_Code
pub const EXTERNAL_ISRC: PropertyKey = PropertyKey::new_const("external.sonar.io/isrc");
// https://en.wikipedia.org/wiki/International_Article_Number
pub const EXTERNAL_EAN: PropertyKey = PropertyKey::new_const("external.sonar.io/ean");
// https://en.wikipedia.org/wiki/Universal_Product_Code
pub const EXTERNAL_UPC: PropertyKey = PropertyKey::new_const("external.sonar.io/upc");

pub fn genre_key(genre: &Genre) -> PropertyKey {
    PropertyKey::new(format!("sonar.io/genre/{}", genre.as_str()))
        .expect("invalid property key for genre")
}
