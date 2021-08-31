mod api;

pub use api::Client;

use crate::email::Email;
use serde::{Deserialize, Serialize};
use std::mem;

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

pub fn extract_email_from_message(msg: Message) -> Option<Email> {
    match msg.files {
        Some(mut files) if !files.is_empty() => {
            let file = mem::take(&mut files[0]);
            match file {
                File::Email {
                    mut from,
                    subject,
                    plain_text,
                    ..
                } => {
                    let sender_name = mem::take(&mut from[0].name);
                    Some(Email {
                        subject,
                        sender_name,
                        text_body: plain_text,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}
