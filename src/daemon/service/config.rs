use std::error::Error;
use std::fs;
use std::path::Path;

use crate::state::State;

pub fn read_or_create_empty(path: impl AsRef<Path>) -> Result<State, Box<dyn Error>> {
    if let Some(dir) = path.as_ref().parent() {
        fs::create_dir_all(dir)?;
    }

    if fs::exists(&path)? {
        let content = fs::read_to_string(&path)?;

        match yaml_serde::from_str(&content) {
            Ok(dto) => Ok(State::from_dto(dto)),
            Err(_) => create_and_write_empty_config(path),
        }
    } else {
        create_and_write_empty_config(path)
    }
}

fn create_and_write_empty_config(path: impl AsRef<Path>) -> Result<State, Box<dyn Error>> {
    let state = State::new();
    write(&state, path)?;
    Ok(state)
}

pub fn write(state: &State, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let content = yaml_serde::to_string(&state.as_dto())?;
    fs::write(&path, content)?;
    Ok(())
}
