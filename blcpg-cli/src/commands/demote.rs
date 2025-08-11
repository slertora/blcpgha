// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(client: &impl ApiClientTrait, node_id: &str) -> Result<()> {
    println!("{}", "Node Demotion Analysis".bold().blue());
    println!("{}", "=".repeat(30));

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    // Find the target node to demote
    let target_node = if node_id == "primary" || node_id == "current" {
        // Demote the current primary
        cluster_status.nodes
            .iter()
            .find(|node| node.is_primary)
            .ok_or_else(|| anyhow::anyhow!("No primary node found to demote"))?
    } else {
        // Find the specified node
        cluster_status.nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .ok_or_else(|| anyhow::anyhow!("Node '{}' not found in cluster", node_id))?
    };

    println!("Target Node to Demote: {}", target_node.node_id.bold());
    println!("Current Role: {}", if target_node.is_primary { "PRIMARY".red().bold() } else { "REPLICA".yellow() });
    println!("Health Status: {}", if target_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
    println!("Health Score: {:.2}%", target_node.health_score);
    println!();

    // Check if the target node is actually primary
    if !target_node.is_primary {
        println!("{}", "Node is not primary!".yellow());
        println!("Current role: {}", "REPLICA".yellow());
        return Ok(());
    }

    // Check if there are other healthy nodes available
    let healthy_replicas: Vec<_> = cluster_status.nodes
        .iter()
        .filter(|node| node.is_healthy && !node.is_primary)
        .collect();

    if healthy_replicas.is_empty() {
        println!("{}", "WARNING: No healthy replicas available!".red().bold());
        println!("Demoting the primary will leave the cluster without a primary node.");
        
        let response = get_user_confirmation("Do you want to continue with demotion? (y/N): ")?;
        if !response {
            println!("Demotion cancelled.");
            return Ok(());
        }
    } else {
        // Show the best candidate for promotion
        let best_candidate = select_best_candidate(&healthy_replicas)?;
        println!("{}", "Best candidate for promotion:".bold().cyan());
        println!("  Node ID: {}", best_candidate.node_id.bold());
        println!("  Health Score: {:.2}%", best_candidate.health_score);
        println!("  Replication Lag: {}", get_replication_lag_text(best_candidate.replication_lag_seconds));
        println!();
    }

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    println!("  Current Primary: {}", target_node.node_id.bold());
    println!("  Will be demoted to: {}", "REPLICA".yellow());
    if !healthy_replicas.is_empty() {
        let best_candidate = select_best_candidate(&healthy_replicas)?;
        println!("  New Primary will be: {}", best_candidate.node_id.green().bold());
    } else {
        println!("  New Primary: {}", "NONE (Cluster will be without primary)".red().bold());
    }
    println!();

    // Final confirmation
    let response = get_user_confirmation(&format!("Do you want to demote node '{}'? (y/N): ", target_node.node_id))?;
    if !response {
        println!("Demotion cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Demoting node...".bold().cyan());
    
    // Execute demotion
    match client.demote_node(&target_node.node_id).await {
        Ok(_) => {
            println!("{}", "✅ Demotion successful!".green().bold());
            println!("Demoted Node: {}", target_node.node_id.yellow().bold());
            
            // Show updated status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_status = client.get_status().await?;
            println!("Primary Node: {}", if updated_status.health.is_primary { "This node".green() } else { "None".yellow() });
            println!("Health Score: {:.2}%", updated_status.health.score);
            
            // Show new primary if any
            let updated_cluster = client.get_cluster_status().await?;
            if let Some(new_primary) = updated_cluster.nodes.iter().find(|n| n.is_primary) {
                println!("New Primary: {}", new_primary.node_id.green().bold());
            } else {
                println!("{}", "No primary node in cluster".red().bold());
            }
        }
        Err(e) => {
            println!("{}", "❌ Demotion failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn select_best_candidate<'a>(nodes: &'a [&'a crate::client::NodeInfo]) -> Result<&'a crate::client::NodeInfo> {
    if nodes.is_empty() {
        return Err(anyhow::anyhow!("No candidates available"));
    }

    // Sort by health score (descending) and replication lag (ascending)
    let mut candidates: Vec<_> = nodes.iter().collect();
    candidates.sort_by(|a, b| {
        // First, sort by health score (descending)
        b.health_score.partial_cmp(&a.health_score).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                // Then, sort by replication lag (ascending, None is considered highest)
                match (a.replication_lag_seconds, b.replication_lag_seconds) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, _) => std::cmp::Ordering::Greater,
                    (_, None) => std::cmp::Ordering::Less,
                    (Some(lag_a), Some(lag_b)) => lag_a.partial_cmp(&lag_b).unwrap_or(std::cmp::Ordering::Equal),
                }
            })
    });

    Ok(candidates.first().unwrap())
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

fn get_user_confirmation(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
} 