// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct StatusTable {
    #[tabled(rename = "Node ID")]
    node_id: String,
    #[tabled(rename = "Role")]
    role: String,
    #[tabled(rename = "Health Score")]
    health_score: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Replication Lag")]
    replication_lag: String,
    #[tabled(rename = "Server Version")]
    server_version: String,
    #[tabled(rename = "Last Check")]
    last_check: String,
}

pub async fn execute(client: &impl ApiClientTrait, detailed: bool) -> Result<()> {
    let status = client.get_status().await?;

    if detailed {
        println!("{}", "Node Status (Detailed)".bold().blue());
        println!("{}", "=".repeat(50));
        println!("Node ID: {}", status.raft.node_id.bold());
        println!("Role: {}", get_role_text(&status.raft));
        println!("Health Score: {:.2}%", status.health.score);
        println!("Status: {}", get_status_text(status.health.is_healthy));
        println!("Replication Lag: {}", get_replication_lag_text(status.health.replication_lag_seconds));
        println!("Server Version: {}", status.health.server_version.as_ref().unwrap_or(&"Unknown".to_string()));
        println!("Current LSN: {}", status.health.current_lsn.as_ref().unwrap_or(&"Unknown".to_string()));
        println!("Last Check: {}", status.health.last_check);
        println!("Error Count: {}", status.health.error_count);
        println!("Consecutive Failures: {}", status.health.consecutive_failures);
        println!("Raft Term: {}", status.raft.current_term);
        println!("Is Leader: {}", if status.raft.is_leader { "Yes".green() } else { "No".yellow() });
    } else {
        let table = StatusTable {
            node_id: status.raft.node_id.clone(),
            role: get_role_text(&status.raft),
            health_score: format!("{:.2}%", status.health.score),
            status: get_status_text(status.health.is_healthy),
            replication_lag: get_replication_lag_text(status.health.replication_lag_seconds),
            server_version: status.health.server_version.clone().unwrap_or_else(|| "Unknown".to_string()),
            last_check: status.health.last_check.clone(),
        };

        println!("{}", Table::new([table]));
    }

    Ok(())
}

fn get_role_text(raft: &crate::client::RaftInfo) -> String {
    if raft.is_leader {
        "Leader".green().to_string()
    } else {
        "Follower".yellow().to_string()
    }
}

fn get_status_text(is_healthy: bool) -> String {
    if is_healthy {
        "Healthy".green().to_string()
    } else {
        "Unhealthy".red().to_string()
    }
}

fn get_replication_lag_text(lag: Option<f64>) -> String {
    match lag {
        Some(lag_seconds) => {
            if lag_seconds < 1.0 {
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