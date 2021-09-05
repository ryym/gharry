use crate::github;
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Review {
    pub user: github::User,
    pub body: String,
    pub state: ReviewState,
}

#[derive(Debug)]
pub enum ReviewState {
    Commented,
    Approved,
    ChangesRequested,
}

impl ReviewState {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "COMMENTED" => Ok(Self::Commented),
            "APPROVED" => Ok(Self::Approved),
            "CHANGES_REQUESTED" => Ok(Self::ChangesRequested),
            _ => Err(anyhow!("unknown review state: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for ReviewState {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s: &str = Deserialize::deserialize(d)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}
