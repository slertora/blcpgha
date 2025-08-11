// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::config::ApiConfig;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub health: HealthInfo,
    pub raft: RaftInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    pub is_healthy: bool,
    pub score: f64,
    pub is_primary: bool,
    pub replication_lag_seconds: Option<f64>,
    pub server_version: Option<String>,
    pub current_lsn: Option<String>,
    pub error_count: u32,
    pub consecutive_failures: u32,
    pub last_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftInfo {
    pub node_id: String,
    pub is_leader: bool,
    pub current_term: u64,
    pub leader_id: Option<String>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub peers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatusResponse {
    pub nodes: Vec<NodeInfo>,
    pub leader: Option<String>,
    pub healthy_nodes: u32,
    pub total_nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub is_leader: bool,
    pub is_healthy: bool,
    pub health_score: f64,
    pub is_primary: bool,
    pub replication_lag_seconds: Option<f64>,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub is_healthy: bool,
    pub health_score: f64,
    pub error_count: u32,
    pub consecutive_failures: u32,
    pub last_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub health_checks_total: u64,
    pub health_checks_failed: u64,
    pub failovers_total: u64,
    pub vip_updates_total: u64,
    pub raft_leader_changes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipStatusResponse {
    pub enabled: bool,
    pub r#type: String,
    pub current_primary: Option<String>,
    pub is_healthy: bool,
    pub last_update: Option<String>,
}

#[async_trait::async_trait]
pub trait ApiClientTrait {
    async fn get_cluster_status(&self) -> Result<ClusterStatusResponse>;
    async fn get_status(&self) -> Result<StatusResponse>;
    async fn get_health(&self) -> Result<HealthResponse>;
    async fn get_metrics(&self) -> Result<MetricsResponse>;
    async fn get_vip_status(&self) -> Result<VipStatusResponse>;
    async fn promote_node(&self, node_id: &str) -> Result<()>;
    async fn demote_node(&self, node_id: &str) -> Result<()>;
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: String,
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Result<Self> {
        let timeout = humantime::Duration::from_str(&config.timeout)
            .map_err(|e| anyhow::anyhow!("Invalid timeout format: {}", e))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout.as_secs()))
            .build()?;

        let base_url = format!("http://{}:{}", config.host, config.port);

        Ok(Self {
            client,
            base_url,
            auth_token: config.auth_token,
        })
    }

    async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed: {}",
                response.status()
            ));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    async fn post<T>(&self, endpoint: &str, body: &impl Serialize) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed: {}",
                response.status()
            ));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }
}

#[async_trait::async_trait]
impl ApiClientTrait for ApiClient {
    async fn get_cluster_status(&self) -> Result<ClusterStatusResponse> {
        self.get("/api/v1/cluster/status").await
    }

    async fn get_status(&self) -> Result<StatusResponse> {
        self.get("/api/v1/status").await
    }

    async fn get_health(&self) -> Result<HealthResponse> {
        self.get("/api/v1/health").await
    }

    async fn get_metrics(&self) -> Result<MetricsResponse> {
        self.get("/api/v1/metrics").await
    }

    async fn get_vip_status(&self) -> Result<VipStatusResponse> {
        self.get("/api/v1/vip/status").await
    }

    async fn promote_node(&self, node_id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct PromoteRequest {
            node_id: String,
        }

        let request = PromoteRequest {
            node_id: node_id.to_string(),
        };

        self.post("/api/v1/cluster/promote", &request).await?;
        Ok(())
    }

    async fn demote_node(&self, node_id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct DemoteRequest {
            node_id: String,
        }

        let request = DemoteRequest {
            node_id: node_id.to_string(),
        };

        self.post("/api/v1/cluster/demote", &request).await?;
        Ok(())
    }
} 