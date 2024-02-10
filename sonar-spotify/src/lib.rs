mod convert;
mod external;
mod metadata;

pub use external::SpotifyService;
pub use metadata::SpotifyMetadata;
pub use spotdl::{session::LoginCredentials, Resource, ResourceId, SpotifyId};
