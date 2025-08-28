// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::ApiResponse;
use axum::response::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddReplicaRequest {
    pub host: String,
    pub port: u16,
    pub ssh_user: String,
    pub ssh_key_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterExpandRequest {
    pub nodes: Vec<AddReplicaRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRequest {
    pub backup_type: String,
    pub compression: String,
    pub destination_path: String,
    pub retention_days: u32,
}

// Failover de emergencia
pub async fn failover() -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/failover")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Failover ejecutado exitosamente".to_string()),
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

// Agregar réplica
pub async fn add_replica(Json(request): Json<AddReplicaRequest>) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/add-replica")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Réplica agregada exitosamente".to_string()),
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

// Expandir cluster
pub async fn cluster_expand(
    Json(request): Json<ClusterExpandRequest>,
) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/expand")
        .json(&request)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Cluster expandido exitosamente".to_string()),
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

// Crear backup
pub async fn create_backup(Json(request): Json<BackupRequest>) -> Json<ApiResponse<String>> {
    let client = reqwest::Client::new();

    match client
        .post("http://localhost:8080/api/v1/cluster/backup")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Json(ApiResponse {
                    success: true,
                    data: Some("Backup iniciado exitosamente".to_string()),
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
