use crate::{
    github::{Credentials, GetIssueParams, User},
    web,
};
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
}

#[derive(Debug, Deserialize)]
pub struct GetIssueResponse {
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

impl Client {
    pub fn new(creds: Credentials) -> Result<Self> {
        let client = Self::build_inner_client(creds)?;
        Ok(Client { client })
    }

    fn build_inner_client(creds: Credentials) -> Result<reqwest::blocking::Client> {
        use reqwest::header;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("token {}", creds.auth_token))?,
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent("reqwest") // GitHub API requires User-Agent header.
            .default_headers(headers)
            .build()?;
        Ok(client)
    }

    pub fn get_issue(&self, params: &GetIssueParams) -> Result<GetIssueResponse> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            params.repo.owner, params.repo.name, params.number
        );
        let res = self.client.get(&url).send()?;

        if res.status().as_u16() != 200 {
            web::log_error_response(&url, res);
            return Err(anyhow!("failed to fetch issue: {}", url));
        }

        let data = res.json()?;
        Ok(data)
    }
}
