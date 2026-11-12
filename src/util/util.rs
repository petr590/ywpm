use std::env;
use std::sync::LazyLock;

use crossterm::terminal;

pub fn get_terminal_width() -> u16 {
    match terminal::size() {
        Ok((w, _h)) => w,
        Err(_) => 80,
    }
}

pub static IS_RU: LazyLock<bool> = LazyLock::new(|| {
    env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("ru")
});
