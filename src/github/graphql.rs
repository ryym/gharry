use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

pub trait Query {
    type Output: DeserializeOwned;
    fn to_json(&self) -> serde_json::Value;
}

#[derive(Debug, Deserialize)]
pub struct QueryResult<D> {
    pub data: Option<D>,
    pub errors: Option<Vec<ErrorItem>>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorItem {
    message: String,
}

pub fn send_request<O: DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: &str,
    query: &impl Query<Output = O>,
) -> Result<reqwest::blocking::Response> {
    let res = client.post(url).body(query.to_json().to_string()).send()?;
    Ok(res)
}

pub fn send<O: DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: &str,
    query: &impl Query<Output = O>,
) -> Result<QueryResult<O>> {
    let res = send_request(client, url, query)?;
    Ok(res.json()?)
}

pub fn fetch_data<O: DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: &str,
    query: &impl Query<Output = O>,
) -> Result<O> {
    let r = send(client, url, query)?;
    if let Some(errs) = r.errors {
        let msg = errs
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join(",");
        return Err(anyhow!("GraphQL response errors: {}", msg));
    }
    match r.data {
        Some(data) => Ok(data),
        None => panic!("GraphQL response does not contain data nor errors"),
    }
}

#[derive(Debug)]
pub struct GetReviewRequestsQuery<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub pr_number: usize,
}

impl Query for GetReviewRequestsQuery<'_> {
    type Output = get_review_requests::Payload;

    fn to_json(&self) -> serde_json::Value {
        let query = r#"
            query($owner: String!, $repo: String!, $pr_number: Int!) {
              repository(owner: $owner, name: $repo) {
                pullRequest(number: $pr_number) {
                  id
                  reviewRequests(first: 20) {
                    nodes {
                      requestedReviewer {
                        __typename
                        ... on User {
                          login
                        }
                      }
                    }
                  }
                }
              }
            }
        "#;
        json!({
            "query": query,
            "variables": {
                "owner": self.owner,
                "repo": self.repo,
                "pr_number": self.pr_number,
            },
        })
    }
}

pub mod get_review_requests {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Payload {
        pub repository: Option<Repository>,
    }

    impl Payload {
        pub fn pull_request(&self) -> Option<&PullRequest> {
            self.repository.as_ref()?.pull_request.as_ref()
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Repository {
        pub pull_request: Option<PullRequest>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PullRequest {
        pub id: String,
        pub review_requests: ReviewRequestConn,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReviewRequestConn {
        pub nodes: Vec<ReviewRequest>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReviewRequest {
        pub requested_reviewer: Option<RequestedReviewer>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "__typename")]
    pub enum RequestedReviewer {
        #[serde(rename_all = "camelCase")]
        Mannequin,
        #[serde(rename_all = "camelCase")]
        Team,
        #[serde(rename_all = "camelCase")]
        User { login: String },
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubscriptionInput {
    pub state: SubscriptionState,
    pub subscribable_id: String,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubscriptionState {
    Ignored,
    Subscribed,
    Unsubscribed,
}

#[derive(Debug)]
pub struct UpdateSubscriptionMut {
    pub input: UpdateSubscriptionInput,
}

impl Query for UpdateSubscriptionMut {
    type Output = update_subscription_mut::Payload;

    fn to_json(&self) -> serde_json::Value {
        let query = r#"
            mutation($input: UpdateSubscriptionInput!) {
              updateSubscription(input: $input) {
                subscribable {
                    viewerSubscription
                }
              }
            }
        "#;
        json!({
            "query": query,
            "variables": { "input": &self.input }
        })
    }
}

pub mod update_subscription_mut {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub update_subscription: UpdateSubscription,
    }

    impl Payload {
        pub fn viewer_subscription(&self) -> Option<super::SubscriptionState> {
            self.update_subscription
                .subscribable
                .as_ref()?
                .viewer_subscription
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateSubscription {
        pub subscribable: Option<Subscribable>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Subscribable {
        pub viewer_subscription: Option<super::SubscriptionState>,
    }
}
