// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

mod config;
mod postgresql;
mod logging;
mod raft;
mod health;
mod metrics;
mod api;
mod cluster;
mod vip;


use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first
    let config = config::AppConfig::load()?;
    info!("Configuration loaded successfully");
    
    // Initialize logging with config
    logging::init(&config.logging)?;
    
    info!("Starting BLC PostgreSQL HA Agent v{}", env!("CARGO_PKG_VERSION"));
    
    // Initialize PostgreSQL connection (optional - may fail if PostgreSQL is down)
    let pg_conn = match postgresql::connect(&config.postgresql).await {
        Ok(conn) => {
            info!("PostgreSQL connection established");
            Some(Arc::new(conn))
        }
        Err(e) => {
            warn!("Failed to connect to PostgreSQL: {}. Agent will start in degraded mode.", e);
            warn!("PostgreSQL connection will be retried periodically");
            None
        }
    };
    
    // Initialize cluster manager first
    let cluster_manager = Arc::new(cluster::ClusterManager::new(config.cluster.clone()));
    info!("Cluster manager initialized");
    
    // Initialize Raft cluster
    let raft_node = Arc::new(raft::RaftNode::new(config.raft.clone(), cluster_manager.clone()).await?);
    info!("Raft node initialized");
    
    // Initialize health checker (with optional PostgreSQL connection)
    let health_checker = Arc::new(health::HealthChecker::new(
        config.health.clone(),
        config.critical_failover.clone(),
        pg_conn.clone(),
        cluster_manager.clone(),
    ));
    info!("Health checker initialized");
    
    // Initialize metrics server
    let metrics_server = Arc::new(metrics::MetricsServer::new(
        config.metrics.clone(),
        health_checker.clone(),
        raft_node.clone(),
    ));
    info!("Metrics server initialized");
    
    // Initialize VIP manager
    let vip_manager = Arc::new(vip::VipManager::new(config.vip.clone()).await?);
    info!("VIP manager initialized");
    
    // Initialize API server
    let api_server = Arc::new(api::ApiServer::new(
        config.api.clone(),
        health_checker.clone(),
        raft_node.clone(),
        cluster_manager.clone(),
    ));
    info!("API server initialized");
    
    // Start all services concurrently
    let health_handle = tokio::spawn(async move { health_checker.run().await });
    let raft_handle = tokio::spawn(async move { raft_node.run().await });
    let cluster_handle = tokio::spawn(async move { cluster_manager.run().await });
    let metrics_handle = tokio::spawn(async move { metrics_server.run().await });
    let api_handle = tokio::spawn(async move { api_server.run().await });
    let vip_handle = tokio::spawn(async move { 
        // VIP manager runs as a background service
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    // Wait for all services
    tokio::try_join!(
        health_handle,
        raft_handle,
        cluster_handle,
        metrics_handle,
        api_handle,
        vip_handle
    )?;
    
    info!("All services stopped");
    
    Ok(())
} 