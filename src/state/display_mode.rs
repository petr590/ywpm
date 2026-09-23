use std::{error::Error, fmt};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use names::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FitMode {
    Cover,
    Contain,
    Stretch,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Alignment {
    Center,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DisplayMode {
    pub fit_mode: FitMode,
    pub alignment: Alignment,
}

pub(super) mod names {
    pub(crate) const COVER:   &str = "cover";
    pub(crate) const CONTAIN: &str = "contain";
    pub(crate) const STRETCH: &str = "stretch";
    pub(crate) const LEFT:    &str = "left";
    pub(crate) const RIGHT:   &str = "right";
    pub(crate) const TOP:     &str = "top";
    pub(crate) const BOTTOM:  &str = "bottom";
    pub(crate) const CENTER:  &str = "center";
}


impl DisplayMode {
    pub const fn new() -> Self {
        Self::new_with_fields(FitMode::Cover, Alignment::Center)
    }

    pub const fn new_with_fields(fit_mode: FitMode, alignment: Alignment) -> Self {
        Self { fit_mode, alignment }
    }

    pub fn is_default(&self) -> bool {
        self.fit_mode  == FitMode::Cover &&
        self.alignment == Alignment::Center
    }
}

impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?},{:?}", self.fit_mode, self.alignment);
        write!(f, "{}", s.to_ascii_lowercase())
    }
}

impl FromStr for DisplayMode {
    type Err = DisplayModeParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut mode = DisplayMode::new();

        for token in split_tokens(string) {
            match token.to_lowercase().as_str() {
                COVER   => mode.fit_mode = FitMode::Cover,
                CONTAIN => mode.fit_mode = FitMode::Contain,
                STRETCH => mode.fit_mode = FitMode::Stretch,

                CENTER => mode.alignment = Alignment::Center,
                TOP    => mode.alignment = Alignment::Top,
                BOTTOM => mode.alignment = Alignment::Bottom,
                LEFT   => mode.alignment = Alignment::Left,
                RIGHT  => mode.alignment = Alignment::Right,

                _ => return Err(DisplayModeParseError),
            }
        }

        Ok(mode)
    }
}

pub(super) fn is_separator(c: char) -> bool {
    c == ',' || c.is_whitespace()
}

pub(super) fn split_tokens(string: &str) -> impl Iterator<Item = &str> {
    string
        .split(is_separator)
        .filter(|s| !s.is_empty())
}


#[derive(Debug)]
pub struct DisplayModeParseError;

impl Error for DisplayModeParseError {}

impl fmt::Display for DisplayModeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DisplayModeParseError")
    }
}