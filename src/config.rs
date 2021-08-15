use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let home_dir = env::var("HOME").context("Failed to detect home directory via HOME")?;
        let config_dir: PathBuf = [&home_dir, ".config", "gharry"].iter().collect();

        if !config_dir.as_path().exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(Config { dir: config_dir })
    }
}
