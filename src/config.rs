use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct RawConfig {
    pub slack_oauth_bot_token: String,
    pub slack_mail_channel_id: String,
    pub slack_dest_channel_id: String,
    pub github_access_token: String,
    pub github_login_name: String,
}

impl RawConfig {
    pub fn from_file(work_dir: &Path) -> Result<Self> {
        let config_path = {
            let mut path = work_dir.to_path_buf();
            path.push("config.toml");
            path
        };
        if !config_path.as_path().exists() {
            return Err(anyhow!("config file not found: {}", config_path.display()));
        }
        let file_content = fs::read_to_string(&config_path)?;
        toml::from_str(&file_content).with_context(|| "failed to parse config file")
    }
}

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub slack: SlackConfig,
    pub github: GitHubConfig,
}

impl Config {
    pub fn build_default(work_dir: PathBuf) -> Result<Self> {
        let raw = RawConfig::from_file(&work_dir)?;
        Ok(Config {
            dir: work_dir,
            slack: SlackConfig {
                bot_token: raw.slack_oauth_bot_token,
                mail_channel_id: raw.slack_mail_channel_id,
                dest_channel_id: raw.slack_dest_channel_id,
            },
            github: GitHubConfig {
                auth_token: raw.github_access_token,
                login_name: raw.github_login_name,
            },
        })
    }
}

#[derive(Debug)]
pub struct SlackConfig {
    pub bot_token: String,
    pub mail_channel_id: String,
    pub dest_channel_id: String,
}

#[derive(Debug)]
pub struct GitHubConfig {
    pub auth_token: String,
    pub login_name: String,
}
