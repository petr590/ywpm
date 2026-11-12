use std::path::PathBuf;

use comfy_table::{ColumnConstraint, ContentArrangement, Table, Width, presets};
use itertools::Itertools;

use crate::cli::{CliTimePeriod, ParsedTimePeriod, Settings};
use crate::core::ActionPerformError;
use crate::daemon::backend;
use crate::daemon::service::wallpaper;
use crate::state::{State, TimePeriod, Wallpaper};
use crate::{action_perform_error_localized, localized, util};

pub fn get_list(state: &State, is_verbose: bool, term_width: u16) -> String {
    if state.nodes.is_empty() {
        return String::from(localized!(
            "No wallpapers are found",
            "Обои не найдены",
        ));
    }
    
    if is_verbose {
        let mut table = Table::new();

        table
            .load_style(presets::UTF8_FULL)
            .set_width(term_width)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_constraints([
                ColumnConstraint::UpperBoundary(Width::Percentage(25)),
                ColumnConstraint::UpperBoundary(Width::Percentage(25)),
                ColumnConstraint::Absolute(Width::Fixed(9)),
                ColumnConstraint::UpperBoundary(Width::Percentage(100)),
            ])
            .set_header(localized!(
                vec!["Mode",  "Period", "Recurs.\nlevel", "Path"],
                vec!["Режим", "Период", "Уровень\nрекурс.", "Путь"]
            ));
        
        let nodes = state.nodes.values()
            .sorted_by(|node1, node2| node1.borrow().path().cmp(node2.borrow().path()));

        for node in nodes {
            let node = node.borrow();

            let recursive_level = if node.path().is_file() {
                String::from("-")
            } else {
                node.recursive_level.to_string()
            };

            table.add_row(vec![
                node.mode.to_string(),
                TimePeriod::opt_to_string(&node.period),
                recursive_level,
                util::replace_home_with_tilde(node.path()),
            ]);
        }

        table.to_string()

    } else {
        let mut result = String::from("Wallpapers:\n");

        for node in state.nodes.values() {
            result.push_str(&node.borrow().path().to_string_lossy());
            result.push('\n');
        }

        result
    }
}

pub fn add_or_update(state: &mut State, paths: &Vec<PathBuf>, settings: &Settings, period: ParsedTimePeriod) -> Result<(), ActionPerformError> {
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

        backend::run(state.config(), &Wallpaper::from(&*node.borrow())).map_err(|err| {
            state.current_wallpaper_path = None;
            ActionPerformError::new(err.to_string())
        })?;
    }

    Ok(())
}

pub fn update(state: &mut State, mut paths: Vec<PathBuf>, settings: &Settings, period: ParsedTimePeriod) -> Result<(), ActionPerformError> {

    if settings.is_none() && period == ParsedTimePeriod::NotSpecified {
        return Err(action_perform_error_localized!(
            "One of the following option must be specified: {}, {}",
            "Одна из следующих опций должна быть указана: {}, {}",
            Settings::ALL_OPTIONS, CliTimePeriod::ALL_OPTIONS,
        ));
    }

    if paths.is_empty() {
        paths.push(wallpaper::current_wallpaper_path_or_error(state)?);
    }

    add_or_update(state, &paths, &settings, period)
}

pub fn remove(state: &mut State, paths: &Vec<PathBuf>) {
    for path in paths {
        if state.current_wallpaper_path
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
