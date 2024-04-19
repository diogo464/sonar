use serde::{Deserialize, Serialize};
use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};

///Returns all internet radio stations. Takes no extra parameters.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#getInternetRadioStations>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct GetInternetRadioStations;

/// Adds a new internet radio station.
/// Only users with admin privileges are allowed to call this method.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#createInternetRadioStation>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct CreateInternetRadioStation {
    /// The stream URL for the station.
    pub stream_url: String,
    /// The user-defined name for the station.
    pub name: String,
    /// The home page URL for the station.
    pub homepage_url: Option<String>,
}

/// Updates an existing internet radio station.
/// Only users with admin privileges are allowed to call this method.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#updateInternetRadioStation>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternetRadioStation {
    /// The ID for the station.
    pub id: String,
    /// The stream URL for the station.
    pub stream_url: String,
    /// The user-defined name for the station.
    pub name: String,
    /// The home page URL for the station.
    pub homepage_url: Option<String>,
}

/// Deletes an existing internet radio station.
/// Only users with admin privileges are allowed to call this method.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#deleteInternetRadioStation>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct DeleteInternetRadioStation {
    /// The ID for the station.
    pub id: String,
}
