use core::fmt;
use std::{fmt::Display, str::FromStr};
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FitMode { Cover, Contain, Stretch }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VerticalAlignment { Top, Center, Bottom }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum HorizontalAlignment { Left, Center, Right }


pub struct DisplayModeParseError;

impl Display for DisplayModeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DisplayModeParseError")
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DisplayMode {
    pub fit_mode: FitMode,
    pub h_align: HorizontalAlignment,
    pub v_align: VerticalAlignment,
}

impl DisplayMode {
    pub const fn new() -> Self {
        Self {
            fit_mode: FitMode::Cover,
            h_align: HorizontalAlignment::Center,
            v_align: VerticalAlignment::Center,
        }
    }

    pub const fn fit_mode(mut self, fit_mode: FitMode) -> Self {
        self.fit_mode = fit_mode;
        self
    }

    pub const fn h_align(mut self, h_align: HorizontalAlignment) -> Self {
        self.h_align = h_align;
        self
    }

    pub const fn v_align(mut self, v_align: VerticalAlignment) -> Self {
        self.v_align = v_align;
        self
    }
}

impl fmt::Display for DisplayMode {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?} {:?} {:?}", self.fit_mode, self.h_align, self.v_align);
        write!(f, "{}", s.to_ascii_lowercase())
    }
}



impl FromStr for DisplayMode {
    type Err = DisplayModeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mode = DisplayMode::new();
        
        for token in s.to_ascii_lowercase().split(" ") {
            match token {
                "cover"   => mode.fit_mode = FitMode::Cover,
                "contain" => mode.fit_mode = FitMode::Contain,
                "stretch" => mode.fit_mode = FitMode::Stretch,

                "left"    => mode.h_align = HorizontalAlignment::Left,
                "hcenter" => mode.h_align = HorizontalAlignment::Center,
                "right"   => mode.h_align = HorizontalAlignment::Right,

                "top"     => mode.v_align = VerticalAlignment::Top,
                "vcenter" => mode.v_align = VerticalAlignment::Center,
                "bottom"  => mode.v_align = VerticalAlignment::Bottom,

                "center" => {
                    mode.h_align = HorizontalAlignment::Center;
                    mode.v_align = VerticalAlignment::Center;
                }

                "" => {}

                _ => return Err(DisplayModeParseError)
            }
        }

        Ok(mode)
    }
}