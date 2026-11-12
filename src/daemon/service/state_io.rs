use std::error::Error;
use std::fs;

use crate::config::Config;
use crate::state::State;

pub fn read_or_create_empty(config: Config) -> Result<State, Box<dyn Error>> {
	let path = config.state_path();

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    if fs::exists(&path)? {
        let content = fs::read_to_string(&path)?;

        match yaml_serde::from_str(&content) {
            Ok(dto) => Ok(State::from_dto(config, dto)),
            Err(_) => create_and_write_empty(config),
        }
    } else {
        create_and_write_empty(config)
    }
}

fn create_and_write_empty(config: Config) -> Result<State, Box<dyn Error>> {
    let state = State::new(config);
    write(&state)?;
    Ok(state)
}

pub fn write(state: &State) -> Result<(), Box<dyn Error>> {
    let content = yaml_serde::to_string(&state.as_dto())?;
    fs::write(state.path(), content)?;
    Ok(())
}
