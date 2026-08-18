#[macro_export]
macro_rules! assert_is_err {
    ($arg:expr) => {
        assert!($arg.is_err())
    };
}