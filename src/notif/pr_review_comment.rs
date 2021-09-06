use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(url)) => {
            let re = Regex::new(r"#discussion_r(?P<id>\d+)$")?;
            let review_comment_id = match re.captures(&url) {
                None => return Ok(None),
                Some(caps) => caps["id"].parse().unwrap(),
            };

            let params = github::GetPrReviewCommentParams {
                repo: &issue.repo,
                review_comment_id,
            };
            let comment = match cx.github.get_pr_review_comment(&params)? {
                None => return Ok(None),
                Some(comment) => comment,
            };

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::PrReviewCommented {
                    url: url.clone(),
                    pr: issue.clone(),
                    commenter: comment.user,
                    comment: comment.body,
                },
            }))
        }
        _ => Ok(None),
    }
}
