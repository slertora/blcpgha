// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::{ApiResponse, ClusterMetrics};
use axum::response::Json;
use chrono::Utc;

pub async fn get_metrics() -> Json<ApiResponse<ClusterMetrics>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let metrics = ClusterMetrics {
        total_connections: 26,
        active_connections: 26,
        queries_per_second: 2035.0,
        transactions_per_second: 567.0,
        replication_lag_avg: 0.35,
        failover_count: 2,
        last_failover: Some(Utc::now() - chrono::Duration::hours(24)),
        uptime: 86400, // 24 hours in seconds
    };

    Json(ApiResponse::success(metrics))
}
