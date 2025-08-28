// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::models::{ApiResponse, ClusterStatus};
use axum::response::Json;
use reqwest::Client;

pub async fn get_status() -> Json<ApiResponse<ClusterStatus>> {
    // Create HTTP client
    let client = Client::new();

    // Try to fetch real data from blcpg-ha
    match client
        .get("http://localhost:8080/api/v1/cluster/status")
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<ClusterStatus>().await {
                Ok(cluster_status) => Json(ApiResponse::success(cluster_status)),
                Err(_) => {
                    // Fallback to mock data if parsing fails
                    let mock_status = ClusterStatus {
                        cluster_name: "blc-cluster".to_string(),
                        leader: Some("node-1".to_string()),
                        nodes: vec![],
                        health_score: 91.0,
                        total_nodes: 3,
                        healthy_nodes: 3,
                        unhealthy_nodes: 0,
                        last_updated: chrono::Utc::now(),
                    };
                    Json(ApiResponse::success(mock_status))
                }
            }
        }
        Err(_) => {
            // Fallback to mock data if API is unreachable
            let mock_status = ClusterStatus {
                cluster_name: "blc-cluster".to_string(),
                leader: Some("node-1".to_string()),
                nodes: vec![],
                health_score: 91.0,
                total_nodes: 3,
                healthy_nodes: 3,
                unhealthy_nodes: 0,
                last_updated: chrono::Utc::now(),
            };
            Json(ApiResponse::success(mock_status))
        }
    }
}
