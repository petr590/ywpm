#[macro_export]
macro_rules! localized {
    ($en_expr:expr, $ru_expr:expr $(,)?) => {
        if !*crate::util::IS_RU {
            $en_expr
        } else {
            $ru_expr
        }
    };
}

#[macro_export]
macro_rules! format_localized {
    ($en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::localized!(
            format!($en_fmt $(, $arg)*),
            format!($ru_fmt $(, $arg)*),
        )
    };
}


#[macro_export]
macro_rules! writeln_localized {
    ($stream:expr, $en_fmt:expr, $ru_fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::localized!(
            writeln!($stream, $en_fmt $(, $arg)*),
            writeln!($stream, $ru_fmt $(, $arg)*),
        )
    };
}

#[macro_export]
macro_rules! nzu16 {
    ($n:expr) => {
        const {
            match std::num::NonZeroU16::new($n) {
                Some(n) => n,
                None => panic!("Value cannot be zero!"),
            }
        }
    };
}