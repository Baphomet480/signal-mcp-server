use std::path::PathBuf;

use anyhow::{Context, Result};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_signal_cli_path")]
    pub signal_cli_path: PathBuf,
    pub account: String,
    #[serde(default = "default_storage_directory")]
    pub storage: PathBuf,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix("SIGNAL_MCP").separator("__"));

        let config = builder
            .build()
            .map_err(map_config_error)
            .context("failed to build configuration")?;

        config
            .try_deserialize::<Settings>()
            .map_err(map_config_error)
            .context("failed to deserialize configuration")
    }
}

fn map_config_error(err: ConfigError) -> anyhow::Error {
    match err {
        ConfigError::NotFound(_) => err.into(),
        _ => anyhow::anyhow!(err),
    }
}

fn default_signal_cli_path() -> PathBuf {
    PathBuf::from("/usr/bin/signal-cli")
}

fn default_storage_directory() -> PathBuf {
    PathBuf::from("./var")
}
