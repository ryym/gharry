use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(url)) => {
            let re = Regex::new(r"#issuecomment-(?P<id>\d+)$")?;
            let comment_id = match re.captures(&url) {
                None => return Ok(None),
                Some(caps) => caps["id"].parse().unwrap(),
            };

            let params = github::GetIssueCommentParams {
                repo: &issue.repo,
                comment_id,
            };
            let comment = match cx.github.get_issue_comment(&params)? {
                None => return Ok(None),
                Some(comment) => comment,
            };

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::Commented {
                    url: url.clone(),
                    commenter: comment.user,
                    issue: issue.clone(),
                    comment: comment.body,
                },
            }))
        }
        _ => Ok(None),
    }
}
