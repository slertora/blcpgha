// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ReplicationStatusRow {
    #[tabled(rename = "Node ID")]
    node_id: String,
    #[tabled(rename = "Role")]
    role: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Sync State")]
    sync_state: String,
    #[tabled(rename = "Replay Lag")]
    replay_lag: String,
    #[tabled(rename = "Write Lag")]
    write_lag: String,
    #[tabled(rename = "Health Score")]
    health_score: String,
    #[tabled(rename = "WAL Receive LSN")]
    wal_receive_lsn: String,
    #[tabled(rename = "WAL Replay LSN")]
    wal_replay_lsn: String,
}

pub async fn execute(
    client: &impl ApiClientTrait,
    detailed: bool,
    issues_only: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Replication Status".bold().blue());
    println!("{}", "=".repeat(18));

    if demo {
        println!(
            "{}",
            "DEMO MODE ENABLED - Simulating operations".yellow().bold()
        );
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Cluster Overview:");
    println!("  Total Nodes: {}", cluster_status.total_nodes);
    println!("  Healthy Nodes: {}", cluster_status.healthy_nodes);
    println!(
        "  Primary Node: {}",
        get_primary_node_id(&cluster_status.nodes)
    );
    println!();

    // Collect replication status for all nodes
    let mut replication_rows = Vec::new();
    let mut issues_found = Vec::new();

    for node in &cluster_status.nodes {
        let (sync_state, replay_lag, write_lag, wal_receive_lsn, wal_replay_lsn) = if demo {
            get_demo_replication_info(node)
        } else {
            get_replication_info(node).await?
        };

        let status = if node.is_healthy {
            "Healthy".green()
        } else {
            "Unhealthy".red()
        };
        let role = if node.is_primary {
            "Primary".bold().cyan()
        } else {
            "Replica".yellow()
        };

        let row = ReplicationStatusRow {
            node_id: node.node_id.clone(),
            role: role.to_string(),
            status: status.to_string(),
            sync_state: sync_state.clone(),
            replay_lag: replay_lag.clone(),
            write_lag: write_lag.clone(),
            health_score: format!("{:.2}%", node.health_score),
            wal_receive_lsn: wal_receive_lsn.clone(),
            wal_replay_lsn: wal_replay_lsn.clone(),
        };

        // Check for issues
        let has_issues = !node.is_healthy
            || sync_state.contains("disconnected")
            || replay_lag.contains(">5s")
            || write_lag.contains(">5s");

        if has_issues {
            issues_found.push(node.node_id.clone());
        }

        if !issues_only || has_issues {
            replication_rows.push(row);
        }
    }

    // Display replication status table
    if !replication_rows.is_empty() {
        let table = Table::new(replication_rows);
        println!("{}", table);
    } else {
        println!("{}", "✅ No issues found!".green().bold());
    }

    // Show issues summary
    if !issues_found.is_empty() {
        println!();
        println!("{}", "Issues Detected:".bold().red());
        for issue_node in &issues_found {
            println!("  • Node {} has replication issues", issue_node.bold());
        }
    }

    // Show recommendations
    if detailed {
        println!();
        println!("{}", "Recommendations:".bold().cyan());
        show_recommendations(&cluster_status.nodes, demo).await?;
    }

    Ok(())
}

fn get_primary_node_id(nodes: &[crate::client::NodeInfo]) -> String {
    nodes
        .iter()
        .find(|node| node.is_primary)
        .map(|node| node.node_id.clone())
        .unwrap_or_else(|| "None".red().to_string())
}

fn get_demo_replication_info(
    node: &crate::client::NodeInfo,
) -> (String, String, String, String, String) {
    if node.is_primary {
        (
            "N/A".yellow().to_string(),
            "N/A".yellow().to_string(),
            "N/A".yellow().to_string(),
            "0/12345678".to_string(),
            "0/12345678".to_string(),
        )
    } else {
        let sync_state = if node.node_id == "node-2" {
            "sync".green()
        } else {
            "async".yellow()
        };
        let replay_lag = if node.node_id == "node-2" {
            "0.05s".green()
        } else {
            "0.25s".yellow()
        };
        let write_lag = if node.node_id == "node-2" {
            "0.02s".green()
        } else {
            "0.15s".yellow()
        };

        (
            sync_state.to_string(),
            replay_lag.to_string(),
            write_lag.to_string(),
            "0/12345678".to_string(),
            "0/12345670".to_string(),
        )
    }
}

async fn get_replication_info(
    node: &crate::client::NodeInfo,
) -> Result<(String, String, String, String, String)> {
    // In production, this would query PostgreSQL for actual replication information
    // For now, we'll simulate based on the node's health and replication lag

    if node.is_primary {
        return Ok((
            "N/A".yellow().to_string(),
            "N/A".yellow().to_string(),
            "N/A".yellow().to_string(),
            "0/12345678".to_string(),
            "0/12345678".to_string(),
        ));
    }

    let sync_state = if node.health_score > 80.0 {
        "sync".green()
    } else {
        "async".yellow()
    };
    let replay_lag = get_replication_lag_text(node.replication_lag_seconds);
    let write_lag = get_replication_lag_text(node.replication_lag_seconds.map(|lag| lag * 0.8));

    // Simulate WAL LSNs
    let wal_receive_lsn = "0/12345678".to_string();
    let wal_replay_lsn = if node.replication_lag_seconds.unwrap_or(0.0) < 1.0 {
        "0/12345670".to_string()
    } else {
        "0/12345600".to_string()
    };

    Ok((
        sync_state.to_string(),
        replay_lag,
        write_lag,
        wal_receive_lsn,
        wal_replay_lsn,
    ))
}

fn get_replication_lag_text(lag: Option<f64>) -> String {
    match lag {
        Some(lag_seconds) => {
            if lag_seconds < 0.0 {
                "N/A".yellow().to_string()
            } else if lag_seconds < 0.1 {
                format!("{:.2}s", lag_seconds).green().to_string()
            } else if lag_seconds < 1.0 {
                format!("{:.2}s", lag_seconds).yellow().to_string()
            } else if lag_seconds < 5.0 {
                format!("{:.1}s", lag_seconds).red().to_string()
            } else {
                format!("{:.1}s", lag_seconds).red().bold().to_string()
            }
        }
        None => "N/A".yellow().to_string(),
    }
}

async fn show_recommendations(nodes: &[crate::client::NodeInfo], demo: bool) -> Result<()> {
    if demo {
        println!("  [DEMO MODE] Simulating recommendations...");
        println!("  • Consider promoting node-2 to primary for better performance");
        println!("  • Monitor replication lag on node-3");
        println!("  • Verify network connectivity between nodes");
        return Ok(());
    }

    // Analyze nodes and provide recommendations
    let unhealthy_nodes: Vec<_> = nodes.iter().filter(|node| !node.is_healthy).collect();
    let high_lag_nodes: Vec<_> = nodes
        .iter()
        .filter(|node| !node.is_primary && node.replication_lag_seconds.unwrap_or(0.0) > 1.0)
        .collect();

    if !unhealthy_nodes.is_empty() {
        println!("  • {} unhealthy node(s) detected:", unhealthy_nodes.len());
        for node in &unhealthy_nodes {
            println!(
                "    - Node {} (Health: {:.2}%)",
                node.node_id, node.health_score
            );
        }
    }

    if !high_lag_nodes.is_empty() {
        println!(
            "  • {} node(s) with high replication lag:",
            high_lag_nodes.len()
        );
        for node in &high_lag_nodes {
            println!(
                "    - Node {} (Lag: {:.1}s)",
                node.node_id,
                node.replication_lag_seconds.unwrap_or(0.0)
            );
        }
    }

    if unhealthy_nodes.is_empty() && high_lag_nodes.is_empty() {
        println!("  • All nodes are healthy and replicating properly");
    }

    // Performance recommendations
    let avg_health = nodes.iter().map(|n| n.health_score).sum::<f64>() / nodes.len() as f64;
    if avg_health < 70.0 {
        println!(
            "  • Overall cluster health is low ({:.1}%). Consider investigation.",
            avg_health
        );
    }

    Ok(())
}
