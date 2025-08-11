// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::models::{NodeInfo, ApiResponse, NodeStatus, NodeMetrics};
use chrono::Utc;

pub async fn list_nodes() -> Json<ApiResponse<Vec<NodeInfo>>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let nodes = vec![
        NodeInfo {
            node_id: "node-1".to_string(),
            host: "192.168.1.10".to_string(),
            port: 5432,
            is_primary: true,
            is_healthy: true,
            health_score: 95.0,
            replication_lag_seconds: None,
            last_seen: Utc::now(),
            status: NodeStatus::Online,
            metrics: NodeMetrics {
                cpu_usage: 45.0,
                memory_usage: 67.0,
                disk_usage: 23.0,
                active_connections: 12,
                max_connections: 100,
                queries_per_second: 1234.0,
                replication_lag: None,
            },
        },
        NodeInfo {
            node_id: "node-2".to_string(),
            host: "192.168.1.11".to_string(),
            port: 5432,
            is_primary: false,
            is_healthy: true,
            health_score: 90.0,
            replication_lag_seconds: Some(0.2),
            last_seen: Utc::now(),
            status: NodeStatus::Online,
            metrics: NodeMetrics {
                cpu_usage: 38.0,
                memory_usage: 72.0,
                disk_usage: 25.0,
                active_connections: 8,
                max_connections: 100,
                queries_per_second: 567.0,
                replication_lag: Some(0.2),
            },
        },
        NodeInfo {
            node_id: "node-3".to_string(),
            host: "192.168.1.12".to_string(),
            port: 5432,
            is_primary: false,
            is_healthy: true,
            health_score: 88.0,
            replication_lag_seconds: Some(0.5),
            last_seen: Utc::now(),
            status: NodeStatus::Online,
            metrics: NodeMetrics {
                cpu_usage: 42.0,
                memory_usage: 65.0,
                disk_usage: 28.0,
                active_connections: 6,
                max_connections: 100,
                queries_per_second: 234.0,
                replication_lag: Some(0.5),
            },
        },
    ];

    Json(ApiResponse::success(nodes))
}

pub async fn get_node(Path(node_id): Path<String>) -> Json<ApiResponse<NodeInfo>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let node = NodeInfo {
        node_id: node_id.clone(),
        host: "192.168.1.10".to_string(),
        port: 5432,
        is_primary: node_id == "node-1",
        is_healthy: true,
        health_score: 95.0,
        replication_lag_seconds: if node_id == "node-1" { None } else { Some(0.2) },
        last_seen: Utc::now(),
        status: NodeStatus::Online,
        metrics: NodeMetrics {
            cpu_usage: 45.0,
            memory_usage: 67.0,
            disk_usage: 23.0,
            active_connections: 12,
            max_connections: 100,
            queries_per_second: 1234.0,
            replication_lag: if node_id == "node-1" { None } else { Some(0.2) },
        },
    };

    Json(ApiResponse::success(node))
}

pub async fn promote_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    // In a real implementation, this would call the blcpg-ha API
    Json(ApiResponse::success(format!("Node {} promoted successfully", node_id)))
}

pub async fn demote_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    // In a real implementation, this would call the blcpg-ha API
    Json(ApiResponse::success(format!("Node {} demoted successfully", node_id)))
}

pub async fn disable_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    // In a real implementation, this would call the blcpg-ha API
    Json(ApiResponse::success(format!("Node {} disabled successfully", node_id)))
}

pub async fn enable_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    // In a real implementation, this would call the blcpg-ha API
    Json(ApiResponse::success(format!("Node {} enabled successfully", node_id)))
} 