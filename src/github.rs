mod api;

pub use api::Client;

use crate::email::Email;
use anyhow::{anyhow, Result};
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

#[derive(Debug, Clone)]
pub struct IssueInfo {
    pub repo: Repository,
    pub number: usize,
    pub title: String,
}

#[derive(Debug)]
pub struct EmailNotif {
    pub email: Email,
    pub lines: Vec<String>,
    pub detected_issue: Option<IssueInfo>,
    pub github_url: Option<String>,
}

#[derive(Debug)]
pub struct GetIssueParams<'a> {
    pub repo: &'a Repository,
    pub number: usize,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub html_url: String,
    pub state: IssueState,
    pub number: usize,
    pub title: String,
    pub user: User,
}

#[derive(Debug)]
pub enum IssueState {
    Open,
    Closed,
}

impl IssueState {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            _ => Err(anyhow!("unknown issue state: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for IssueState {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s: &str = Deserialize::deserialize(d)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
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

pub fn build_notif_from_email(email: Email) -> Result<EmailNotif> {
    let lines = email
        .text_body
        .replace('\r', "")
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let issue = issue_info_from_notif_subject(&email.subject)?;
    let github_url = find_github_link(&lines);

    Ok(EmailNotif {
        email,
        lines,
        detected_issue: issue,
        github_url,
    })
}

fn issue_info_from_notif_subject(subject: &str) -> Result<Option<IssueInfo>> {
    // A regex matches with a subject such as "Re: [ryym/gharry] Fix typo (#1234)".
    let re = Regex::new(
        r"^(?:Re: )?\[(?P<owner>[^/]+)/(?P<repo>[^\]]+)\] (?P<title>.+) \(#(?P<issue>\d+)\)$",
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
