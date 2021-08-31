use crate::{github, slack};
use anyhow::Result;

#[derive(Debug)]
pub struct Notification {
    issue: Option<github::IssueInfo>,
}

pub fn messages_into_notifications(
    messages: impl Iterator<Item = slack::Message>,
) -> Result<Vec<Notification>> {
    let notifs = messages
        .filter_map(slack::extract_email_from_message)
        .try_fold(Vec::new(), |mut notifs, email| {
            github::build_notif_from_email(email).map(|notif| {
                notifs.push(build_notification(notif));
                notifs
            })
        })?;

    Ok(notifs)
}

fn build_notification(enotif: github::EmailNotif) -> Notification {
    Notification {
        issue: enotif.detected_issue,
    }
}
