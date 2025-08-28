// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::{ApiResponse, HealthCheck, HealthLevel, HealthStatus};
use axum::response::Json;
use chrono::Utc;

pub async fn get_health() -> Json<ApiResponse<HealthStatus>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let health_status = HealthStatus {
        overall_health: HealthLevel::Healthy,
        checks: vec![
            HealthCheck {
                name: "PostgreSQL".to_string(),
                status: HealthLevel::Healthy,
                message: "All nodes are running".to_string(),
                last_check: Utc::now(),
            },
            HealthCheck {
                name: "Replication".to_string(),
                status: HealthLevel::Healthy,
                message: "Replication lag is within acceptable limits".to_string(),
                last_check: Utc::now(),
            },
            HealthCheck {
                name: "Network".to_string(),
                status: HealthLevel::Healthy,
                message: "All network connections are stable".to_string(),
                last_check: Utc::now(),
            },
        ],
        last_check: Utc::now(),
    };

    Json(ApiResponse::success(health_status))
}
