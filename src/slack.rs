mod api;

pub use api::Client;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Credentials {
    pub bot_token: String,
}

pub struct ConvHistoryParams<'a> {
    pub channel: &'a str,
    pub oldest_ts: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RawConvHistoryResponse {
    pub error: Option<String>,
    pub messages: Option<Vec<Message>>,
}

#[derive(Debug)]
pub struct ConvHistoryResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub ts: String,
    pub text: String,
    pub files: Option<Vec<File>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "pretty_type")]
pub enum File {
    Unknown,
    Email {
        subject: String,
        to: Vec<EmailAddress>,
        from: Vec<EmailAddress>,
        plain_text: String,
    },
}

impl Default for File {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub name: String,
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
