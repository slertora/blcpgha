// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::str::FromStr;
use std::net::IpAddr;

mod config;
mod handlers;
mod api;
mod models;
mod static_files;

use config::load_config;
use api::create_api_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config("config.toml")?;
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    tracing::info!("BLC PostgreSQL HA GUI v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Configuration loaded from config.toml");
    
    // Parse host and port from config
    let host = IpAddr::from_str(&config.server.host)?;
    let port = config.server.port;
    let addr = SocketAddr::new(host, port);
    
    tracing::info!("Starting GUI server on http://{}:{}", config.server.host, config.server.port);
    
    // Create router
    let app = Router::new()
        .route("/", get(handlers::dashboard::dashboard_page))
        .route("/dashboard", get(handlers::dashboard::dashboard_page))
        .route("/nodes", get(handlers::nodes::nodes_page))
        .route("/cluster", get(handlers::cluster::cluster_page))
        .route("/metrics", get(handlers::metrics::metrics_page))
        .route("/settings", get(handlers::settings::settings_page))
        .nest("/api/v1", create_api_router())
        .nest_service("/static", static_files::static_handler());
    
    // Start server
    tracing::info!("Listening on {}:{}", config.server.host, config.server.port);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    
    Ok(())
}
