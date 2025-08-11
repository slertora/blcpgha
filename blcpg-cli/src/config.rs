// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub auth_token: String,
    pub timeout: String,
    pub retry_attempts: u32,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                host: "localhost".to_string(),
                port: 8009,
                auth_token: "your_auth_token_here".to_string(),
                timeout: "30s".to_string(),
                retry_attempts: 3,
            },
        }
    }
}

impl CliConfig {
    pub fn validate(&self) -> Result<()> {
        if self.api.host.is_empty() {
            return Err(anyhow::anyhow!("API host cannot be empty"));
        }
        if self.api.port == 0 {
            return Err(anyhow::anyhow!("API port must be greater than 0"));
        }
        if self.api.auth_token.is_empty() {
            return Err(anyhow::anyhow!("API auth token cannot be empty"));
        }
        if self.api.retry_attempts == 0 {
            return Err(anyhow::anyhow!("API retry attempts must be greater than 0"));
        }

        // Validate timeout format
        self.validate_duration_string(&self.api.timeout, "api.timeout")?;

        Ok(())
    }

    fn validate_duration_string(&self, duration: &str, field_name: &str) -> Result<()> {
        humantime::Duration::from_str(duration)
            .map_err(|e| anyhow::anyhow!("Invalid duration format for {}: {}", field_name, e))?;
        Ok(())
    }
}

pub fn load_config(config_path: &str) -> Result<CliConfig> {
    let config_builder = config::Config::builder();

    let config_builder = if Path::new(config_path).exists() {
        config_builder.add_source(config::File::with_name(config_path))
    } else {
        config_builder
    };

    let config_builder = config_builder
        .add_source(config::Environment::with_prefix("BLCPG_CLI"))
        .set_default("api.host", "localhost")?
        .set_default("api.port", 8009)?
        .set_default("api.auth_token", "your_auth_token_here")?
        .set_default("api.timeout", "30s")?
        .set_default("api.retry_attempts", 3)?;

    let config = config_builder.build()?;
    let cli_config: CliConfig = config.try_deserialize()?;

    cli_config.validate()?;

    Ok(cli_config)
} 