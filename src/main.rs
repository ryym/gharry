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

use crate::{config::Config, env::must_get_env};
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let work_dir = [&must_get_env("HOME")?, ".gharry"]
        .iter()
        .collect::<PathBuf>();
    if !work_dir.as_path().exists() {
        fs::create_dir_all(&work_dir).context("failed to create config directory")?;
    }

    let config = Config::build_default(work_dir)?;
    polling::run(config)?;

    Ok(())
}
