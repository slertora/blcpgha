// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use crate::models::{ClusterStatus, ApiResponse};

pub async fn get_status() -> Json<ApiResponse<ClusterStatus>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let cluster_status = ClusterStatus {
        cluster_name: "blc-cluster".to_string(),
        leader: Some("node-1".to_string()),
        nodes: vec![], // Would be populated from API
        health_score: 91.0,
        total_nodes: 3,
        healthy_nodes: 3,
        unhealthy_nodes: 0,
        last_updated: chrono::Utc::now(),
    };

    Json(ApiResponse::success(cluster_status))
} 