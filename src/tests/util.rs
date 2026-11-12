use std::{path::Path, sync::LazyLock};

#[cfg(test)]
#[macro_export]
macro_rules! assert_is_err {
    ($arg:expr) => {
        assert!($arg.is_err())
    };
}

pub(super) const TEST_DIR: LazyLock<&Path> = LazyLock::new(|| Path::new("/tmp/ywpm-test/"));