use serde::{Deserialize, Serialize};
use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};

/// Returns the current status for media library scanning. Takes no extra parameters.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#getScanStatus>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct GetScanStatus;

/// Initiates a rescan of the media libraries. Takes no extra parameters.
///
/// For more information, see <http://www.subsonic.org/pages/api.jsp#startScan>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct StartScan;
