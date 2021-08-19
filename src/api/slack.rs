use anyhow::{anyhow, Result};
use reqwest::blocking::{RequestBuilder, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
    creds: Credentials,
}

#[derive(Debug)]
pub struct Credentials {
    pub bot_token: String,
}

impl Client {
    pub fn new(creds: Credentials) -> Client {
        let client = reqwest::blocking::Client::new();
        Client { client, creds }
    }

    fn fetch(&self, r: RequestBuilder) -> reqwest::Result<Response> {
        r.header("Authorization", format!("Bearer {}", self.creds.bot_token))
            .send()
    }

    pub fn conversations_history(&self, params: ConvHistoryParams) -> Result<ConvHistoryResponse> {
        let req = self
            .client
            .get("https://slack.com/api/conversations.history")
            .query(&[
                ("channel", params.channel),
                ("oldest", params.oldest_ts),
                ("limit", "3"), // XXX: temporarily
            ]);
        let res = self.fetch(req)?;
        let data = res.json::<RawConvHistoryResponse>()?;

        if let Some(messages) = data.messages {
            return Ok(ConvHistoryResponse { messages });
        }

        let err_msg = data.error.unwrap_or_else(|| String::from("unknown error"));
        Err(anyhow!("failed to fetch conversation history: {}", err_msg))
    }

    pub fn chat_post_message(&self, msg: &ChatMessage) -> Result<()> {
        let body = serde_json::ser::to_string(&msg)?;
        let req = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Content-Type", "application/json")
            .body(body);
        let res = self.fetch(req)?;
        let data = res.json::<RawChatPostMsgResponse>()?;

        if let Some(err_msg) = data.error {
            return Err(anyhow!("failed to post message: {}", err_msg));
        }
        Ok(())
    }
}

pub struct ConvHistoryParams<'a> {
    pub channel: &'a str,
    pub oldest_ts: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RawConvHistoryResponse {
    pub error: Option<String>,
    pub messages: Option<Vec<ConversationMessage>>,
}

#[derive(Debug)]
pub struct ConvHistoryResponse {
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationMessage {
    pub ts: String,
    pub text: String,
    pub files: Option<Vec<EmailFile>>,
}

#[derive(Debug, Deserialize)]
pub struct EmailFile {
    pub subject: String,
    pub to: Vec<EmailAddress>,
    pub from: Vec<EmailAddress>,
    pub plain_text: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailAddress {
    address: String,
    name: String,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub channel: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct RawChatPostMsgResponse {
    pub error: Option<String>,
}
