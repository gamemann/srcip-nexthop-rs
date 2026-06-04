use anyhow::{Result, anyhow};
use tokio::fs;

use crate::config::cfg::Config;

impl Config {
    pub async fn from_file(&mut self, path: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| anyhow!("Failed to read file {}: {}", path, e))?;

        *self = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse JSON from file {}: {}", path, e))?;

        Ok(())
    }
}
