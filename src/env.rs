use anyhow::{Context, Result};

pub fn must_get_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("environment variable required: {}", name))
}
