// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::LoggingConfig;
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init(config: &LoggingConfig) -> Result<()> {
    // Build environment filter from config
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            let level = &config.level;
            format!("blcpg_ha={},tower_http=info", level).into()
        });

    // Initialize tracing with configurable settings
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Logging initialized with level: {}", config.level);
    Ok(())
} 