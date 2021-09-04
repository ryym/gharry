mod api;

pub use api::Client;

use crate::email::Email;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug)]
pub struct Credentials {
    pub bot_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub ts: String,
    pub text: String,
    pub files: Option<Vec<File>>,
}

#[derive(Debug)]
pub struct ConvHistoryParams<'a> {
    pub channel: &'a str,
    pub oldest_ts: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage<'a> {
    pub channel: &'a str,
    pub text: &'a str,
    pub username: Option<&'a str>,
    pub icon_url: Option<&'a str>,
    pub icon_emoji: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub name: String,
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
