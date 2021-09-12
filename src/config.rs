use crate::env::must_get_env;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub slack: SlackConfig,
    pub github: GitHubConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let dir = [&must_get_env("HOME")?, ".gharry"].iter().collect();
        let slack = SlackConfig::from_env()?;
        let github = GitHubConfig::from_env()?;
        Ok(Self { dir, slack, github })
    }
}

#[derive(Debug)]
pub struct SlackConfig {
    pub bot_token: String,
    pub mail_channel_id: String,
    pub dest_channel_id: String,
}

impl SlackConfig {
    fn from_env() -> Result<Self> {
        let bot_token = must_get_env("SLACK_OAUTH_BOT_TOKEN")?;
        let mail_channel_id = must_get_env("SLACK_MAIL_CHANNEL_ID")?;
        let dest_channel_id = must_get_env("SLACK_DEST_CHANNEL_ID")?;
        Ok(Self {
            bot_token,
            mail_channel_id,
            dest_channel_id,
        })
    }
}

#[derive(Debug)]
pub struct GitHubConfig {
    pub auth_token: String,
}

impl GitHubConfig {
    fn from_env() -> Result<Self> {
        let auth_token = must_get_env("GITHUB_ACCESS_TOKEN")?;
        Ok(Self { auth_token })
    }
}
