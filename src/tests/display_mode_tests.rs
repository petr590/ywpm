use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::daemon::state::{AlignX, AlignY, DisplayMode, DisplayModeParseError, FitMode};

#[derive(Serialize, Deserialize)]
struct DisplayModeProxy {
    #[serde(with = "crate::daemon::state::display_mode_format")]
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

const CONTAIN_LEFT_TOP: DisplayMode =
    DisplayMode::new_with_fields(FitMode::Contain, AlignX::Left, AlignY::Top);
const CONTAIN_RIGHT_CENTER: DisplayMode =
    DisplayMode::new_with_fields(FitMode::Contain, AlignX::Right, AlignY::Center);
const CONTAIN_CENTER_TOP: DisplayMode =
    DisplayMode::new_with_fields(FitMode::Contain, AlignX::Center, AlignY::Top);
const STRETCH_CENTER: DisplayMode = DisplayMode::new().fit_mode(FitMode::Stretch);

#[test]
fn display_mode_to_string() {
    assert_eq!(CONTAIN_LEFT_TOP.to_string(), "contain left top");
    assert_eq!(CONTAIN_RIGHT_CENTER.to_string(), "contain right center");
    assert_eq!(CONTAIN_CENTER_TOP.to_string(), "contain center top");
    assert_eq!(STRETCH_CENTER.to_string(), "stretch center");
}

#[test]
fn display_mode_parses() -> Result<(), DisplayModeParseError> {
    assert_eq!(DisplayMode::from_str("contain top left")?, CONTAIN_LEFT_TOP);
    assert_eq!(
        DisplayMode::from_str("contain right center")?,
        CONTAIN_RIGHT_CENTER
    );
    assert_eq!(
        DisplayMode::from_str("contain center top")?,
        CONTAIN_CENTER_TOP
    );
    assert_eq!(
        DisplayMode::from_str("top contain center ")?,
        CONTAIN_CENTER_TOP
    );
    Ok(())
}
