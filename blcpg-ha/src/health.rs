// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::cluster::ClusterManager;
use crate::config::{CriticalFailoverConfig, HealthConfig};
use crate::postgresql::PostgreSQLConnection;
use anyhow::Result;
use chrono;
use humantime::Duration as HumanDuration;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub score: f64,
    pub last_check: i64,
    pub replication_lag: Option<u64>,
    pub is_primary: bool,
    pub server_version: Option<String>,
    pub current_lsn: Option<String>,
    pub error_count: u32,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone)]
pub struct HealthChecker {
    config: HealthConfig,
    failover_config: CriticalFailoverConfig,
    pg_conn: Option<Arc<PostgreSQLConnection>>,
    cluster_manager: Arc<ClusterManager>,
    status: Arc<RwLock<HealthStatus>>,
    shutdown_tx: mpsc::Sender<()>,
}

impl HealthChecker {
    pub fn new(
        health_config: HealthConfig,
        failover_config: CriticalFailoverConfig,
        pg_conn: Option<Arc<PostgreSQLConnection>>,
        cluster_manager: Arc<ClusterManager>,
    ) -> Self {
        let status = Arc::new(RwLock::new(HealthStatus {
            is_healthy: false,
            score: 0.0,
            last_check: chrono::Utc::now().timestamp(),
            replication_lag: None,
            is_primary: false,
            server_version: None,
            current_lsn: None,
            error_count: 0,
            consecutive_failures: 0,
        }));

        let (shutdown_tx, _) = mpsc::channel::<()>(1);

        Self {
            config: health_config,
            failover_config,
            pg_conn,
            cluster_manager,
            status,
            shutdown_tx,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            "Starting health checker with interval: {}",
            self.config.interval
        );

        let interval_duration = HumanDuration::from_str(&self.config.interval)
            .map_err(|e| anyhow::anyhow!("Invalid health interval: {}", e))?;

        let mut interval = interval(interval_duration.into());

        loop {
            interval.tick().await;

            if let Err(e) = self.perform_health_check().await {
                error!("Health check failed: {}", e);
                self.update_error_status().await;
            }
        }
    }

    async fn perform_health_check(&self) -> Result<()> {
        let start_time = Instant::now();

        // Check if we have a PostgreSQL connection
        let pg_conn = match &self.pg_conn {
            Some(conn) => conn,
            None => {
                // No PostgreSQL connection available - try to reconnect
                self.attempt_reconnection().await;

                // Update status to reflect this
                let mut status = self.status.write().await;
                status.is_healthy = false;
                status.score = 0.0;
                status.last_check = chrono::Utc::now().timestamp();
                status.replication_lag = None;
                status.is_primary = false;
                status.server_version = None;
                status.current_lsn = None;
                status.consecutive_failures += 1;

                warn!("PostgreSQL connection not available - health check skipped");
                return Err(anyhow::anyhow!("PostgreSQL connection not available"));
            }
        };

        // Basic connection health check
        let connection_healthy = match pg_conn.health_check().await {
            Ok(healthy) => healthy,
            Err(e) => {
                warn!("PostgreSQL health check failed: {}", e);
                self.update_error_status().await;
                return Err(e);
            }
        };

        if !connection_healthy {
            self.update_error_status().await;
            return Err(anyhow::anyhow!("PostgreSQL connection unhealthy"));
        }

        // Get detailed health information
        let replication_lag = pg_conn.get_replication_lag().await?;
        let is_primary = pg_conn.is_primary().await?;
        let server_version = pg_conn.get_server_version().await?;
        let current_lsn = pg_conn.get_current_wal_lsn().await?;

        // Calculate health score
        let score = self.calculate_health_score(
            connection_healthy,
            replication_lag,
            is_primary,
            &server_version,
        );

        // Update status
        let mut status = self.status.write().await;
        status.is_healthy = connection_healthy && score >= self.failover_config.min_health_score;
        status.score = score;
        status.last_check = chrono::Utc::now().timestamp();
        status.replication_lag = replication_lag;
        status.is_primary = is_primary;
        status.server_version = server_version;
        status.current_lsn = current_lsn;
        status.consecutive_failures = 0;

        let check_duration = start_time.elapsed();
        debug!(
            "Health check completed in {:?} - Score: {:.2}, Primary: {}, Lag: {:?}",
            check_duration, score, is_primary, replication_lag
        );

        Ok(())
    }

