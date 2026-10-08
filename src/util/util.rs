use std::env;

use crossterm::terminal;
use once_cell::sync::Lazy;

pub static IS_RU: Lazy<bool> = Lazy::new(|| {
    env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("ru")
});

pub fn get_terminal_width() -> u16 {
    match terminal::size() {
        Ok((width, _height)) => width,
        Err(_) => 80,
    }
}

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
