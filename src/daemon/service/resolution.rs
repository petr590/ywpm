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

        let w = if gcd > 1 {
            self.width / gcd
        } else {
            self.width
        };
        let h = if gcd > 1 {
            self.height / gcd
        } else {
            self.height
        };

        format!("{w}:{h}")
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
