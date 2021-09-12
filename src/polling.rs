use crate::{
    config::Config,
    github, notif, notifier, slack,
    store::{State, Store},
};
use anyhow::Result;
use std::{collections::HashSet, thread, time::Duration};

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
    let unsubscribed = unsubscribe_undesired_notifs(github, &notifs)?;

    for (idx, notif) in notifs.into_iter().enumerate() {
        if !unsubscribed.contains(&idx) {
            notifier::notify_by_slack(slack, &config.slack.dest_channel_id, notif)?;
        }
    }

    Ok(last_ts)
}

fn unsubscribe_undesired_notifs(
    github: &github::Client,
    notifs: &[notif::Notification],
) -> Result<HashSet<usize>> {
    let targets = notifs
        .iter()
        .enumerate()
        .filter_map(|(idx, notif)| match notif.detail {
            notif::NotifDetail::TeamReviewRequested { ref pr, .. } => Some((idx, pr)),
            _ => None,
        });

    let mut unsubscribed = HashSet::new();
    for (idx, pr) in targets {
        log::debug!("unsubscribing {}#{}...", pr.repo.fullname(), pr.number);
        let done = github.unsubscribe_pr(&github::UnsubscribePrParams {
            repo: &pr.repo,
            number: pr.number,
            user_login: "ryym", // TODO: Get value from config
        })?;
        if done {
            unsubscribed.insert(idx);
        }
    }

    Ok(unsubscribed)

    // TODO: reason が mention な通知が来てたら unsubscribe しないようにする。
    // 常に最新の通知を処理してれば大抵は unsubscribe -> メンション通知という順序なので問題ないが、
    // 溜まった通知メールを一気に処理するようなケースでは、メンション通知を受け取ってるのに
    // 後から unsubscribe してしまいそれ以降の通知を受け取れなくなる、という事が起きえる。
}
