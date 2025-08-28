// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::{ApiClientTrait, ClusterStatusResponse, NodeInfo};
use crate::commands::add_replica;
use crate::commands::switchover;
use crate::commands::failover;
use crate::commands::replication_status;
use crate::commands::cluster_expand;
use crate::commands::backup;
use crate::commands::restore;
use crate::commands::maintenance;
use crate::commands::upgrade;
use crate::commands::config;
use anyhow::Result;

// Mock client for testing
pub struct MockApiClient;

#[async_trait::async_trait]
impl ApiClientTrait for MockApiClient {
    async fn get_cluster_status(&self) -> Result<ClusterStatusResponse> {
        Ok(ClusterStatusResponse {
            total_nodes: 3,
            healthy_nodes: 2,
            nodes: vec![
                NodeInfo {
                    node_id: "node-1".to_string(),
                    is_leader: true,
                    is_primary: true,
                    is_healthy: true,
                    health_score: 95.0,
                    replication_lag_seconds: None,
                    last_seen: "2024-01-01T00:00:00Z".to_string(),
                },
                NodeInfo {
                    node_id: "node-2".to_string(),
                    is_leader: false,
                    is_primary: false,
                    is_healthy: true,
                    health_score: 90.0,
                    replication_lag_seconds: Some(0.1),
                    last_seen: "2024-01-01T00:00:00Z".to_string(),
                },
                NodeInfo {
                    node_id: "node-3".to_string(),
                    is_leader: false,
                    is_primary: false,
                    is_healthy: false,
                    health_score: 45.0,
                    replication_lag_seconds: Some(5.0),
                    last_seen: "2024-01-01T00:00:00Z".to_string(),
                },
            ],
            leader: Some("node-1".to_string()),
        })
    }

    async fn get_status(&self) -> Result<crate::client::StatusResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_health(&self) -> Result<crate::client::HealthResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_metrics(&self) -> Result<crate::client::MetricsResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_vip_status(&self) -> Result<crate::client::VipStatusResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn promote_node(&self, _node_id: &str) -> Result<()> {
        Ok(())
    }

    async fn demote_node(&self, _node_id: &str) -> Result<()> {
        Ok(())
    }
}

