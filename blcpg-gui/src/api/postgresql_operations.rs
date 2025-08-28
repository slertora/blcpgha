// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::ApiResponse;
use axum::response::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub backup_file: String,
    pub restore_type: String,
    pub target_node: String,
    pub point_in_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub maintenance_type: String,
    pub maintenance_level: String,
    pub target_nodes: String,
    pub schedule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationOptions {
    pub connections: bool,
    pub replication: bool,
    pub performance: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeRequest {
    pub target_version: String,
    pub upgrade_strategy: String,
    pub validation: ValidationOptions,
}

// Restore desde backup
pub async fn restore(Json(request): Json<RestoreRequest>) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/restore")
        .json(&request)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Restore iniciado exitosamente".to_string()),
                    error: None,
                    timestamp: Utc::now(),
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Cannot connect to blcpg-ha agent".to_string()),
            timestamp: Utc::now(),
        }),
    }
}

// Ejecutar mantenimiento
pub async fn maintenance(Json(request): Json<MaintenanceRequest>) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/maintenance")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Mantenimiento iniciado exitosamente".to_string()),
                    error: None,
                    timestamp: Utc::now(),
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Cannot connect to blcpg-ha agent".to_string()),
            timestamp: Utc::now(),
        }),
    }
}

// Actualizar PostgreSQL
pub async fn upgrade(Json(request): Json<UpgradeRequest>) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/upgrade")
        .json(&request)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Actualización iniciada exitosamente".to_string()),
                    error: None,
                    timestamp: Utc::now(),
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Cannot connect to blcpg-ha agent".to_string()),
            timestamp: Utc::now(),
        }),
    }
}
