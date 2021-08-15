use crate::{
    config::Config,
    store::{State, Store},
};
use anyhow::Result;
use log::info;
use std::{thread, time::Duration};

pub fn run(config: Config) -> Result<()> {
    let state_path = config.dir.join("state.json");
    let mut i = 0;
    let mut store = Store::load(state_path, State { test: i })?;
    info!("Start from state: {}", store.state);

    loop {
        i += 1;
        store.update_state(State { test: i })?;

        info!("Finished so wait a while...");
        thread::sleep(Duration::from_secs(10));
    }
}
