use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(url)) => {
            let re = Regex::new(r"#pullrequestreview-(?P<id>\d+)$")?;
            let review_id = match re.captures(&url) {
                None => return Ok(None),
                Some(caps) => caps["id"].parse().unwrap(),
            };

            let params = github::GetPrReviewParams {
                repo: &issue.repo,
                pr_number: issue.number,
                review_id,
            };
            let review = cx.github.get_pr_review(&params)?;

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::PrReviewed {
                    url: url.clone(),
                    pr: issue.clone(),
                    state: review.state,
                    commenter: review.user,
                    comment: review.body,
                },
            }))
        }
        _ => Ok(None),
    }
}
