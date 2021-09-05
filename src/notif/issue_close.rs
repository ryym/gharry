use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(url)) => {
            if !is_issue_closed_notif(enotif) {
                return Ok(None);
            }

            let re = Regex::new(r"#event-(?P<id>\d+)$")?;
            let event_id = match re.captures(&url) {
                None => return Ok(None),
                Some(caps) => caps["id"].parse().unwrap(),
            };

            let params = github::GetIssueEventParams {
                repo: &issue.repo,
                event_id,
            };
            let event = match cx.github.get_issue_event(&params)? {
                None => return Ok(None),
                Some(event) => event,
            };

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::IssueClosed {
                    closer: event.actor,
                    issue: issue.clone(),
                    is_merge: event.event == "merged",
                },
            }))
        }
        _ => Ok(None),
    }
}

fn is_issue_closed_notif(enotif: &github::EmailNotif) -> bool {
    let l = &enotif.lines[0];
    l.starts_with("Closed #") || l.starts_with("Merged")
}
