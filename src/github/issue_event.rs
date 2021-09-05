use crate::github;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IssueEvent {
    pub event: String,
    pub actor: github::User,
    pub issue: github::Issue,
    pub pull_request: Option<IssueEventPr>,
}

#[derive(Debug, Deserialize)]
pub struct IssueEventPr {
    url: String,
}
