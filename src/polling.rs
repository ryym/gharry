use crate::{
    config::Config,
    slack,
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

    let slack = slack::Client::new(slack::Credentials {
        bot_token: config.slack.bot_token.clone(),
    });

    loop {
        let data = slack.conversations_history(slack::ConvHistoryParams {
            channel: &config.slack.mail_channel_id,
            // - We don't need to specify the valid ts; it is just a timestamp.
            // - The result does not contain a message with the specified ts if exists.
            oldest_ts: &store.state.last_ts,
        })?;

        if data.messages.is_empty() {
            info!("No new notifications found");
        } else {
            info!("{} notifications found", data.messages.len());
            let last_ts = filter_and_notify(&config, &slack, data.messages)?;
            store.update_state(State { last_ts })?;
        }

        info!("Finished so wait a while...");
        thread::sleep(Duration::from_secs(10));
    }
}

fn filter_and_notify(
    config: &Config,
    slack: &slack::Client,
    mut messages: Vec<slack::Message>,
) -> Result<String> {
    // The messages are sorted by newest to oldest so
    // we reverse the order to process them from oldest.
    messages.reverse();

    let last_ts = messages[messages.len() - 1].ts.clone();

    for _ in &messages {
        notify_by_slack(
            slack,
            config.slack.dest_channel_id.to_string(),
            String::from("test"),
        )?;
    }

    Ok(last_ts)
}

fn notify_by_slack(slack: &slack::Client, channel: String, text: String) -> Result<()> {
    slack.chat_post_message(&slack::ChatMessage { channel, text })?;
    Ok(())
}
