mod api;
mod graphql;

pub use api::Client;

use crate::email::Email;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug)]
pub struct Credentials {
    pub auth_token: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Repository {
    pub owner: String,
    pub name: String,
}

impl Repository {
    pub fn fullname(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct IssueInfo {
    pub repo: Repository,
    pub number: usize,
    pub title: String,
}

#[derive(Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
}

#[derive(Debug)]
pub struct EmailNotif {
    pub lines: Vec<String>,
    pub detected_issue: Option<IssueInfo>,
    pub github_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub html_url: String,
    pub state: IssueState,
    pub number: usize,
    pub title: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug, Deserialize)]
pub struct IssueEvent {
    pub event: String,
    pub actor: User,
    pub issue: Issue,
    pub pull_request: Option<IssueEventPr>,
}

#[derive(Debug, Deserialize)]
pub struct IssueEventPr {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct Review {
    pub user: User,
    pub body: String,
    pub state: ReviewState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReviewState {
    Commented,
    Approved,
    ChangesRequested,
    Dismissed,
}

#[derive(Debug)]
pub struct GetUserParams<'a> {
    pub name: &'a str,
}

#[derive(Debug)]
pub struct GetIssueParams<'a> {
    pub repo: &'a Repository,
    pub number: usize,
}

#[derive(Debug)]
pub struct GetIssueCommentParams<'a> {
    pub repo: &'a Repository,
    pub comment_id: usize,
}

#[derive(Debug, Deserialize)]
pub struct IssueComment {
    pub body: String,
    pub user: User,
}

#[derive(Debug)]
pub struct GetIssueEventParams<'a> {
    pub repo: &'a Repository,
    pub event_id: usize,
}

#[derive(Debug)]
pub struct GetPrReviewParams<'a> {
    pub repo: &'a Repository,
    pub pr_number: usize,
    pub review_id: usize,
}

#[derive(Debug)]
pub struct GetPrReviewCommentParams<'a> {
    pub repo: &'a Repository,
    pub review_comment_id: usize,
}

#[derive(Debug, Deserialize)]
pub struct ReviewComment {
    pub user: User,
    pub body: String,
}

#[derive(Debug)]
pub struct UnsubscribePrParams<'a> {
    pub repo: &'a Repository,
    pub number: usize,
    pub user_login: &'a str,
}

pub fn build_notif_from_email(email: &Email) -> Result<EmailNotif> {
    let lines = email
        .text_body
        .replace('\r', "")
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let issue = issue_info_from_notif_subject(&email.subject)?;
    let github_url = find_github_link(&lines);

    Ok(EmailNotif {
        lines,
        detected_issue: issue,
        github_url,
    })
}

fn issue_info_from_notif_subject(subject: &str) -> Result<Option<IssueInfo>> {
    // A regex matches with a subject such as "Re: [ryym/gharry] Fix typo (#1234)".
    // Note that some email subjects end with "(PR #1234)" instead of "(#1234)" since 2021-10-18.
    let re = Regex::new(
        r"^(?:Re: )?\[(?P<owner>[^/]+)/(?P<repo>[^\]]+)\] (?P<title>.+) \((?:PR )?#(?P<issue>\d+)\)$",
    )?;
    match re.captures(subject) {
        None => Ok(None),
        Some(caps) => {
            let issue = IssueInfo {
                repo: Repository {
                    owner: caps["owner"].to_string(),
                    name: caps["repo"].to_string(),
                },
                number: caps["issue"].parse().unwrap(),
                title: caps["title"].to_string(),
            };
            Ok(Some(issue))
        }
    }
}

fn find_github_link(lines: &[String]) -> Option<String> {
    let idx_last = lines.len() - 1;
    lines.get(idx_last - 1).and_then(|prev_last| {
        if prev_last.starts_with("Reply to this email directly or view it on GitHub:")
            || prev_last.starts_with("View it on GitHub:")
        {
            let maybe_url = lines[idx_last].trim();
            if maybe_url.starts_with("https://github.com") {
                return Some(maybe_url.to_string());
            }
        }
        None
    })
}
