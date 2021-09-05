use crate::{github, notif};
use anyhow::{anyhow, Result};
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
            let review = match cx.github.get_pr_review(&params)? {
                None => return Ok(None),
                Some(review) => review,
            };

            // Get the review comment from the email instead of `review.body`. This is because
            // the former contains review discussion comments submitted with the review.
            let whole_comment = extract_whole_comments(&enotif.lines, &review)?;

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::PrReviewed {
                    url: url.clone(),
                    pr: issue.clone(),
                    state: review.state,
                    commenter: review.user,
                    comment: whole_comment,
                },
            }))
        }
        _ => Ok(None),
    }
}

fn extract_whole_comments(lines: &[String], review: &github::Review) -> Result<String> {
    // [Expected text is like following:]
    //
    // @foo commented on this pull request.
    //
    // 1点コメントしたので確認お願いします。
    //
    // > @@ -573,5 +573,91 @@
    //        let(:user) { create(:user) }
    //        it { is_expected.to change { user.reload.deleted? }.from(false).to(true) }
    //
    //  reload ってなぜ必要なんですか？
    //
    //  --
    //  You are receiving this because you were mentioned.
    //  Reply to this email directly or view it on GitHub:
    //  https://github.com/foo/bar/pull/12345#pullrequestreview-1234567890

    // Detect the first and last line index of comment part.
    let head_cmt = review_head_comment(review);
    let head_idx = match lines.iter().position(|l| l == &head_cmt) {
        None => return Err(anyhow!("no head comment in email: {}", lines.join("\n"))),
        Some(idx) => idx,
    };
    let foot_idx = match lines.iter().rposition(|l| l == "-- ") {
        None => return Err(anyhow!("no footer comment in email: {}", lines.join("\n"))),
        Some(idx) => idx,
    };

    // Remove the first and last lines if they are empty.
    let mut comments = &lines[(head_idx + 1)..foot_idx];
    if comments[0].trim() == "" {
        comments = &comments[1..];
    }
    let last_idx = comments.len() - 1;
    if comments[last_idx].trim() == "" {
        comments = &comments[..last_idx];
    }

    Ok(comments.join("\n"))
}

fn review_head_comment(review: &github::Review) -> String {
    let login = &review.user.login;
    match review.state {
        github::ReviewState::Commented => {
            format!("@{} commented on this pull request.", login)
        }
        github::ReviewState::Approved => {
            format!("@{} approved this pull request.", login)
        }
        github::ReviewState::ChangesRequested => {
            format!("@{} requested changes on this pull request.", login)
        }
    }
}
