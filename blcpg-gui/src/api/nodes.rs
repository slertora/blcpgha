// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{
    extract::Path,
    response::Json,
};
use crate::models::{ApiResponse, NodeInfo};
use reqwest::Client;
use std::time::Duration;
use chrono::Utc;

pub async fn list_nodes() -> Json<ApiResponse<Vec<NodeInfo>>> {
    let client = Client::new();
    
    // Intentar obtener datos reales del agente blcpg-ha
    match client
        .get("http://localhost:8080/api/v1/nodes")
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<Vec<NodeInfo>>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        // Si falla el parsing, devolver error
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            // Si no se puede conectar, devolver error
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
}

pub async fn get_node(Path(node_id): Path<String>) -> Json<ApiResponse<NodeInfo>> {
    let client = Client::new();
    
    // Intentar obtener datos reales del agente blcpg-ha
    match client
        .get(&format!("http://localhost:8080/api/v1/nodes/{}", node_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<NodeInfo>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
}

pub async fn promote_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    let client = Client::new();
    
    // Intentar promover el nodo a través del agente blcpg-ha
    match client
        .post(&format!("http://localhost:8080/api/v1/nodes/{}/promote", node_id))
        .timeout(Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<String>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
}

pub async fn demote_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    let client = Client::new();
    
    // Intentar demover el nodo a través del agente blcpg-ha
    match client
        .post(&format!("http://localhost:8080/api/v1/nodes/{}/demote", node_id))
        .timeout(Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<String>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
}

pub async fn disable_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    let client = Client::new();
    
    // Intentar deshabilitar el nodo a través del agente blcpg-ha
    match client
        .post(&format!("http://localhost:8080/api/v1/nodes/{}/disable", node_id))
        .timeout(Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<String>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
}

pub async fn enable_node(Path(node_id): Path<String>) -> Json<ApiResponse<String>> {
    let client = Client::new();
    
    // Intentar habilitar el nodo a través del agente blcpg-ha
    match client
        .post(&format!("http://localhost:8080/api/v1/nodes/{}/enable", node_id))
        .timeout(Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ApiResponse<String>>().await {
                    Ok(data) => Json(data),
                    Err(_) => {
                        Json(ApiResponse {
                            success: false,
                            data: None,
                            error: Some("Error parsing response from blcpg-ha".to_string()),
                            timestamp: Utc::now(),
                        })
                    }
                }
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("blcpg-ha returned status: {}", response.status())),
                    timestamp: Utc::now(),
                })
            }
        }
        Err(_) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Cannot connect to blcpg-ha agent".to_string()),
                timestamp: Utc::now(),
            })
        }
    }
} 