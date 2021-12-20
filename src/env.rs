use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct EnvSetup {
    pub work_dir: PathBuf,
}

pub fn setup_exec_env() -> Result<EnvSetup> {
    // Initialize the logger.
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Create a working directory if necessary.
    let work_dir = [&must_get_env("HOME")?, ".gharry"]
        .iter()
        .collect::<PathBuf>();
    if !work_dir.as_path().exists() {
        fs::create_dir_all(&work_dir).context("failed to create config directory")?;
    }

    Ok(EnvSetup { work_dir })
}

fn must_get_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("environment variable required: {}", name))
}
