use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::state::{Alignment, DisplayMode, DisplayModeParseError, FitMode};

#[derive(Serialize, Deserialize)]
struct DisplayModeProxy {
    #[serde(with = "crate::state::display_mode_format")]
    mode: DisplayMode,
}

#[test]
fn display_mode_serializes() -> Result<(), yaml_serde::Error> {
    let mode = DisplayMode::new();
    assert_eq!(mode.to_string(), "cover center");

    let proxy = DisplayModeProxy { mode };
    assert_eq!(yaml_serde::to_string(&proxy)?.trim(), "mode: cover center");

    Ok(())
}

const CONTAIN_TOP:    DisplayMode = DisplayMode::new_with_fields(FitMode::Contain, Alignment::Top);
const CONTAIN_RIGHT:  DisplayMode = DisplayMode::new_with_fields(FitMode::Contain, Alignment::Right);
const CONTAIN_CENTER: DisplayMode = DisplayMode::new_with_fields(FitMode::Contain, Alignment::Center);
const STRETCH_CENTER: DisplayMode = DisplayMode::new_with_fields(FitMode::Stretch, Alignment::Center);

#[test]
fn display_mode_to_string() {
    assert_eq!(CONTAIN_TOP.to_string(),    "contain top");
    assert_eq!(CONTAIN_RIGHT.to_string(),  "contain right");
    assert_eq!(CONTAIN_CENTER.to_string(), "contain center");
    assert_eq!(STRETCH_CENTER.to_string(), "stretch center");
}

#[test]
fn display_mode_parses() -> Result<(), DisplayModeParseError> {
    assert_eq!(DisplayMode::from_str("contain top")?,           CONTAIN_TOP);
    assert_eq!(DisplayMode::from_str("contain right")?,         CONTAIN_RIGHT);
    assert_eq!(DisplayMode::from_str("contain center")?,        CONTAIN_CENTER);
    assert_eq!(DisplayMode::from_str("top contain   center ")?, CONTAIN_CENTER);
    Ok(())
}
