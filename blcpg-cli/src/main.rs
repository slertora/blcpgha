// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::{error, info};

mod client;
mod commands;
mod config;

#[derive(Parser)]
#[command(
    name = "blcpg-cli",
    about = "Command-line interface for BLC PostgreSQL HA",
    version,
    long_about = "BLC PostgreSQL HA CLI - Manage and monitor your PostgreSQL high availability cluster"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show cluster status
    Status {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Promote a node to primary
    Promote {
        /// Node ID to promote
        node_id: String,
    },
    /// Demote a node from primary
    Demote {
        /// Node ID to demote
        node_id: String,
    },
    /// Show cluster information
    ClusterInfo {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Show node information
    NodeInfo {
        /// Node ID to show information for
        node_id: String,
    },
    /// List all nodes in the cluster
    Nodes,
    /// Show health information
    Health {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Show metrics
    Metrics,
    /// Show VIP status
    Vip {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Add a new replica to the cluster
    AddReplica {
        /// Target node ID to add as replica
        #[arg(long)]
        target: String,
        /// Source node ID for backup (auto, primary, or specific node)
        #[arg(long, default_value = "auto")]
        source: String,
        /// Wait for replication to be in sync before completing
        #[arg(long, default_value = "true")]
        wait_sync: bool,
        /// Force operation even if target node is not empty
        #[arg(long)]
        force: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Perform a planned switchover (safer than demote)
    Switchover {
        /// Target node ID to switchover to
        #[arg(long)]
        target: String,
        /// Wait for replication to be in sync before switchover
        #[arg(long, default_value = "true")]
        wait_sync: bool,
        /// Force switchover even if conditions are not ideal
        #[arg(long)]
        force: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Perform automatic failover (emergency)
    Failover {
        /// Target node ID to failover to (auto for auto-selection)
        #[arg(long, default_value = "auto")]
        target: String,
        /// Enable notifications during failover
        #[arg(long, default_value = "true")]
        notify: bool,
        /// Force failover even if conditions are not ideal
        #[arg(long)]
        force: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Show detailed replication status
    ReplicationStatus {
        /// Show detailed information for all nodes
        #[arg(long)]
        detailed: bool,
        /// Show only nodes with issues
        #[arg(long)]
        issues_only: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Expand cluster by adding new nodes
    ClusterExpand {
        /// Number of nodes to add
        #[arg(long, default_value = "1")]
        count: u32,
        /// Auto-install PostgreSQL on new nodes
        #[arg(long, default_value = "true")]
        auto_install: bool,
        /// Auto-configure as replicas
        #[arg(long, default_value = "true")]
        auto_configure: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Create intelligent backup
    Backup {
        /// Target node ID for backup (auto for auto-selection)
        #[arg(long, default_value = "auto")]
        target: String,
        /// Backup type (full, incremental, differential)
        #[arg(long, default_value = "full")]
        backup_type: String,
        /// Compression level (0-9)
        #[arg(long, default_value = "6")]
        compression: u8,
        /// Encrypt backup
        #[arg(long)]
        encrypt: bool,
        /// Backup retention days
        #[arg(long, default_value = "30")]
        retention_days: u32,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Restore intelligent backup
    Restore {
        /// Backup ID to restore
        #[arg(long)]
        backup_id: String,
        /// Target node ID for restore (auto for auto-selection)
        #[arg(long, default_value = "auto")]
        target: String,
        /// Restore type (full, point-in-time, schema-only)
        #[arg(long, default_value = "full")]
        restore_type: String,
        /// Point-in-time recovery timestamp (YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        point_in_time: Option<String>,
        /// Verify backup before restore
        #[arg(long, default_value = "true")]
        verify: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Perform maintenance operations
    Maintenance {
        /// Target node ID for maintenance (auto for auto-selection)
        #[arg(long, default_value = "auto")]
        target: String,
        /// Maintenance type (vacuum, analyze, reindex, checkpoint)
        #[arg(long, default_value = "vacuum")]
        maintenance_type: String,
        /// Maintenance level (light, full, aggressive)
        #[arg(long, default_value = "light")]
        level: String,
        /// Run maintenance in background
        #[arg(long)]
        background: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Upgrade PostgreSQL version
    Upgrade {
        /// Target node ID for upgrade (auto for auto-selection)
        #[arg(long, default_value = "auto")]
        target: String,
        /// Target PostgreSQL version
        #[arg(long)]
        version: String,
        /// Upgrade strategy (rolling, all-at-once, blue-green)
        #[arg(long, default_value = "rolling")]
        strategy: String,
        /// Backup before upgrade
        #[arg(long, default_value = "true")]
        backup: bool,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
    /// Remote configuration and node management
    Config {
        /// Target node ID for configuration
        #[arg(long)]
        target: String,
        /// Configuration action (get, set, validate, backup, restore)
        #[arg(long, default_value = "get")]
        action: String,
        /// Configuration key (for set action)
        #[arg(long)]
        key: Option<String>,
        /// Configuration value (for set action)
        #[arg(long)]
        value: Option<String>,
        /// Configuration file path
        #[arg(long, default_value = "config.toml")]
        file: String,
        /// Demo mode for testing without real nodes
        #[arg(long)]
        demo: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    info!("BLC PostgreSQL HA CLI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::load_config(&cli.config)?;

    // Create API client
    let client = client::ApiClient::new(config.api)?;

    // Execute command
    match cli.command {
        Commands::Status { detailed } => {
            commands::status::execute(&client, detailed).await?;
        }
        Commands::Promote { node_id } => {
            commands::promote::execute(&client, &node_id).await?;
        }
        Commands::Demote { node_id } => {
            commands::demote::execute(&client, &node_id).await?;
        }
        Commands::ClusterInfo { detailed } => {
            commands::cluster_info::execute(&client, detailed).await?;
        }
        Commands::NodeInfo { node_id } => {
            commands::node_info::execute(&client, &node_id).await?;
        }
        Commands::Nodes => {
            commands::nodes::execute(&client).await?;
        }
        Commands::Health { detailed } => {
            commands::health::execute(&client, detailed).await?;
        }
        Commands::Metrics => {
            commands::metrics::execute(&client).await?;
        }
        Commands::Vip { detailed } => {
            commands::vip::execute(&client, detailed).await?;
        }
        Commands::AddReplica { target, source, wait_sync, force, demo } => {
            commands::add_replica::execute(&client, &target, &source, wait_sync, force, demo).await?;
        }
        Commands::Switchover { target, wait_sync, force, demo } => {
            commands::switchover::execute(&client, &target, wait_sync, force, demo).await?;
        }
        Commands::Failover { target, notify, force, demo } => {
            commands::failover::execute(&client, &target, notify, force, demo).await?;
        }
        Commands::ReplicationStatus { detailed, issues_only, demo } => {
            commands::replication_status::execute(&client, detailed, issues_only, demo).await?;
        }
        Commands::ClusterExpand { count, auto_install, auto_configure, demo } => {
            commands::cluster_expand::execute(&client, count, auto_install, auto_configure, demo).await?;
        }
        Commands::Backup { target, backup_type, compression, encrypt, retention_days, demo } => {
            commands::backup::execute(&client, &target, &backup_type, compression, encrypt, retention_days, demo).await?;
        }
        Commands::Restore { backup_id, target, restore_type, point_in_time, verify, demo } => {
            commands::restore::execute(&client, &backup_id, &target, &restore_type, point_in_time, verify, demo).await?;
        }
        Commands::Maintenance { target, maintenance_type, level, background, demo } => {
            commands::maintenance::execute(&client, &target, &maintenance_type, &level, background, demo).await?;
        }
        Commands::Upgrade { target, version, strategy, backup, demo } => {
            commands::upgrade::execute(&client, &target, &version, &strategy, backup, demo).await?;
        }
        Commands::Config { target, action, key, value, file, demo } => {
            commands::config::execute(&client, &target, &action, key, value, &file, demo).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::MockApiClient;
    use anyhow::Result;

    #[tokio::test]
    async fn test_add_replica_demo_mode() -> Result<()> {
        let client = MockApiClient;
        let result = commands::add_replica::execute(&client, "node-4", "auto", true, false, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_switchover_demo_mode() -> Result<()> {
        let client = MockApiClient;
        let result = commands::switchover::execute(&client, "node-2", true, false, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_failover_demo_mode() -> Result<()> {
        let client = MockApiClient;
        let result = commands::failover::execute(&client, "auto", true, false, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_replication_status_demo_mode() -> Result<()> {
        let client = MockApiClient;
        let result = commands::replication_status::execute(&client, true, false, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_cluster_expand_demo_mode() -> Result<()> {
        let client = MockApiClient;
        let result = commands::cluster_expand::execute(&client, 2, true, true, true).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_generate_new_node_ids() {
        let existing_nodes = vec![
            crate::client::NodeInfo {
                node_id: "node-1".to_string(),
                is_leader: true,
                is_primary: true,
                is_healthy: true,
                health_score: 95.0,
                replication_lag_seconds: None,
                last_seen: "2024-01-01T00:00:00Z".to_string(),
            },
            crate::client::NodeInfo {
                node_id: "node-2".to_string(),
                is_leader: false,
                is_primary: false,
                is_healthy: true,
                health_score: 90.0,
                replication_lag_seconds: Some(0.1),
                last_seen: "2024-01-01T00:00:00Z".to_string(),
            },
        ];

        let new_ids = commands::cluster_expand::generate_new_node_ids(&existing_nodes, 2).unwrap();
        assert_eq!(new_ids.len(), 2);
        assert_eq!(new_ids[0], "node-3");
        assert_eq!(new_ids[1], "node-4");
    }
} 