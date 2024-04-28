//! System requests.

use opensubsonic_macro::{FromQuery, SubsonicRequest, ToQuery};
use serde::{Deserialize, Serialize};

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
