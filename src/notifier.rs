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

    let mention = if should_alert(&notif.detail) {
        "\n<!here>"
    } else {
        ""
    };
    match generate_message(notif) {
        None => {
            log::info!("Skip sending notification");
        }
        Some(msg) => {
            log::info!("Sending notification...");
            let text = format!("{}{}", msg.text, mention);
            slack.chat_post_message(&slack::ChatMessage {
                channel,
                text: &text,
                username: Some(msg.user_name.as_deref().unwrap_or(DEFAULT_USER_NAME)),
                icon_url: msg.icon_url.as_deref(),
                icon_emoji: icon_emoji(&msg.icon_url),
                unfurl_links: false,
                unfurl_media: false,
            })?;
        }
    }

    Ok(())
}

fn should_alert(detail: &NotifDetail) -> bool {
    !matches!(
        detail,
        NotifDetail::Pushed { .. } | NotifDetail::PrOpened { .. } | NotifDetail::IssueClosed { .. },
    )
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

        NotifDetail::PrReviewed {
            url,
            pr,
            state,
            commenter,
            comment,
        } => {
            let login = format!("@{}", commenter.login);
            let pr_sbj = issue_subject(&pr, Some(&url));
            let state_icon = review_state_emoji(&state);
            let text = format!("{} {} {}\n{}", login, state_icon, pr_sbj, comment);
            Some(NotifMessage {
                text,
                user_name: Some(login),
                icon_url: Some(commenter.avatar_url),
            })
        }

        NotifDetail::PrReviewCommented {
            url,
            pr,
            commenter,
            comment,
        } => {
            let login = format!("@{}", commenter.login);
            let pr_sbj = issue_subject(&pr, Some(&url));
            Some(NotifMessage {
                text: format!("{} 💬  {}\n{}", login, pr_sbj, comment),
                user_name: Some(login),
                icon_url: Some(commenter.avatar_url),
            })
        }

        NotifDetail::DirectReviewRequested { reviewee, pr } => {
            let login = format!("@{}", reviewee.login);
            let pr_sbj = issue_subject(&pr, None);
            Some(NotifMessage {
                text: format!("{} requested your review on {}", login, pr_sbj),
                user_name: Some(login),
                icon_url: Some(reviewee.avatar_url),
            })
        }

        NotifDetail::TeamReviewRequested { .. } => None,

        NotifDetail::IssueClosed {
            closer,
            issue,
            is_merge,
        } => {
            let login = format!("@{}", closer.login);
            let issue_sbj = issue_subject(&issue, None);
            let action = if is_merge { "merged" } else { "closed" };
            Some(NotifMessage {
                text: format!("{} {} {}", login, action, issue_sbj),
                user_name: Some(login),
                icon_url: Some(closer.avatar_url),
            })
        }

        NotifDetail::Commented {
            url,
            commenter,
            issue,
            comment,
        } => {
            let login = format!("@{}", commenter.login);
            let issue_sbj = issue_subject(&issue, Some(&url));
            Some(NotifMessage {
                text: format!("{} 💬  {}\n{}", login, issue_sbj, comment),
                user_name: Some(login),
                icon_url: Some(commenter.avatar_url),
            })
        }

        NotifDetail::Pushed {
            pr,
            diff_url,
            committer,
            commits,
        } => {
            let login = format!("@{}", committer.login);
            let commits_summary = format!(
                "{} commit{}",
                commits.len(),
                if commits.len() == 1 { "" } else { "s" }
            );
            let joined_msg = join_commit_messages(&commits, 10);
            let pr_sbj = issue_subject(&pr, None);
            Some(NotifMessage {
                text: format!(
                    "{} pushed <{}|{}> to {}\n{}",
                    login, diff_url, commits_summary, pr_sbj, joined_msg
                ),
                user_name: Some(login),
                icon_url: Some(committer.avatar_url),
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
        "[{}<{}|#{}>] {}",
        &issue.repo.fullname(),
        pr_url,
        issue.number,
        title
    )
}

fn review_state_emoji(state: &github::ReviewState) -> &'static str {
    match *state {
        github::ReviewState::Commented => "💬",
        github::ReviewState::Approved => "👍",
        github::ReviewState::ChangesRequested => "⚠️",
        github::ReviewState::Dismissed => "",
    }
}

fn join_commit_messages(commits: &[github::CommitInfo], max: usize) -> String {
    let mut joined_msg = commits
        .iter()
        .take(max)
        .map(|c| c.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if commits.len() > max {
        joined_msg.push_str(&format!("...and more {} commits", commits.len() - max));
    }
    joined_msg
}
