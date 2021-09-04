use crate::{github, web};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(creds: github::Credentials) -> Result<Self> {
        let client = Self::build_inner_client(creds)?;
        Ok(Client { client })
    }

    fn build_inner_client(creds: github::Credentials) -> Result<reqwest::blocking::Client> {
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

    pub fn get_issue(&self, params: &github::GetIssueParams) -> Result<github::Issue> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            params.repo.owner, params.repo.name, params.number
        );
        let res = self.client.get(&url).send()?;

        if res.status().as_u16() != 200 {
            web::log_error_response(&url, res);
            return Err(anyhow!("failed to fetch issue: {}", url));
        }

        Ok(res.json()?)
    }
}
