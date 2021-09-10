use crate::github;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub html_url: String,
    pub state: IssueState,
    pub number: usize,
    pub title: String,
    pub user: github::User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}
