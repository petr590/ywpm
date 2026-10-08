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
        Self {
            width: width.into(),
            height: height.into(),
        }
    }

    pub fn ratio_equals(&self, other: &Resolution) -> bool {
        self.width * other.height == self.height * other.width
    }

    pub fn ratio_str(&self) -> String {
        let gcd = gcd(self.width, self.height);

        let w = if gcd > 1 { self.width  / gcd } else { self.width };
        let h = if gcd > 1 { self.height / gcd } else { self.height };

        format!("{w}:{h}")
    }

    pub fn calculate_difference_in_percent(&self, other: &Resolution) -> (u64, bool) {
        let a = self.width * other.height;
        let b = self.height * other.width;

        if a > b { // self wider than other
            (div_round(100 * (a - b), b), true)
        } else {
            (div_round(100 * (b - a), a), false)
        }
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

fn div_round(a: u64, b: u64) -> u64 {
    (a + b / 2) / b
}