use crate::{
    github,
    notif::{NotifDetail, Notification},
    slack,
};
use anyhow::Result;

#[derive(Debug)]
struct NotifMessage {
    text: String,
    user_name: Option<String>,
    icon_url: Option<String>,
}

const DEFAULT_USER_NAME: &str = "Gharry";
const DEFAULT_ICON_EMOJI: &str = ":carousel_horse:";

pub fn notify_by_slack(slack: &slack::Client, channel: &str, notif: Notification) -> Result<()> {
    log::debug!("notifying {:?}", notif);

    match generate_message(notif) {
        None => {
            log::info!("Skip sending notification");
        }
        Some(msg) => {
            log::info!("Sending notification...");
            slack.chat_post_message(&slack::ChatMessage {
                channel,
                text: &msg.text,
                username: Some(msg.user_name.as_deref().unwrap_or(DEFAULT_USER_NAME)),
                icon_url: msg.icon_url.as_deref(),
                icon_emoji: icon_emoji(&msg.icon_url),
            })?;
        }
    }

    Ok(())
}

fn icon_emoji(icon_url: &Option<String>) -> Option<&str> {
    match icon_url {
        Some(_) => None,
        None => Some(DEFAULT_ICON_EMOJI),
    }
}

fn generate_message(notif: Notification) -> Option<NotifMessage> {
    match notif.detail {
        NotifDetail::Unknown { sender, body } => Some(NotifMessage {
            text: body.join("\n"),
            user_name: Some(sender),
            icon_url: None,
        }),
        NotifDetail::PrOpened { opener, pr } => {
            let login = format!("@{}", opener.login);
            let pr_sbj = issue_subject(&pr, None);
            Some(NotifMessage {
                text: format!("{} opened {}", login, pr_sbj),
                user_name: Some(login),
                icon_url: Some(opener.avatar_url),
            })
        }
    }
}

fn issue_subject(issue: &github::IssueInfo, title_link: Option<&str>) -> String {
    let pr_url = format!(
        "https://github.com/{}/{}/pull/{}",
        &issue.repo.owner, &issue.repo.name, issue.number
    );
    let title_link = title_link.map(|link| format!("<{}|{}>", link, &issue.title));
    let title = title_link.as_deref().unwrap_or(&issue.title);
    format!(
        "[{}/{}#<{}|{}>] {}",
        &issue.repo.owner, &issue.repo.name, pr_url, issue.number, title
    )
}
