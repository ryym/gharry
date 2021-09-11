use crate::{
    slack::{ChatMessage, ConvHistoryParams, Credentials, Message},
    web,
};
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
}

#[derive(Debug, Deserialize)]
struct RawConvHistoryResponse {
    pub error: Option<String>,
    pub messages: Option<Vec<Message>>,
}

#[derive(Debug)]
pub struct ConvHistoryResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct RawChatPostMsgResponse {
    pub error: Option<String>,
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
            header::HeaderValue::from_str(&format!("Bearer {}", creds.bot_token))?,
        );

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(client)
    }

    pub fn conversations_history(&self, params: ConvHistoryParams) -> Result<ConvHistoryResponse> {
        let url = "https://slack.com/api/conversations.history";
        let res = self
            .client
            .get(url)
            .query(&[("channel", params.channel), ("oldest", params.oldest_ts)])
            .send()?;

        if res.status().as_u16() != 200 {
            web::log_error_response(&url, res);
            return Err(anyhow!("failed to fetch conversations history"));
        }

        let data = res.json::<RawConvHistoryResponse>()?;
        match data.messages {
            Some(messages) => Ok(ConvHistoryResponse { messages }),
            None => {
                let err_msg = data.error.unwrap_or_else(|| String::from("unknown error"));
                Err(anyhow!("failed to fetch conversation history: {}", err_msg))
            }
        }
    }

    pub fn chat_post_message(&self, msg: &ChatMessage) -> Result<()> {
        let url = "https://slack.com/api/chat.postMessage";
        let body = serde_json::ser::to_string(&msg)?;
        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()?;

        if res.status().as_u16() != 200 {
            web::log_error_response(&url, res);
            return Err(anyhow!("failed to post chat message"));
        }

        let data = res.json::<RawChatPostMsgResponse>()?;
        if let Some(err_msg) = data.error {
            return Err(anyhow!("failed to post chat message: {}", err_msg));
        }

        Ok(())
    }
}
