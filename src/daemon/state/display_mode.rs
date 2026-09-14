use std::{error::Error, fmt};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FitMode {
    Cover,
    Contain,
    Stretch,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AlignX {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AlignY {
    Top,
    Center,
    Bottom,
}

#[derive(Debug)]
pub struct DisplayModeParseError;

impl Error for DisplayModeParseError {}

impl fmt::Display for DisplayModeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DisplayModeParseError")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DisplayMode {
    pub fit_mode: FitMode,
    pub align_x: AlignX,
    pub align_y: AlignY,
}

impl DisplayMode {
    pub const fn new() -> Self {
        Self {
            fit_mode: FitMode::Cover,
            align_x: AlignX::Center,
            align_y: AlignY::Center,
        }
    }

    pub const fn new_with_fields(fit_mode: FitMode, align_x: AlignX, align_y: AlignY) -> Self {
        Self {
            fit_mode,
            align_x,
            align_y,
        }
    }

    pub fn is_default(&self) -> bool {
        self.fit_mode == FitMode::Cover
            && self.align_x == AlignX::Center
            && self.align_y == AlignY::Center
    }

    pub const fn fit_mode(mut self, fit_mode: FitMode) -> Self {
        self.fit_mode = fit_mode;
        self
    }

    pub const fn align_x(mut self, align_x: AlignX) -> Self {
        self.align_x = align_x;
        self
    }

    pub const fn align_y(mut self, align_y: AlignY) -> Self {
        self.align_y = align_y;
        self
    }
}

impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let center = self.align_x == AlignX::Center && self.align_y == AlignY::Center;

        let s = if center {
            format!("{:?} center", self.fit_mode)
        } else {
            format!("{:?} {:?} {:?}", self.fit_mode, self.align_x, self.align_y)
        };

        write!(f, "{}", s.to_ascii_lowercase())
    }
}

impl FromStr for DisplayMode {
    type Err = DisplayModeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mode = DisplayMode::new();

        for token in s.to_ascii_lowercase().split_whitespace() {
            match token {
                "cover" => mode.fit_mode = FitMode::Cover,
                "contain" => mode.fit_mode = FitMode::Contain,
                "stretch" => mode.fit_mode = FitMode::Stretch,

                "left" => mode.align_x = AlignX::Left,
                "right" => mode.align_x = AlignX::Right,

                "top" => mode.align_y = AlignY::Top,
                "bottom" => mode.align_y = AlignY::Bottom,

                "center" => {} // Do nothing because defaults are center

                _ => return Err(DisplayModeParseError),
            }
        }

        Ok(mode)
    }
}
