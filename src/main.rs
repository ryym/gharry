mod config;
mod email;
mod env;
mod github;
mod notif;
mod notifier;
mod polling;
mod slack;
mod store;
mod web;

use crate::config::Config;
use anyhow::{Context, Result};
use std::fs;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let config = Config::from_env()?;
    if !config.dir.as_path().exists() {
        fs::create_dir_all(&config.dir).context("failed to create config directory")?;
    }

    polling::run(config)?;
    Ok(())
}
