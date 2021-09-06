use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(_)) => {
            let re = Regex::new(r"^@(?P<user>[^\s]+) requested your review on:")?;
            let reviewee = match re.captures(&enotif.lines[0]) {
                None => return Ok(None),
                Some(caps) => {
                    let params = github::GetUserParams {
                        name: &caps["user"],
                    };
                    match cx.github.get_user(&params)? {
                        None => return Ok(None),
                        Some(reviewee) => reviewee,
                    }
                }
            };
            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::DirectReviewRequested {
                    reviewee,
                    pr: issue.clone(),
                },
            }))
        }
        _ => Ok(None),
    }
}
