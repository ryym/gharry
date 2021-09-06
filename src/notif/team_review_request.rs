use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(_)) => {
            let re = Regex::new(r"^@(?P<user>[^\s]+) requested review from @(?P<team>[^\s]+) on:")?;
            let (reviewee, team) = match re.captures(&enotif.lines[0]) {
                None => return Ok(None),
                Some(caps) => {
                    let params = github::GetUserParams {
                        name: &caps["user"],
                    };
                    let reviewee = match cx.github.get_user(&params)? {
                        None => return Ok(None),
                        Some(reviewee) => reviewee,
                    };
                    (reviewee, caps["team"].to_string())
                }
            };
            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::TeamReviewRequested {
                    reviewee,
                    pr: issue.clone(),
                    team,
                },
            }))
        }
        _ => Ok(None),
    }
}
