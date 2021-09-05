use crate::{github, notif};
use anyhow::Result;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(_)) => {
            if !is_pr_open_notif(enotif) {
                return Ok(None);
            }

            let params = github::GetIssueParams {
                repo: &issue.repo,
                number: issue.number,
            };
            let pr = match cx.github.get_issue(&params)? {
                None => return Ok(None),
                Some(pr) => pr,
            };

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::PrOpened {
                    opener: pr.user,
                    pr: issue.clone(),
                },
            }))
        }
        _ => Ok(None),
    }
}

fn is_pr_open_notif(enotif: &github::EmailNotif) -> bool {
    let t = "You can view, comment on, or merge this pull request online at:";
    enotif.lines.iter().any(|l| l.starts_with(t))
}
