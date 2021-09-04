use crate::github;
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub html_url: String,
    pub state: IssueState,
    pub number: usize,
    pub title: String,
    pub user: github::User,
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