    async fn attempt_reconnection(&self) {
        // This is a placeholder for now - in a real implementation,
        // we would need to pass the PostgreSQL config to the HealthChecker
        // and attempt to establish a new connection
        debug!("Attempting to reconnect to PostgreSQL...");
        // TODO: Implement reconnection logic
    }

    fn calculate_health_score(
        &self,
        connection_healthy: bool,
        replication_lag: Option<u64>,
        is_primary: bool,
        server_version: &Option<String>,
    ) -> f64 {
        let mut score = 0.0_f32;

        // Connection health (40% weight)
        if connection_healthy {
            score += 40.0;
        }

        // Primary status (30% weight)
        if is_primary {
            score += 30.0;
        } else {
            // Replica health based on lag
            if let Some(lag) = replication_lag {
                let lag_seconds = lag as f64 / 1_000_000.0; // Convert microseconds to seconds
                if lag_seconds < 1.0 {
                    score += 30.0; // Excellent lag
                } else if lag_seconds < 5.0 {
                    score += 25.0; // Good lag
                } else if lag_seconds < 10.0 {
                    score += 20.0; // Acceptable lag
                } else if lag_seconds < 30.0 {
                    score += 10.0; // Poor lag
                } else {
                    score += 5.0; // Very poor lag
                }
            } else {
                score += 15.0; // Unknown lag
            }
        }

        // Server version (10% weight)
        if server_version.is_some() {
            score += 10.0;
        }

        // Replication lag within limits (20% weight)
        if let Some(lag) = replication_lag {
            let lag_bytes = lag as u64;
            if lag_bytes <= self.failover_config.max_lag_bytes {
                score += 20.0;
            } else {
                score += 5.0; // Penalty for excessive lag
            }
        } else {
            score += 10.0; // Unknown lag
        }

        score.min(100.0_f32) as f64 // Cap at 100%
    }

    async fn should_auto_promote(&self) -> bool {
        let status = self.status.read().await;

        // Check if we're healthy enough
        if status.score < self.failover_config.min_health_score {
            return false;
        }

        // Check if we're not already primary
        if status.is_primary {
            return false;
        }

        // Check replication lag
        if let Some(lag) = status.replication_lag {
            let lag_bytes = lag as u64;
            if lag_bytes > self.failover_config.max_lag_bytes {
                return false;
            }
        }

        true
    }

    async fn update_error_status(&self) {
        let mut status = self.status.write().await;
        status.is_healthy = false;
        status.score = 0.0;
        status.error_count += 1;
        status.consecutive_failures += 1;
        status.last_check = chrono::Utc::now().timestamp();
    }

    pub async fn get_status(&self) -> HealthStatus {
        self.status.read().await.clone()
    }

    pub async fn is_healthy(&self) -> bool {
        let status = self.status.read().await;
        status.is_healthy
    }

    pub async fn get_score(&self) -> f64 {
        let status = self.status.read().await;
        status.score
    }

    pub async fn get_replication_lag(&self) -> Option<u64> {
        let status = self.status.read().await;
        status.replication_lag
    }

    pub async fn is_primary(&self) -> bool {
        let status = self.status.read().await;
        status.is_primary
    }

    pub async fn get_server_version(&self) -> Option<String> {
        let status = self.status.read().await;
        status.server_version.clone()
    }

    pub async fn get_current_lsn(&self) -> Option<String> {
        let status = self.status.read().await;
        status.current_lsn.clone()
    }

    pub async fn get_error_count(&self) -> u32 {
        let status = self.status.read().await;
        status.error_count
    }

    pub async fn get_consecutive_failures(&self) -> u32 {
        let status = self.status.read().await;
        status.consecutive_failures
    }

    pub async fn force_health_check(&self) -> Result<()> {
        info!("Forcing health check");
        self.perform_health_check().await
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down health checker");
        self.shutdown_tx.send(()).await?;
        Ok(())
    }
}
