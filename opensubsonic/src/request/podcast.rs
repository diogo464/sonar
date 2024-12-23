use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};
use serde::{Deserialize, Serialize};

/// Returns all Podcast channels the server subscribes to, and (optionally) their episodes.
/// This method can also be used to return details for only one channel - refer to the id parameter.
/// A typical use case for this method would be to first retrieve all channels without episodes, and then retrieve all episodes for the single channel the user selects.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#getPodcasts>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct GetPodcasts {
    /// Since 1.9.0
    /// Whether to include Podcast episodes in the returned result.
    pub include_episodes: Option<bool>,
    /// Since 1.9.0
    /// If specified, only return the Podcast channel with this ID.
    pub id: Option<String>,
}

/// Returns the most recently published Podcast episodes.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#getNewestPodcasts>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct GetNewestPodcasts {
    /// The maximum number of episodes to return.
    pub count: Option<u32>,
}

/// Requests the server to check for new Podcast episodes.
/// Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts).
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#refreshPodcasts>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct RefreshPodcasts;

/// Adds a new Podcast channel.
/// Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts).
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#createPodcastChannel>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct CreatePodcastChannel {
    /// The URL of the Podcast to add.
    pub url: String,
}

/// Deletes a Podcast channel.
/// Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts).
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#deletePodcastChannel>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct DeletePodcastChannel {
    /// The ID of the Podcast channel to delete.
    pub id: String,
}

/// Deletes a Podcast episode.
/// Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts).
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#deletePodcastEpisode>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct DeletePodcastEpisode {
    /// The ID of the Podcast episode to delete.
    pub id: String,
}

/// Request the server to start downloading a given Podcast episode.
/// Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts).
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#downloadPodcastEpisode>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct DownloadPodcastEpisode {
    /// The ID of the Podcast episode to download.
    pub id: String,
}
