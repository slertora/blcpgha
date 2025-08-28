// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::MetricsConfig;
use crate::health::HealthChecker;
use crate::raft::RaftNode;
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use prometheus_client::{
    encoding::text::encode,
    metrics::{counter::Counter, gauge::Gauge, histogram::Histogram},
    registry::Registry,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};

pub struct MetricsServer {
    config: MetricsConfig,
    registry: Arc<Registry>,
    health_checker: Arc<HealthChecker>,
    raft_node: Arc<RaftNode>,
}

// Define metrics
#[derive(Clone)]
struct Metrics {
    health_status: Gauge,
    health_score: Gauge,
    replication_lag_seconds: Gauge,
    is_primary: Gauge,
    error_count: Counter,
    consecutive_failures: Counter,
    health_check_duration: Histogram,
    raft_is_leader: Gauge,
    raft_current_term: Gauge,
    raft_commit_index: Gauge,
    raft_last_applied: Gauge,
    uptime_seconds: Gauge,
}

impl MetricsServer {
    pub fn new(
        config: MetricsConfig,
        health_checker: Arc<HealthChecker>,
        raft_node: Arc<RaftNode>,
    ) -> Self {
        let mut registry = Registry::default();
        let health_checker = health_checker;
        let raft_node = raft_node;

        // Create metrics
        let metrics = Metrics {
            health_status: Gauge::default(),
            health_score: Gauge::default(),
            replication_lag_seconds: Gauge::default(),
            is_primary: Gauge::default(),
            error_count: Counter::default(),
            consecutive_failures: Counter::default(),
            health_check_duration: Histogram::new(
                [0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0].into_iter(),
            ),
            raft_is_leader: Gauge::default(),
            raft_current_term: Gauge::default(),
            raft_commit_index: Gauge::default(),
            raft_last_applied: Gauge::default(),
            uptime_seconds: Gauge::default(),
        };

        // Register metrics
        registry.register(
            "blcpg_health_status",
            "Current health status (1=healthy, 0=unhealthy)",
            metrics.health_status.clone(),
        );
        registry.register(
            "blcpg_health_score",
            "Health score from 0 to 100",
            metrics.health_score.clone(),
        );
        registry.register(
            "blcpg_replication_lag_seconds",
            "Replication lag in seconds",
            metrics.replication_lag_seconds.clone(),
        );
        registry.register(
            "blcpg_is_primary",
            "Whether this node is primary (1=primary, 0=replica)",
            metrics.is_primary.clone(),
        );
        registry.register(
            "blcpg_error_count",
            "Total number of health check errors",
            metrics.error_count.clone(),
        );
        registry.register(
            "blcpg_consecutive_failures",
            "Number of consecutive health check failures",
            metrics.consecutive_failures.clone(),
        );
        registry.register(
            "blcpg_health_check_duration_seconds",
            "Duration of health checks in seconds",
            metrics.health_check_duration.clone(),
        );
        registry.register(
            "blcpg_raft_is_leader",
            "Whether this Raft node is leader (1=leader, 0=follower)",
            metrics.raft_is_leader.clone(),
        );
        registry.register(
            "blcpg_raft_current_term",
            "Current Raft term",
            metrics.raft_current_term.clone(),
        );
        registry.register(
            "blcpg_raft_commit_index",
            "Raft commit index",
            metrics.raft_commit_index.clone(),
        );
        registry.register(
            "blcpg_raft_last_applied",
            "Raft last applied index",
            metrics.raft_last_applied.clone(),
        );
        registry.register(
            "blcpg_uptime_seconds",
            "Service uptime in seconds",
            metrics.uptime_seconds.clone(),
        );

        let server = Self {
            config,
            registry: Arc::new(registry),
            health_checker,
            raft_node,
        };

        // Start metrics collection
        server.start_metrics_collection(metrics);

        server
    }

    fn start_metrics_collection(&self, metrics: Metrics) {
        let health_checker = Arc::clone(&self.health_checker);
        let raft_node = Arc::clone(&self.raft_node);
        let start_time = std::time::Instant::now();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Update uptime
                metrics
                    .uptime_seconds
                    .set(start_time.elapsed().as_secs() as i64);

                // Update health metrics
                let health_status = health_checker.get_status().await;
                metrics
                    .health_status
                    .set(if health_status.is_healthy { 1 } else { 0 });
                metrics.health_score.set(health_status.score as i64);
                metrics
                    .is_primary
                    .set(if health_status.is_primary { 1 } else { 0 });
                metrics.error_count.inc_by(health_status.error_count as u64);
                metrics
                    .consecutive_failures
                    .inc_by(health_status.consecutive_failures as u64);

                // Update replication lag
                if let Some(lag) = health_status.replication_lag {
                    let lag_seconds = lag as f64 / 1_000_000.0; // Convert microseconds to seconds
                    metrics.replication_lag_seconds.set(lag_seconds as i64);
                } else {
                    metrics.replication_lag_seconds.set(-1); // Unknown
                }

                // Update Raft metrics
                let raft_state = raft_node.get_state().await;
                metrics
                    .raft_is_leader
                    .set(if raft_node.is_leader().await { 1 } else { 0 });
                metrics
                    .raft_current_term
                    .set(raft_state.current_term as i64);
                metrics
                    .raft_commit_index
                    .set(raft_state.commit_index as i64);
                metrics
                    .raft_last_applied
                    .set(raft_state.last_applied as i64);

                debug!("Metrics updated");
            }
        });
    }

    pub async fn run(&self) -> Result<()> {
        let addr = self.config.prometheus.as_str();
        info!("Starting metrics server on {}", addr);

        // Create router with metrics endpoint
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(self.registry.clone());

        // Start server
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}

async fn metrics_handler(
    State(registry): State<Arc<Registry>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut buffer = String::new();
    encode(&mut buffer, &registry).map_err(|e| {
        error!("Failed to encode metrics: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics".to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        buffer,
    ))
}

// Additional metrics functions for manual updates
pub async fn record_health_check_duration(duration: Duration) {
    // This would be called from the health checker
    debug!("Health check duration: {:?}", duration);
}

pub async fn record_failover_event() {
    // This would be called when a failover occurs
    info!("Failover event recorded");
}
