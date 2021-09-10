use crate::github;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Review {
    pub user: github::User,
    pub body: String,
    pub state: ReviewState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReviewState {
    Commented,
    Approved,
    ChangesRequested,
}
