use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub test: usize,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    pub state: State,
}

impl Store {
    pub fn load(path: PathBuf, default_state: State) -> Result<Store> {
        if path.as_path().exists() {
            let content = fs::read(&path).context("Failed to load state")?;
            let state = serde_json::from_slice(&content).with_context(|| {
                format!(
                    "Failed to serialize stored state: {}",
                    String::from_utf8_lossy(&content),
                )
            })?;
            Ok(Store { path, state })
        } else {
            Store::store_state(&path, &default_state)?;
            Ok(Store {
                path,
                state: default_state,
            })
        }
    }

    fn store_state(path: &Path, state: &State) -> Result<()> {
        let json = serde_json::to_string(state)?;
        fs::write(path, &json).context("Failed to store state")?;
        Ok(())
    }

    pub fn update_state(&mut self, state: State) -> Result<()> {
        Store::store_state(&self.path, &state)?;
        self.state = state;
        Ok(())
    }
}
