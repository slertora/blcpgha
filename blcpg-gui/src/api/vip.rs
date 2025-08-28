// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::{ApiResponse, VipStatus};
use axum::response::Json;
use chrono::Utc;

pub async fn get_status() -> Json<ApiResponse<VipStatus>> {
    // In a real implementation, this would fetch data from the blcpg-ha API
    let vip_status = VipStatus {
        virtual_ip: "192.168.1.100".to_string(),
        interface: "eth0".to_string(),
        is_active: true,
        current_owner: Some("node-1".to_string()),
        last_switch: Some(Utc::now() - chrono::Duration::hours(24)),
        manager_type: "keepalived".to_string(),
    };

    Json(ApiResponse::success(vip_status))
}
