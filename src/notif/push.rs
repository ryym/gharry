use crate::{github, notif};
use anyhow::Result;
use regex::Regex;

pub(super) fn try_parse(
    cx: &notif::BuildContext,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    match (&enotif.detected_issue, &enotif.github_url) {
        (Some(issue), Some(url)) => {
            let re = Regex::new(r"^@(?P<user>[^\s]+) pushed (?P<count>\d+) commits?\.")?;
            let (committer, commit_cnt) = match re.captures(&enotif.lines[0]) {
                None => return Ok(None),
                Some(caps) => {
                    let params = github::GetUserParams {
                        name: &caps["user"],
                    };
                    let user = match cx.github.get_user(&params)? {
                        None => return Ok(None),
                        Some(user) => user,
                    };
                    let commit_cnt = caps["count"].parse::<usize>().unwrap();
                    (user, commit_cnt)
                }
            };
            let commits = extract_commit_info(&enotif.lines)?;
            if commits.len() != commit_cnt {
                panic!("commit count mismatch: {} != {}", commits.len(), commit_cnt);
            }

            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::Pushed {
                    pr: issue.clone(),
                    diff_url: url.clone(),
                    committer,
                    commits,
                },
            }))
        }
        _ => Ok(None),
    }
}

fn extract_commit_info(lines: &[String]) -> Result<Vec<github::CommitInfo>> {
    let hash_re = Regex::new(r"^[a-z0-9]{40}")?;
    let commits = lines
        .iter()
        .filter_map(|line| {
            let mut parts = line.split_ascii_whitespace();
            match (parts.next(), parts.next()) {
                (Some(maybe_hash), Some(maybe_msg)) if hash_re.is_match(maybe_hash) => {
                    Some(github::CommitInfo {
                        hash: maybe_hash.to_string(),
                        message: maybe_msg.to_string(),
                    })
                }
                _ => None,
            }
        })
        .collect();
    Ok(commits)
}
