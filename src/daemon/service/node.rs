use std::env;

use comfy_table::{ColumnConstraint, ContentArrangement, Table, Width, presets};
use itertools::Itertools;

use crate::cli::{ParsedTimePeriod, Settings};
use crate::core::ActionPerformError;
use crate::daemon::backend;
use crate::state::{State, TimePeriod, Wallpaper};
use crate::localized;

pub fn get_list(state: &State, is_verbose: bool, term_width: u16) -> String {
    if state.nodes.is_empty() {
        String::from("No wallpapers are found")
    } else if is_verbose {

        let mut table = Table::new();

        println!("term_width: {term_width}");

        table
            .load_style(presets::UTF8_FULL)
            .set_width(term_width)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_constraints([
                ColumnConstraint::UpperBoundary(Width::Percentage(25)),
                ColumnConstraint::UpperBoundary(Width::Percentage(25)),
                ColumnConstraint::UpperBoundary(Width::Percentage(50)),
            ])
            .set_header(localized!(
                vec!["Mode",  "Period", "Path"],
                vec!["Режим", "Период", "Путь"]
            ));
        
        let home = env::home_dir()
            .and_then(|home| home.to_str().map(String::from))
            .filter(|home| !home.is_empty())
            .map(|home| if home.ends_with('/') { home } else { home + "/" });

        for node in state.nodes.values() {
            let node = node.borrow();
            let path = node.path();

            let path = if let Some(ref home) = home && path.starts_with(home) {
                String::from("~/") + &path[home.len()..]
            } else {
                String::from(path)
            };

            table.add_row(vec![
                node.mode.to_string(),
                TimePeriod::opt_to_string(&node.period),
                path,
            ]);
        }

        table.to_string()

    } else {
        format!(
            "Wallpapers:\n{}",
            state.nodes.values()
                .map(|node| String::from(node.borrow().path()))
                .format("\n")
        )
    }
}

pub fn add(state: &mut State, paths: &Vec<String>, settings: &Settings, period: ParsedTimePeriod) -> Result<(), ActionPerformError> {
    let mut update_current = false;

    for path in paths {
        state.update_node(path, settings, period.clone());
        
        update_current = update_current ||
            state.current_wallpaper_path
                .as_ref()
                .is_some_and(|p| path.starts_with(p));
    }

    if update_current {
        let node = state.get_or_insert_node(&state.current_wallpaper_path.clone().unwrap());

        backend::run(&Wallpaper::from(&*node.borrow())).map_err(|err| {
            state.current_wallpaper_path = None;
            ActionPerformError::from_boxed(err)
        })?;
    }

    Ok(())
}

pub fn remove(state: &mut State, paths: &Vec<String>) {
    for path in paths {
        if state
            .current_wallpaper_path
            .as_ref()
            .is_some_and(|p| *p == *path)
        {
            backend::stop();
            state.current_wallpaper_path = None;
        }

        for (_, group) in &mut state.groups {
            group.remove(path);
        }

        state.nodes.remove(path);
    }
}

pub fn clear(state: &mut State) {
    state.current_wallpaper_path = None;
    state.nodes.clear();
    state.groups.clear();
}
