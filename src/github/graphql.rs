use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize};

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
