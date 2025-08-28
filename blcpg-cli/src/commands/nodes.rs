// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct NodeTable {
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
    #[tabled(rename = "Last Seen")]
    last_seen: String,
}

pub async fn execute(client: &impl ApiClientTrait) -> Result<()> {
    let cluster_status = client.get_cluster_status().await?;

    println!("{}", "Cluster Nodes".bold().blue());
    println!("{}", "=".repeat(20));
    println!("Total Nodes: {}", cluster_status.total_nodes);
    println!("Healthy Nodes: {}", cluster_status.healthy_nodes);
    println!();

    let table_data: Vec<NodeTable> = cluster_status
        .nodes
        .iter()
        .map(|node| NodeTable {
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
            last_seen: node.last_seen.clone(),
        })
        .collect();

    println!("{}", Table::new(table_data));

    Ok(())
}
