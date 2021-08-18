use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub last_ts: String,
}

impl State {
    pub fn new() -> Result<State> {
        let now = SystemTime::now();
        let current_ts = now
            .duration_since(UNIX_EPOCH)
            .context("failed to get current timestamp")?
            .as_secs();
        Ok(State {
            last_ts: current_ts.to_string(),
        })
    }
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
    pub fn load<F: FnOnce() -> Result<State>>(path: PathBuf, make_state: F) -> Result<Store> {
        match Store::load_state(&path)? {
            Some(state) => Ok(Store { path, state }),
            None => Store::create(path, make_state()?),
        }
    }

    fn create(path: PathBuf, state: State) -> Result<Store> {
        Store::store_state(&path, &state)?;
        Ok(Store { path, state })
    }

    fn load_state(path: &Path) -> Result<Option<State>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read(&path).context("failed to load state")?;
        let state = serde_json::from_slice(&content).with_context(|| {
            format!(
                "failed to serialize stored state: {}",
                String::from_utf8_lossy(&content),
            )
        })?;
        Ok(Some(state))
    }

    fn store_state(path: &Path, state: &State) -> Result<()> {
        let json = serde_json::to_string(state)?;
        fs::write(path, &json).context("failed to store state")?;
        Ok(())
    }

    pub fn update_state(&mut self, state: State) -> Result<()> {
        Store::store_state(&self.path, &state)?;
        self.state = state;
        Ok(())
    }
}
