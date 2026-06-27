use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FitMode { Cover, Contain, Stretch }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VerticalAlignment { Top, Center, Bottom }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum HorizontalAlignment { Left, Center, Right }


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
            v_align: VerticalAlignment::Center,
            h_align: HorizontalAlignment::Center,
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