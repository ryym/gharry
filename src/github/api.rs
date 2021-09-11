use crate::{
    github::{self, graphql},
    web,
};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
}
#[derive(Debug, Serialize)]
struct GetReviewRequestsVariables<'a> {
    owner: &'a str,
    repo: &'a str,
    pr_number: usize,
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
            header::HeaderValue::from_str(&format!("bearer {}", creds.auth_token))?,
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent("reqwest") // GitHub API requires User-Agent header.
            .default_headers(headers)
            .build()?;
        Ok(client)
    }

    pub fn get_user(&self, params: &github::GetUserParams) -> Result<Option<github::User>> {
        let url = format!("https://api.github.com/users/{}", params.name);
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch user: {}", url))
            }
        }
    }

    pub fn get_issue(&self, params: &github::GetIssueParams) -> Result<Option<github::Issue>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            params.repo.owner, params.repo.name, params.number
        );
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch issue: {}", url))
            }
        }
    }

    pub fn get_issue_comment(
        &self,
        params: &github::GetIssueCommentParams,
    ) -> Result<Option<github::IssueComment>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/comments/{}",
            params.repo.owner, params.repo.name, params.comment_id,
        );
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch issue comment: {}", url))
            }
        }
    }

    pub fn get_issue_event(
        &self,
        params: &github::GetIssueEventParams,
    ) -> Result<Option<github::IssueEvent>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/events/{}",
            params.repo.owner, params.repo.name, params.event_id,
        );
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch issue event: {}", url))
            }
        }
    }

    pub fn get_pr_review(
        &self,
        params: &github::GetPrReviewParams,
    ) -> Result<Option<github::Review>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews/{}",
            params.repo.owner, params.repo.name, params.pr_number, params.review_id,
        );
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch PR review: {}", url))
            }
        }
    }

    pub fn get_pr_review_comment(
        &self,
        params: &github::GetPrReviewCommentParams,
    ) -> Result<Option<github::ReviewComment>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/comments/{}",
            params.repo.owner, params.repo.name, params.review_comment_id,
        );
        let res = self.client.get(&url).send()?;

        match res.status() {
            StatusCode::OK => Ok(res.json()?),
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                web::log_error_response(&url, res);
                Err(anyhow!("failed to fetch PR review: {}", url))
            }
        }
    }

    fn send_graphql<O: DeserializeOwned>(
        &self,
        query: &impl graphql::Query<Output = O>,
    ) -> Result<O> {
        graphql::fetch_data(&self.client, "https://api.github.com/graphql", query)
    }

    pub fn unsubscribe_pr(&self, params: &github::UnsubscribePrParams) -> Result<bool> {
        let data = self.send_graphql(&graphql::GetReviewRequestsQuery {
            owner: &params.repo.owner,
            repo: &params.repo.name,
            pr_number: params.number,
        })?;

        let pr = match data.pull_request() {
            Some(pr) => pr,
            None => return Ok(false),
        };

        let review_requested_directly = pr.review_requests.nodes.iter().any(|r| {
            use graphql::get_review_requests::RequestedReviewer::*;
            match &r.requested_reviewer {
                Some(User { login }) => login == params.user_login,
                _ => false,
            }
        });
        if !review_requested_directly {
            log::info!("skip unsubscribing: no direct review request");
            return Ok(false);
        }

        let data = self.send_graphql(&graphql::UpdateSubscriptionMut {
            input: graphql::UpdateSubscriptionInput {
                state: graphql::SubscriptionState::Unsubscribed,
                subscribable_id: pr.id.clone(),
            },
        })?;

        let updated_state = data.viewer_subscription();
        log::debug!("unsubscribed: {:?}", updated_state);

        Ok(matches!(
            updated_state,
            Some(graphql::SubscriptionState::Unsubscribed)
        ))
    }
}
