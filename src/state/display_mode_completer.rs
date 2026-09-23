use std::ffi::OsStr;

use clap_complete::CompletionCandidate;
use clap_complete::engine::ValueCompleter;

use crate::state::display_mode::{self, names::*};

const NAMES: [&str; 8] = [ COVER, CONTAIN, STRETCH, LEFT, RIGHT, TOP, BOTTOM, CENTER ];

pub(crate) struct DisplayModeCompleter;

impl ValueCompleter for DisplayModeCompleter {

    fn complete(&self, current: &OsStr) -> Vec<CompletionCandidate> {
        let current = unquote(current.to_str().unwrap_or_default());

        let mut tokens = display_mode::split_tokens(&current)
                .collect::<Vec<&str>>();
        
        let last_token = if ends_with_separator(&current) { Some("") } else { tokens.pop() };
        let tokens_except_last = tokens.join(",");

        match last_token {

            Some(last_word) => {
                let mut fit_mode_used = false;
                let mut alignment_used = false;

                for token in tokens {
                    match token.to_lowercase().as_str() {
                        COVER | CONTAIN | STRETCH            => fit_mode_used = true,
                        LEFT | RIGHT | TOP | BOTTOM | CENTER => alignment_used = true,
                        _ => {}
                    }
                }


                let names: &[&str] = match (fit_mode_used, alignment_used) {
                    (false, false) => &NAMES,
                    (false, true)  => &[COVER, CONTAIN, STRETCH],
                    (true, false)  => &[LEFT, RIGHT, TOP, BOTTOM, CENTER],
                    (true, true)   => &[],
                };


                names.into_iter()
                    .filter(|&name| name.starts_with(last_word))
                    .map(|name| {
                        let space = if tokens_except_last.is_empty() { "" } else { "," };
                        format!("{tokens_except_last}{space}{name}")
                    })
                    .map(CompletionCandidate::new)
                    .collect()
            }

            None => {
                NAMES.iter()
                    .map(CompletionCandidate::new)
                    .collect()
            }
        }
    }
}

fn unquote(input: &str) -> String {
    if let Some(words) = shlex::split(input) {

        if ends_with_separator(input) {
            words.join(",") + ","
        } else {
            words.join(",")
        }

    } else {
        input.trim_matches(|c| c == '\'' || c == '"').to_owned()
    }
}

fn ends_with_separator(input: &str) -> bool {
    input.chars().last().is_some_and(display_mode::is_separator)
}