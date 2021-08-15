mod config;
mod polling;
mod store;

use crate::config::Config;
use anyhow::Result;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::max())
        .init();

    let config = Config::from_env()?;
    polling::run(config)?;
    Ok(())
}
