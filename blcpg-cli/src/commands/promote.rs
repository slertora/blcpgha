// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(client: &impl ApiClientTrait, node_id: &str) -> Result<()> {
    println!("{}", "Node Promotion Analysis".bold().blue());
    println!("{}", "=".repeat(30));

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    // Find the target node
    let target_node = if node_id == "auto" || node_id == "best" {
        // Auto-select the best candidate
        select_best_candidate(&cluster_status.nodes)?
    } else {
        // Find the specified node
        cluster_status.nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .ok_or_else(|| anyhow::anyhow!("Node '{}' not found in cluster", node_id))?
    };

    println!("Target Node: {}", target_node.node_id.bold());
    println!("Current Health: {}", if target_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
    println!("Health Score: {:.2}%", target_node.health_score);
    println!("Replication Lag: {}", get_replication_lag_text(target_node.replication_lag_seconds));
    println!("Is Primary: {}", if target_node.is_primary { "YES".green() } else { "NO".yellow() });
    println!();

    // Check if the target node is already primary
    if target_node.is_primary {
        println!("{}", "Node is already primary!".yellow());
        return Ok(());
    }

    // Validate the candidate
    if !target_node.is_healthy {
        println!("{}", "WARNING: Target node is not healthy!".red().bold());
        println!("Health Score: {:.2}%", target_node.health_score);
        
        let response = get_user_confirmation("Do you want to force promotion anyway? (y/N): ")?;
        if !response {
            println!("Promotion cancelled.");
            return Ok(());
        }
    }

    // Check replication lag
    if let Some(lag) = target_node.replication_lag_seconds {
        if lag > 10.0 {
            println!("{}", "WARNING: High replication lag detected!".red().bold());
            println!("Replication Lag: {:.1}s", lag);
            
            let response = get_user_confirmation("Do you want to continue with promotion? (y/N): ")?;
            if !response {
                println!("Promotion cancelled.");
                return Ok(());
            }
        }
    }

    // Show current primary
    if let Some(current_primary) = cluster_status.nodes.iter().find(|n| n.is_primary) {
        println!("Current Primary: {}", current_primary.node_id.bold());
        println!("Will be demoted: {}", current_primary.node_id.yellow());
    }

    println!();
    println!("{}", "Promoting node...".bold().cyan());
    
    // Execute promotion
    match client.promote_node(&target_node.node_id).await {
        Ok(_) => {
            println!("{}", "✅ Promotion successful!".green().bold());
            println!("New Primary: {}", target_node.node_id.green().bold());
            
            // Show updated status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_status = client.get_status().await?;
            println!("Primary Node: {}", if updated_status.health.is_primary { "This node".green() } else { "None".yellow() });
            println!("Health Score: {:.2}%", updated_status.health.score);
        }
        Err(e) => {
            println!("{}", "❌ Promotion failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn select_best_candidate(nodes: &[crate::client::NodeInfo]) -> Result<&crate::client::NodeInfo> {
    // Filter healthy nodes
    let healthy_nodes: Vec<_> = nodes.iter().filter(|node| node.is_healthy).collect();
    
    if healthy_nodes.is_empty() {
        return Err(anyhow::anyhow!("No healthy nodes available for promotion"));
    }

    // Sort by health score (descending) and replication lag (ascending)
    let mut candidates: Vec<_> = healthy_nodes.iter().collect();
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

    let best_candidate = candidates.first()
        .ok_or_else(|| anyhow::anyhow!("No suitable candidates found"))?;

    println!("{}", "Auto-selected best candidate:".bold().cyan());
    println!("  Node ID: {}", best_candidate.node_id.bold());
    println!("  Health Score: {:.2}%", best_candidate.health_score);
    println!("  Replication Lag: {}", get_replication_lag_text(best_candidate.replication_lag_seconds));
    println!();

    Ok(best_candidate)
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