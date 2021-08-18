use crate::{
    api::slack,
    config::Config,
    store::{State, Store},
};
use anyhow::Result;
use log::info;
use std::{thread, time::Duration};

pub fn run(config: Config) -> Result<()> {
    let state_filename = format!(".state-{}.json", config.slack.mail_channel_id);
    let state_path = config.dir.join(state_filename);

    let mut store = Store::load(state_path, State::new)?;
    info!("Start from state: {}", store.state);

    let client = slack::Client::new(slack::Credentials {
        bot_token: config.slack.bot_token,
    });

    loop {
        let data = client.conversations_history(slack::ConvHistoryParams {
            channel: &config.slack.mail_channel_id,
            oldest_ts: &store.state.last_ts,
        })?;
        println!("{:?}", data);

        store.update_state(State {
            last_ts: store.state.last_ts.clone(),
        })?;

        info!("Finished so wait a while...");
        thread::sleep(Duration::from_secs(10));
    }
}
