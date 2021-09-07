use crate::{
    config::Config,
    github, notif, notifier, slack,
    store::{State, Store},
};
use anyhow::Result;
use std::{thread, time::Duration};

pub fn run(config: Config) -> Result<()> {
    let state_filename = format!(".state-{}.json", config.slack.mail_channel_id);
    let state_path = config.dir.join(state_filename);

    let mut store = Store::load(state_path, State::new)?;
    log::info!("Start from state: {}", store.state);

    let slack = slack::Client::new(slack::Credentials {
        bot_token: config.slack.bot_token.clone(),
    })?;

    let github = github::Client::new(github::Credentials {
        auth_token: config.github.auth_token.clone(),
    })?;

    loop {
        let data = slack.conversations_history(slack::ConvHistoryParams {
            channel: &config.slack.mail_channel_id,
            // - We don't need to specify the valid ts; it is just a timestamp.
            // - The result does not contain a message with the specified ts if exists.
            oldest_ts: &store.state.last_ts,
        })?;

        if data.messages.is_empty() {
            log::info!("No new notifications found");
        } else {
            log::info!("{} notifications found", data.messages.len());
            let last_ts = filter_and_notify(&config, &slack, &github, data.messages)?;
            store.update_state(State { last_ts })?;
        }

        log::info!("Finished so wait a while...");
        thread::sleep(Duration::from_secs(10));
    }
}

fn filter_and_notify(
    config: &Config,
    slack: &slack::Client,
    github: &github::Client,
    mut messages: Vec<slack::Message>,
) -> Result<String> {
    // The messages are sorted by newest to oldest so
    // we reverse the order to process them from oldest.
    messages.reverse();

    let last_ts = messages[messages.len() - 1].ts.clone();

    let notifs = notif::build_notifications(notif::BuildContext { github }, messages)?;
    for notif in notifs {
        notifier::notify_by_slack(slack, &config.slack.dest_channel_id, notif)?;
    }

    Ok(last_ts)
}
