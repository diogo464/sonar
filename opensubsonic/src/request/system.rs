//! System requests.

use serde::{Deserialize, Serialize};
use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};

/// <http://www.subsonic.org/pages/api.jsp#ping>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct Ping;

/// <http://www.subsonic.org/pages/api.jsp#getLicense>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToQuery, FromQuery, SubsonicRequest)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[serde(rename_all = "camelCase")]
pub struct GetLicense;
