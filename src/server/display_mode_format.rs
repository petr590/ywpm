use serde::{Deserialize, Deserializer, Serializer};

use crate::server::display_mode::DisplayMode;


pub fn serialize<S>(mode: &DisplayMode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    serializer.serialize_str(mode.to_string().as_str())
}


pub fn deserialize<'de, D>(deserializer: D) -> Result<DisplayMode, D::Error>
where
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
