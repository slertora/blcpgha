// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ClusterNodeTable {
    #[tabled(rename = "Node ID")]
    node_id: String,
    #[tabled(rename = "Leader")]
    is_leader: String,
    #[tabled(rename = "Health")]
    is_healthy: String,
    #[tabled(rename = "Health Score")]
    health_score: String,
    #[tabled(rename = "Primary")]
    is_primary: String,
    #[tabled(rename = "Replication Lag")]
    replication_lag: String,
    #[tabled(rename = "Last Seen")]
    last_seen: String,
}

pub async fn execute(client: &impl ApiClientTrait, detailed: bool) -> Result<()> {
    let cluster_status = client.get_cluster_status().await?;

    if detailed {
        println!("{}", "Cluster Information (Detailed)".bold().blue());
        println!("{}", "=".repeat(50));
        println!("Total Nodes: {}", cluster_status.total_nodes);
        println!("Healthy Nodes: {}", cluster_status.healthy_nodes);
        println!(
            "Leader: {}",
            cluster_status
                .leader
                .as_ref()
                .unwrap_or(&"None".to_string())
        );
        println!();

        println!("{}", "Nodes:".bold());
        for node in &cluster_status.nodes {
            println!("  Node ID: {}", node.node_id.bold());
            println!(
                "    Leader: {}",
                if node.is_leader {
                    "Yes".green()
                } else {
                    "No".yellow()
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
                "    Primary: {}",
                if node.is_primary {
                    "Yes".green()
                } else {
                    "No".yellow()
                }
            );
            println!(
                "    Replication Lag: {}",
                get_replication_lag_text(node.replication_lag_seconds)
            );
            println!("    Last Seen: {}", node.last_seen);
            println!();
        }
    } else {
        println!("{}", "Cluster Information".bold().blue());
        println!("{}", "=".repeat(30));
        println!("Total Nodes: {}", cluster_status.total_nodes);
        println!("Healthy Nodes: {}", cluster_status.healthy_nodes);
        println!(
            "Leader: {}",
            cluster_status
                .leader
                .as_ref()
                .unwrap_or(&"None".to_string())
        );
        println!();

        let table_data: Vec<ClusterNodeTable> = cluster_status
            .nodes
            .iter()
            .map(|node| ClusterNodeTable {
                node_id: node.node_id.clone(),
                is_leader: if node.is_leader {
                    "Yes".green().to_string()
                } else {
                    "No".yellow().to_string()
                },
                is_healthy: if node.is_healthy {
                    "Healthy".green().to_string()
                } else {
                    "Unhealthy".red().to_string()
                },
                health_score: format!("{:.2}%", node.health_score),
                is_primary: if node.is_primary {
                    "Yes".green().to_string()
                } else {
                    "No".yellow().to_string()
                },
                replication_lag: get_replication_lag_text(node.replication_lag_seconds),
                last_seen: node.last_seen.clone(),
            })
            .collect();

        println!("{}", Table::new(table_data));
    }

    Ok(())
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
