use std::fmt;

#[derive(PartialEq)]
pub(crate) struct Resolution {
    pub width: u64,
    pub height: u64,
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl Resolution {
    pub fn new(width: impl Into<u64>, height: impl Into<u64>) -> Self {
        Self { width: width.into(), height: height.into() }
    }

    pub fn ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn ratio_str(&self) -> String {
        let mut w = self.width;
        let mut h = self.height;

        let max = (w.min(h) as f64).sqrt().ceil() as u64;

        for d in 2 .. max {
            while w % d == 0 &&
                  h % d == 0 {
                w /= d;
                h /= d;
            }
        }

        format!("{w}:{h}")
    }
}


pub(crate) struct FileInfo {
    pub resolution: Resolution,
    pub is_video: bool,
}