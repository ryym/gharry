use anyhow::{Context, Result};
use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub slack: SlackConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let dir = [&must_get_env("HOME")?, ".gharry"].iter().collect();
        let slack = SlackConfig::from_env()?;
        Ok(Config { dir, slack })
    }
}

#[derive(Debug)]
pub struct SlackConfig {
    pub bot_token: String,
    pub mail_channel_id: String,
}

impl SlackConfig {
    fn from_env() -> Result<Self> {
        let bot_token = must_get_env("SLACK_OAUTH_BOT_TOKEN")?;
        let mail_channel_id = must_get_env("SLACK_MAIL_CHANNEL_ID")?;
        Ok(SlackConfig {
            bot_token,
            mail_channel_id,
        })
    }
}

fn must_get_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("environment variable required: {}", name))
}
