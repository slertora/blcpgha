// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(client: &impl ApiClientTrait) -> Result<()> {
    let status = client.get_status().await?;
    let cluster_status = client.get_cluster_status().await?;

    println!("{}", "System Metrics & Cluster Leadership".bold().blue());
    println!("{}", "=".repeat(50));

    // Raft Information
    println!("{}", "RAFT CONSENSUS:".bold().cyan());
    println!("  Current Term: {}", status.raft.current_term);
    println!(
        "  Is Leader: {}",
        if status.raft.is_leader {
            "YES".green().bold()
        } else {
            "NO".yellow()
        }
    );
    println!(
        "  Leader ID: {}",
        status
            .raft
            .leader_id
            .as_ref()
            .unwrap_or(&"None".to_string())
    );
    println!("  Commit Index: {}", status.raft.commit_index);
    println!("  Last Applied: {}", status.raft.last_applied);
    println!(
        "  Peers: {}",
        if status.raft.peers.is_empty() {
            "None".yellow().to_string()
        } else {
            status.raft.peers.join(", ")
        }
    );
    println!();

    // PostgreSQL Information
    println!("{}", "POSTGRESQL STATUS:".bold().cyan());
    println!(
        "  Is Primary: {}",
        if status.health.is_primary {
            "YES".green().bold()
        } else {
            "NO".yellow()
        }
    );
    println!("  Health Score: {:.2}%", status.health.score);
    println!(
        "  Replication Lag: {}",
        get_replication_lag_text(status.health.replication_lag_seconds)
    );
    println!(
        "  Server Version: {}",
        status
            .health
            .server_version
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
    );
    println!(
        "  Current LSN: {}",
        status
            .health
            .current_lsn
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
    );
    println!();

    // Cluster Information
    println!("{}", "CLUSTER OVERVIEW:".bold().cyan());
    println!("  Total Nodes: {}", cluster_status.total_nodes);
    println!("  Healthy Nodes: {}", cluster_status.healthy_nodes);
    println!(
        "  Cluster Leader: {}",
        cluster_status
            .leader
            .as_ref()
            .unwrap_or(&"None".to_string())
    );
    println!();

    // Node Details
    println!("{}", "NODE DETAILS:".bold().cyan());
    for node in &cluster_status.nodes {
        println!("  Node: {}", node.node_id.bold());
        println!(
            "    Raft Leader: {}",
            if node.is_leader {
                "YES".green()
            } else {
                "NO".yellow()
            }
        );
        println!(
            "    PostgreSQL Primary: {}",
            if node.is_primary {
                "YES".green()
            } else {
                "NO".yellow()
            }
        );
        println!(
            "    Health: {}",
            if node.is_healthy {
                "Healthy".green()
            } else {
                "Unhealthy".red()
            }
        );
        println!("    Health Score: {:.2}%", node.health_score);
        println!(
            "    Replication Lag: {}",
            get_replication_lag_text(node.replication_lag_seconds)
        );
        println!("    Last Seen: {}", node.last_seen);
        println!();
    }

    // Error Metrics
    println!("{}", "ERROR METRICS:".bold().cyan());
    println!("  Error Count: {}", status.health.error_count);
    println!(
        "  Consecutive Failures: {}",
        status.health.consecutive_failures
    );
    println!("  Last Check: {}", status.health.last_check);
    println!();

    // Summary
    println!("{}", "SUMMARY:".bold().cyan());
    let raft_leader = if status.raft.is_leader {
        "This node".green().to_string()
    } else {
        status
            .raft
            .leader_id
            .as_ref()
            .unwrap_or(&"None".to_string())
            .to_string()
    };
    let pg_primary = if status.health.is_primary {
        "This node".green().to_string()
    } else {
        "None".yellow().to_string()
    };

    println!("  Raft Leader: {}", raft_leader);
    println!("  PostgreSQL Primary: {}", pg_primary);
    println!(
        "  Overall Status: {}",
        if status.health.is_healthy {
            "Healthy".green()
        } else {
            "Unhealthy".red()
        }
    );

    Ok(())
}

fn get_replication_lag_text(lag: Option<f64>) -> String {
    match lag {
        Some(lag_seconds) => {
            if lag_seconds < 0.0 {
                "N/A".yellow().to_string()
            } else if lag_seconds < 1.0 {
                format!("{:.2}s", lag_seconds)
            } else if lag_seconds < 60.0 {
                format!("{:.1}s", lag_seconds)
            } else {
                format!("{:.1}m", lag_seconds / 60.0)
            }
        }
        None => "N/A".yellow().to_string(),
    }
}
