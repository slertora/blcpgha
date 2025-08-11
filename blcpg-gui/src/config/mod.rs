// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout: u64,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub title: String,
    pub theme: String,
    pub refresh_interval: u64,
    pub auto_refresh: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                log_level: "info".to_string(),
            },
            api: ApiConfig {
                base_url: "http://localhost:8080".to_string(),
                timeout: 30,
                retries: 3,
            },
            database: DatabaseConfig {
                url: "sqlite:gui.db".to_string(),
                max_connections: 10,
            },
            ui: UiConfig {
                title: "BLC PostgreSQL HA".to_string(),
                theme: "dark".to_string(),
                refresh_interval: 5,
                auto_refresh: true,
            },
        }
    }
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let mut config = config::Config::default();
    
    // Load default configuration
    config.set_default("server.host", "0.0.0.0")?;
    config.set_default("server.port", 3000)?;
    config.set_default("server.log_level", "info")?;
    config.set_default("api.base_url", "http://localhost:8080")?;
    config.set_default("api.timeout", 30)?;
    config.set_default("api.retries", 3)?;
    config.set_default("database.url", "sqlite:gui.db")?;
    config.set_default("database.max_connections", 10)?;
    config.set_default("ui.title", "BLC PostgreSQL HA")?;
    config.set_default("ui.theme", "dark")?;
    config.set_default("ui.refresh_interval", 5)?;
    config.set_default("ui.auto_refresh", true)?;

    // Load configuration file if it exists
    if std::path::Path::new(path).exists() {
        config.merge(config::File::with_name(path))?;
    }

    // Load environment variables
    config.merge(config::Environment::with_prefix("BLCGUI"))?;

    // Deserialize into our Config struct
    let config: Config = config.try_deserialize()?;
    Ok(config)
} 