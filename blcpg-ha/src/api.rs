// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::cluster::ClusterManager;
use crate::config::ApiConfig;
use crate::health::HealthChecker;
use crate::raft::RaftNode;
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

pub struct ApiServer {
    config: ApiConfig,
    health_checker: Arc<HealthChecker>,
    raft_node: Arc<RaftNode>,
    cluster_manager: Arc<ClusterManager>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    health: HealthInfo,
    raft: RaftInfo,
}

#[derive(Debug, Serialize)]
struct HealthInfo {
    is_healthy: bool,
    score: f64,
    is_primary: bool,
    replication_lag_seconds: Option<f64>,
    server_version: Option<String>,
    current_lsn: Option<String>,
    error_count: u32,
    consecutive_failures: u32,
    last_check: String,
}

#[derive(Debug, Serialize)]
struct RaftInfo {
    node_id: String,
    is_leader: bool,
    current_term: u64,
    leader_id: Option<String>,
    commit_index: u64,
    last_applied: u64,
    peers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ClusterResponse {
    nodes: Vec<NodeInfo>,
    leader: Option<String>,
    healthy_nodes: u32,
    total_nodes: u32,
}

#[derive(Debug, Serialize)]
struct NodeInfo {
    node_id: String,
    is_leader: bool,
    is_healthy: bool,
    health_score: f64,
    is_primary: bool,
    replication_lag_seconds: Option<f64>,
    last_seen: String,
}

#[derive(Debug, Deserialize)]
struct PromoteRequest {
    force: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BroadcastHealthRequest {
    node_id: String,
    health_status: crate::health::HealthStatus,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct BroadcastRaftRequest {
    node_id: String,
    raft_state: crate::raft::RaftState,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct PingRequest {
    node_id: String,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct BroadcastResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct PingResponse {
    node_id: String,
    timestamp: i64,
    alive: bool,
}

#[derive(Debug, Serialize)]
struct PromoteResponse {
    success: bool,
    message: String,
    new_primary: Option<String>,
}

impl ApiServer {
    pub fn new(
        config: ApiConfig,
        health_checker: Arc<HealthChecker>,
        raft_node: Arc<RaftNode>,
        cluster_manager: Arc<ClusterManager>,
    ) -> Self {
        Self {
            config,
            health_checker,
            raft_node,
            cluster_manager,
        }
    }

    // Cluster communication endpoints
    async fn broadcast_health_handler(
        State(server): State<Arc<ApiServer>>,
        Json(payload): Json<BroadcastHealthRequest>,
    ) -> impl IntoResponse {
        info!("Received health broadcast from node: {}", payload.node_id);

        match server
            .cluster_manager
            .update_peer_health(&payload.node_id, payload.health_status)
            .await
        {
            Ok(_) => (
                StatusCode::OK,
                Json(BroadcastResponse {
                    success: true,
                    message: "Health status updated".to_string(),
                }),
            )
                .into_response(),
            Err(e) => {
                error!("Failed to update peer health: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(BroadcastResponse {
                        success: false,
                        message: e.to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }

    async fn broadcast_raft_handler(
        State(server): State<Arc<ApiServer>>,
        Json(payload): Json<BroadcastRaftRequest>,
    ) -> impl IntoResponse {
        info!("Received Raft broadcast from node: {}", payload.node_id);

        match server
            .cluster_manager
            .update_peer_raft_state(&payload.node_id, payload.raft_state)
            .await
        {
            Ok(_) => (
                StatusCode::OK,
                Json(BroadcastResponse {
                    success: true,
                    message: "Raft state updated".to_string(),
                }),
            )
                .into_response(),
            Err(e) => {
                error!("Failed to update peer Raft state: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(BroadcastResponse {
                        success: false,
                        message: e.to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }

    async fn peers_handler(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
        let cluster_state = server.cluster_manager.get_state().await;

        (StatusCode::OK, Json(cluster_state)).into_response()
    }

    async fn ping_handler(
        State(server): State<Arc<ApiServer>>,
        Json(payload): Json<PingRequest>,
    ) -> impl IntoResponse {
        info!("Received ping from node: {}", payload.node_id);

        let response = PingResponse {
            node_id: server.cluster_manager.config.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            alive: true,
        };

        (StatusCode::OK, Json(response)).into_response()
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Starting API server on {}", addr);

        // Create router with all endpoints
        let app = Router::new()
            .route("/status", get(Self::status_handler))
            .route("/cluster", get(Self::cluster_handler))
            .route("/metrics", get(Self::metrics_handler))
            .route("/promote", post(Self::promote_handler))
            .route(
                "/cluster/broadcast/health",
                post(Self::broadcast_health_handler),
            )
            .route(
                "/cluster/broadcast/raft",
                post(Self::broadcast_raft_handler),
            )
            .route("/cluster/peers", get(Self::peers_handler))
            .route("/cluster/ping", post(Self::ping_handler))
            .with_state(Arc::new(self.clone()));

        // Start server
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }

    async fn status_handler(
        State(server): State<Arc<ApiServer>>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        // Check authentication
        if let Err(e) = server.authenticate(&headers).await {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }

        let start_time = std::time::Instant::now();
        let uptime = start_time.elapsed().as_secs();

        // Get health status
        let health_status = server.health_checker.get_status().await;
        let health_info = HealthInfo {
            is_healthy: health_status.is_healthy,
            score: health_status.score,
            is_primary: health_status.is_primary,
            replication_lag_seconds: health_status
                .replication_lag
                .map(|lag| lag as f64 / 1_000_000.0),
            server_version: health_status.server_version,
            current_lsn: health_status.current_lsn,
            error_count: health_status.error_count,
            consecutive_failures: health_status.consecutive_failures,
            last_check: format!(
                "{}s ago",
                chrono::Utc::now().timestamp() - health_status.last_check
            ),
        };

        // Get Raft status
        let raft_state = server.raft_node.get_state().await;
        let raft_info = RaftInfo {
            node_id: raft_state.node_id,
            is_leader: server.raft_node.is_leader().await,
            current_term: raft_state.current_term,
            leader_id: server.raft_node.get_leader_id().await,
            commit_index: raft_state.commit_index,
            last_applied: raft_state.last_applied,
            peers: raft_state.peers,
        };

        let response = StatusResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            health: health_info,
            raft: raft_info,
        };

        (StatusCode::OK, Json(response)).into_response()
    }

    async fn cluster_handler(
        State(server): State<Arc<ApiServer>>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        // Check authentication
        if let Err(e) = server.authenticate(&headers).await {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }

        // For now, return single node info
        // In a real cluster, this would query all peers
        let health_status = server.health_checker.get_status().await;
        let raft_state = server.raft_node.get_state().await;

        let node_info = NodeInfo {
            node_id: raft_state.node_id.clone(),
            is_leader: server.raft_node.is_leader().await,
            is_healthy: health_status.is_healthy,
            health_score: health_status.score,
            is_primary: health_status.is_primary,
            replication_lag_seconds: health_status
                .replication_lag
                .map(|lag| lag as f64 / 1_000_000.0),
            last_seen: format!(
                "{}s ago",
                chrono::Utc::now().timestamp() - health_status.last_check
            ),
        };

        let response = ClusterResponse {
            nodes: vec![node_info],
            leader: server.raft_node.get_leader_id().await,
            healthy_nodes: if health_status.is_healthy { 1 } else { 0 },
            total_nodes: 1,
        };

        (StatusCode::OK, Json(response)).into_response()
    }

    async fn metrics_handler(
        State(server): State<Arc<ApiServer>>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        // Check authentication
        if let Err(e) = server.authenticate(&headers).await {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }

        // Return basic metrics as JSON
        let health_status = server.health_checker.get_status().await;
        let raft_state = server.raft_node.get_state().await;

        let metrics = serde_json::json!({
            "blcpg_health_status": if health_status.is_healthy { 1 } else { 0 },
            "blcpg_health_score": health_status.score,
            "blcpg_replication_lag_seconds": health_status.replication_lag.map(|lag| lag as f64 / 1_000_000.0).unwrap_or(-1.0),
            "blcpg_is_primary": if health_status.is_primary { 1 } else { 0 },
            "blcpg_error_count": health_status.error_count,
            "blcpg_consecutive_failures": health_status.consecutive_failures,
            "blcpg_raft_is_leader": if server.raft_node.is_leader().await { 1 } else { 0 },
            "blcpg_raft_current_term": raft_state.current_term,
            "blcpg_raft_commit_index": raft_state.commit_index,
            "blcpg_raft_last_applied": raft_state.last_applied,
        });

        (StatusCode::OK, Json(metrics)).into_response()
    }

    async fn promote_handler(
        State(server): State<Arc<ApiServer>>,
        headers: HeaderMap,
        Json(payload): Json<PromoteRequest>,
    ) -> impl IntoResponse {
        // Check authentication
        if let Err(e) = server.authenticate(&headers).await {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }

        // Check if we're already primary
        if server.health_checker.is_primary().await {
            return (
                StatusCode::OK,
                Json(PromoteResponse {
                    success: true,
                    message: "Already primary".to_string(),
                    new_primary: Some(server.raft_node.get_state().await.node_id),
                }),
            )
                .into_response();
        }

        // Check if we're healthy enough to promote
        let health_status = server.health_checker.get_status().await;
        if !health_status.is_healthy && !payload.force.unwrap_or(false) {
            return (
                StatusCode::BAD_REQUEST,
                Json(PromoteResponse {
                    success: false,
                    message: "Node is not healthy enough for promotion".to_string(),
                    new_primary: None,
                }),
            )
                .into_response();
        }

        // Attempt promotion
        match server.health_checker.force_health_check().await {
            Ok(_) => {
                info!("Manual promotion requested");
                (
                    StatusCode::OK,
                    Json(PromoteResponse {
                        success: true,
                        message: "Promotion initiated".to_string(),
                        new_primary: Some(server.raft_node.get_state().await.node_id),
                    }),
                )
                    .into_response()
            }
            Err(e) => {
                error!("Promotion failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(PromoteResponse {
                        success: false,
                        message: format!("Promotion failed: {}", e),
                        new_primary: None,
                    }),
                )
                    .into_response()
            }
        }
    }

    async fn authenticate(&self, headers: &HeaderMap) -> Result<()> {
        let auth_header = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing Authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(anyhow::anyhow!("Invalid Authorization header format"));
        }

        let token = &auth_header[7..]; // Remove "Bearer " prefix

        if token != self.config.auth_token {
            return Err(anyhow::anyhow!("Invalid authentication token"));
        }

        Ok(())
    }
}

impl Clone for ApiServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            health_checker: Arc::clone(&self.health_checker),
            raft_node: Arc::clone(&self.raft_node),
            cluster_manager: Arc::clone(&self.cluster_manager),
        }
    }
}
