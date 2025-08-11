// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

mod api;
mod components;
mod config;
mod handlers;
mod models;
mod static_files;
mod templates;
mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to bind to
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if args.verbose { Level::DEBUG } else { Level::INFO })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("BLC PostgreSQL HA GUI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::load_config(&args.config)?;
    info!("Configuration loaded from {}", args.config);

    // Create CORS layer
    let cors = CorsLayer::permissive();

    // Build application router
    let app = Router::new()
        // Static files
        .route("/", get(handlers::dashboard::index))
        .route("/dashboard", get(handlers::dashboard::index))
        .route("/nodes", get(handlers::nodes::index))
        .route("/cluster", get(handlers::cluster::index))
        .route("/metrics", get(handlers::metrics::index))
        .route("/settings", get(handlers::settings::index))
        
        // API endpoints
        .route("/api/v1/cluster/status", get(api::cluster::get_status))
        .route("/api/v1/nodes", get(api::nodes::list_nodes))
        .route("/api/v1/nodes/:node_id", get(api::nodes::get_node))
        .route("/api/v1/nodes/:node_id/promote", post(api::nodes::promote_node))
        .route("/api/v1/nodes/:node_id/demote", post(api::nodes::demote_node))
        .route("/api/v1/nodes/:node_id/disable", post(api::nodes::disable_node))
        .route("/api/v1/nodes/:node_id/enable", post(api::nodes::enable_node))
        .route("/api/v1/metrics", get(api::metrics::get_metrics))
        .route("/api/v1/health", get(api::health::get_health))
        .route("/api/v1/vip/status", get(api::vip::get_status))
        
        // WebSocket for real-time updates
        .route("/ws", get(websocket::handler))
        
        // Static file serving
        .nest_service("/static", static_files::static_handler())
        
        .layer(cors);

    // Bind to address
    let addr = SocketAddr::from((args.host.parse()?, args.port));
    info!("Starting GUI server on http://{}", addr);

    // Start server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
