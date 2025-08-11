// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub cluster_name: String,
    pub leader: Option<String>,
    pub nodes: Vec<NodeInfo>,
    pub health_score: f64,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub unhealthy_nodes: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub is_primary: bool,
    pub is_healthy: bool,
    pub health_score: f64,
    pub replication_lag_seconds: Option<f64>,
    pub last_seen: DateTime<Utc>,
    pub status: NodeStatus,
    pub metrics: NodeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub active_connections: u32,
    pub max_connections: u32,
    pub queries_per_second: f64,
    pub replication_lag: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub total_connections: u32,
    pub active_connections: u32,
    pub queries_per_second: f64,
    pub transactions_per_second: f64,
    pub replication_lag_avg: f64,
    pub failover_count: u32,
    pub last_failover: Option<DateTime<Utc>>,
    pub uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipStatus {
    pub virtual_ip: String,
    pub interface: String,
    pub is_active: bool,
    pub current_owner: Option<String>,
    pub last_switch: Option<DateTime<Utc>>,
    pub manager_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_health: HealthLevel,
    pub checks: Vec<HealthCheck>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthLevel,
    pub message: String,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub cluster_status: ClusterStatus,
    pub metrics: ClusterMetrics,
    pub vip_status: VipStatus,
    pub health_status: HealthStatus,
    pub recent_events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub message: String,
    pub severity: EventSeverity,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    NodeOnline,
    NodeOffline,
    Failover,
    Switchover,
    ReplicationLag,
    HealthCheck,
    Maintenance,
    Backup,
    Restore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
} 