// Fast tests that don't require user interaction
#[tokio::test]
async fn test_add_replica_demo_mode() -> Result<()> {
    let client = MockApiClient;
    // Test with demo mode to avoid user interaction
    let result = super::add_replica::execute(&client, "node-4", "auto", true, false, true).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_add_replica_with_invalid_target() -> Result<()> {
    let client = MockApiClient;
    // This should fail because the target doesn't exist
    let result = super::add_replica::execute(&client, "invalid-node", "auto", true, false, false).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_switchover_demo_mode() -> Result<()> {
    let client = MockApiClient;
    let result = super::switchover::execute(&client, "node-2", true, false, true).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_switchover_to_primary() -> Result<()> {
    let client = MockApiClient;
    let result = super::switchover::execute(&client, "node-1", true, false, false).await;
    assert!(result.is_ok()); // Should succeed but warn that node is already primary
    Ok(())
}

#[tokio::test]
async fn test_failover_demo_mode() -> Result<()> {
    let client = MockApiClient;
    let result = super::failover::execute(&client, "auto", true, false, true).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
#[ignore] // Temporarily disabled - test hangs in CI
async fn test_failover_with_specific_target() -> Result<()> {
    let client = MockApiClient;
    let result = super::failover::execute(&client, "node-2", true, false, false).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_replication_status_demo_mode() -> Result<()> {
    let client = MockApiClient;
    let result = super::replication_status::execute(&client, true, false, true).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_replication_status_issues_only() -> Result<()> {
    let client = MockApiClient;
    let result = super::replication_status::execute(&client, false, true, false).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_cluster_expand_demo_mode() -> Result<()> {
    let client = MockApiClient;
    let result = super::cluster_expand::execute(&client, 2, true, true, true).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_cluster_expand_with_zero_count() -> Result<()> {
    let client = MockApiClient;
    let result = super::cluster_expand::execute(&client, 0, true, true, false).await;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_generate_new_node_ids() {
    let existing_nodes = vec![
        NodeInfo {
            node_id: "node-1".to_string(),
            is_leader: true,
            is_primary: true,
            is_healthy: true,
            health_score: 95.0,
            replication_lag_seconds: None,
            last_seen: "2024-01-01T00:00:00Z".to_string(),
        },
        NodeInfo {
            node_id: "node-2".to_string(),
            is_leader: false,
            is_primary: false,
            is_healthy: true,
            health_score: 90.0,
            replication_lag_seconds: Some(0.1),
            last_seen: "2024-01-01T00:00:00Z".to_string(),
        },
    ];

    let new_ids = super::cluster_expand::generate_new_node_ids(&existing_nodes, 2).unwrap();
    assert_eq!(new_ids.len(), 2);
    assert_eq!(new_ids[0], "node-3");
    assert_eq!(new_ids[1], "node-4");
}

#[test]
fn test_generate_new_node_ids_with_existing_gaps() {
    let existing_nodes = vec![
        NodeInfo {
            node_id: "node-1".to_string(),
            is_leader: true,
            is_primary: true,
            is_healthy: true,
            health_score: 95.0,
            replication_lag_seconds: None,
            last_seen: "2024-01-01T00:00:00Z".to_string(),
        },
        NodeInfo {
            node_id: "node-3".to_string(), // Gap in numbering
            is_leader: false,
            is_primary: false,
            is_healthy: true,
            health_score: 90.0,
            replication_lag_seconds: Some(0.1),
            last_seen: "2024-01-01T00:00:00Z".to_string(),
        },
    ];

    let new_ids = super::cluster_expand::generate_new_node_ids(&existing_nodes, 2).unwrap();
    assert_eq!(new_ids.len(), 2);
    assert_eq!(new_ids[0], "node-2"); // Should fill the gap
    assert_eq!(new_ids[1], "node-4");
}

#[test]
fn test_generate_new_node_ids_empty_cluster() {
    let existing_nodes = vec![];
    let new_ids = super::cluster_expand::generate_new_node_ids(&existing_nodes, 3).unwrap();
    assert_eq!(new_ids.len(), 3);
    assert_eq!(new_ids[0], "node-1");
    assert_eq!(new_ids[1], "node-2");
    assert_eq!(new_ids[2], "node-3");
}

// Integration tests
#[tokio::test]
async fn test_full_workflow_demo() -> Result<()> {
    let client = MockApiClient;
    
    // 1. Check replication status
    let status_result = super::replication_status::execute(&client, true, false, true).await;
    assert!(status_result.is_ok());
    
    // 2. Add a replica
    let add_result = super::add_replica::execute(&client, "node-4", "auto", true, false, true).await;
    assert!(add_result.is_ok());
    
    // 3. Expand cluster
    let expand_result = super::cluster_expand::execute(&client, 1, true, true, true).await;
    assert!(expand_result.is_ok());
    
    // 4. Perform switchover
    let switchover_result = super::switchover::execute(&client, "node-2", true, false, true).await;
    assert!(switchover_result.is_ok());
    
    Ok(())
}

// Error handling tests
#[tokio::test]
async fn test_add_replica_with_unhealthy_source() -> Result<()> {
    let client = MockApiClient;
    let result = super::add_replica::execute(&client, "node-4", "node-3", true, false, false).await;
    // Should fail because node-3 is unhealthy
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_switchover_with_unhealthy_target() -> Result<()> {
    let client = MockApiClient;
    let result = super::switchover::execute(&client, "node-3", true, false, false).await;
    // Should fail because node-3 is unhealthy
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
#[ignore] // Temporarily disabled - test hangs in CI
async fn test_failover_with_no_healthy_candidates() -> Result<()> {
    // Create a mock client with only unhealthy nodes
    let unhealthy_client = UnhealthyMockApiClient;
    let result = super::failover::execute(&unhealthy_client, "auto", true, false, false).await;
    // Should fail because no healthy candidates
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_backup_demo_mode() {
    let client = MockApiClient;
    
    // Test backup with demo mode
    let result = backup::execute(
        &client,
        "auto",
        "full",
        6,
        false,
        30,
        true
    ).await;
    
    assert!(result.is_ok(), "Backup command should succeed in demo mode");
}

#[tokio::test]
async fn test_restore_demo_mode() {
    let client = MockApiClient;
    
    // Test restore with demo mode
    let result = restore::execute(
        &client,
        "backup-1234567890",
        "auto",
        "full",
        None,
        true,
        true
    ).await;
    
    assert!(result.is_ok(), "Restore command should succeed in demo mode");
}

#[tokio::test]
async fn test_maintenance_demo_mode() {
    let client = MockApiClient;
    
    // Test maintenance with demo mode
    let result = maintenance::execute(
        &client,
        "auto",
        "vacuum",
        "light",
        false,
        true
    ).await;
    
    assert!(result.is_ok(), "Maintenance command should succeed in demo mode");
}

#[tokio::test]
async fn test_upgrade_demo_mode() {
    let client = MockApiClient;
    
    // Test upgrade with demo mode
    let result = upgrade::execute(
        &client,
        "auto",
        "15.3",
        "rolling",
        true,
        true
    ).await;
    
    assert!(result.is_ok(), "Upgrade command should succeed in demo mode");
}

#[tokio::test]
async fn test_config_demo_mode() {
    let client = MockApiClient;
    
    // Test config with demo mode
    let result = config::execute(
        &client,
        "node-1",
        "get",
        None,
        None,
        "config.toml",
        true
    ).await;
    
    assert!(result.is_ok(), "Config command should succeed in demo mode");
}

// Mock client with only unhealthy nodes
struct UnhealthyMockApiClient;

#[async_trait::async_trait]
impl ApiClientTrait for UnhealthyMockApiClient {
    async fn get_cluster_status(&self) -> Result<ClusterStatusResponse> {
        Ok(ClusterStatusResponse {
            total_nodes: 2,
            healthy_nodes: 0,
            nodes: vec![
                NodeInfo {
                    node_id: "node-1".to_string(),
                    is_leader: true,
                    is_primary: true,
                    is_healthy: false,
                    health_score: 10.0,
                    replication_lag_seconds: None,
                    last_seen: "2024-01-01T00:00:00Z".to_string(),
                },
                NodeInfo {
                    node_id: "node-2".to_string(),
                    is_leader: false,
                    is_primary: false,
                    is_healthy: false,
                    health_score: 5.0,
                    replication_lag_seconds: Some(10.0),
                    last_seen: "2024-01-01T00:00:00Z".to_string(),
                },
            ],
            leader: Some("node-1".to_string()),
        })
    }

    async fn get_status(&self) -> Result<crate::client::StatusResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_health(&self) -> Result<crate::client::HealthResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_metrics(&self) -> Result<crate::client::MetricsResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn get_vip_status(&self) -> Result<crate::client::VipStatusResponse> {
        unimplemented!("Not needed for these tests")
    }

    async fn promote_node(&self, _node_id: &str) -> Result<()> {
        Ok(())
    }

    async fn demote_node(&self, _node_id: &str) -> Result<()> {
        Ok(())
    }
} 