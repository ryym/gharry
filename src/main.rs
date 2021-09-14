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
use std::{fs, path::PathBuf, thread, time::Duration};

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
    'poll_loop: loop {
        match polling::run(&config) {
            Ok(()) => return Ok(()),
            Err(err) => {
                for cause in err.chain() {
                    if let Some(err) = cause.downcast_ref::<reqwest::Error>() {
                        log::info!("some web request failed: {}", err);
                        log::info!("will retry polling after {} seconds...", 30);
                        thread::sleep(Duration::from_secs(30));
                        continue 'poll_loop;
                    }
                }
                return Err(err);
            }
        }
    }
}
