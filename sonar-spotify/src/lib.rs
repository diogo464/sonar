#![feature(let_chains)]

mod convert;
mod external;
mod metadata;

pub use external::SpotifyService;
pub use metadata::SpotifyMetadata;
use sonar::Genres;
pub use spotdl::{session::LoginCredentials, Resource, ResourceId, SpotifyId};

fn convert_genres(genres: Vec<String>) -> Genres {
    let mut out = Genres::default();
    for genre in genres {
        match sonar::Genre::canonicalize(genre) {
            Ok(genre) => out.set(&genre),
            Err(err) => {
                tracing::warn!("failed to canonicalize genre: {}", err);
            }
        }
    }
    out
}
