use crate::{config::Config, github, notif, slack};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

pub fn run(config: &Config, oldest_ts: &str) -> Result<()> {
    let slack = slack::Client::new(slack::Credentials {
        bot_token: config.slack.bot_token.clone(),
    })?;
    let github = github::Client::new(github::Credentials {
        auth_token: config.github.auth_token.clone(),
    })?;

    let data = slack.conversations_history(slack::ConvHistoryParams {
        channel: &config.slack.mail_channel_id,
        oldest_ts,
        limit: Some("1"),
    })?;

    let msgs_json = serde_json::to_string(&data.messages)?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("_inspect_msgs.json")?
        .write_all(&msgs_json.into_bytes())?;

    let ctx = notif::BuildContext { github: &github };
    let notifs = notif::build_notifications(ctx, data.messages)?;

    let notifs_json = serde_json::to_string(&notifs)?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("_inspect_notifs.json")?
        .write_all(&notifs_json.into_bytes())?;

    Ok(())
}
