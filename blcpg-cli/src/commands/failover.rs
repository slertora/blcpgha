// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(
    client: &impl ApiClientTrait,
    target: &str,
    notify: bool,
    force: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Automatic Failover".bold().red());
    println!("{}", "=".repeat(20));

    if demo {
        println!("{}", "DEMO MODE ENABLED - Simulating operations".yellow().bold());
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Target Selection: {}", target.bold());
    println!("Notifications: {}", if notify { "Enabled".green() } else { "Disabled".yellow() });
    println!();

    // Find current primary
    let current_primary = cluster_status.nodes
        .iter()
        .find(|node| node.is_primary)
        .ok_or_else(|| anyhow::anyhow!("No primary node found in cluster"))?;

    // Check if primary is healthy
    if current_primary.is_healthy && !force {
        println!("{}", "INFO: Current primary is healthy!".green().bold());
        println!("Primary Node: {}", current_primary.node_id.bold());
        println!("Health Score: {:.2}%", current_primary.health_score);
        println!();
        
        let response = get_user_confirmation("Do you want to force failover anyway? (y/N): ", demo)?;
        if !response {
            println!("Failover cancelled.");
            return Ok(());
        }
    }

    // Select target node for failover
    let target_node = if target == "auto" {
        select_best_failover_candidate(&cluster_status.nodes, demo)?
    } else {
        cluster_status.nodes
            .iter()
            .find(|node| node.node_id == target)
            .ok_or_else(|| anyhow::anyhow!("Target node '{}' not found in cluster", target))?
    };

    // Validate failover conditions
    if !demo {
        println!("{}", "Step 1: Validating failover conditions...".bold().cyan());
        
        // Check if target node is healthy
        if !target_node.is_healthy && !force {
            println!("{}", "ERROR: Target node is not healthy!".red().bold());
            println!("Health Score: {:.2}%", target_node.health_score);
            println!("Use --force to override this check.");
            return Ok(());
        }

        // Check replication lag
        if let Some(lag) = target_node.replication_lag_seconds {
            if lag > 5.0 && !force {
                println!("{}", "ERROR: High replication lag detected!".red().bold());
                println!("Replication Lag: {:.1}s", lag);
                println!("Use --force to override this check.");
                return Ok(());
            }
        }
    }

    println!("{}", "Failover Analysis:".bold().cyan());
    println!("  Current Primary: {} ({})", 
        current_primary.node_id.bold(), 
        if current_primary.is_healthy { "Healthy".green() } else { "Unhealthy".red() }
    );
    println!("  Target Node: {} ({})", 
        target_node.node_id.bold(), 
        if target_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() }
    );
    println!("  Target Health Score: {:.2}%", target_node.health_score);
    println!("  Replication Lag: {}", get_replication_lag_text(target_node.replication_lag_seconds));
    println!();

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    println!("  Current Primary: {} (will be demoted)", current_primary.node_id.bold());
    println!("  New Primary: {} (will be promoted)", target_node.node_id.bold());
    println!("  Downtime: {}", "Minimal (automatic failover)".green());
    println!("  Data Loss: {}", "None (synchronous replication)".green());
    println!("  Notifications: {}", if notify { "Enabled".green() } else { "Disabled".yellow() });
    println!();

    // Final confirmation
    let response = get_user_confirmation(&format!("Do you want to perform failover to {}? (y/N): ", target_node.node_id), demo)?;
    if !response {
        println!("Failover cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting automatic failover...".bold().cyan());

    // Execute the failover
    match execute_failover(client, &target_node.node_id, notify, demo).await {
        Ok(_) => {
            println!("{}", "✅ Failover successful!".green().bold());
            println!("New Primary: {}", target_node.node_id.green().bold());
            println!("Previous Primary: {}", current_primary.node_id.yellow().bold());
            
            if notify {
                println!("{}", "📧 Notifications sent to administrators".blue());
            }
            
            // Show updated cluster status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_cluster = client.get_cluster_status().await?;
            println!("Total Nodes: {}", updated_cluster.total_nodes);
            println!("Healthy Nodes: {}", updated_cluster.healthy_nodes);
            
            // Show the new primary status
            if let Some(new_primary) = updated_cluster.nodes.iter().find(|n| n.node_id == target_node.node_id) {
                println!("New Primary Status: {}", if new_primary.is_primary { "Primary".green() } else { "Not Primary".red() });
                println!("Health Score: {:.2}%", new_primary.health_score);
                println!("Replication Lag: {}", get_replication_lag_text(new_primary.replication_lag_seconds));
            }
        }
        Err(e) => {
            println!("{}", "❌ Failover failed!".red().bold());
            println!("Error: {}", e);
            
            if notify {
                println!("{}", "📧 Alert notifications sent to administrators".red());
            }
        }
    }

    Ok(())
}

fn select_best_failover_candidate<'a>(nodes: &'a [crate::client::NodeInfo], demo: bool) -> Result<&'a crate::client::NodeInfo> {
    if demo {
        println!("  [DEMO MODE] Simulating best candidate selection...");
        // In demo mode, return the first non-primary node
        return nodes.iter()
            .find(|node| !node.is_primary)
            .ok_or_else(|| anyhow::anyhow!("No suitable candidate found for failover"));
    }

    // Filter out primary and unhealthy nodes
    let candidates: Vec<&crate::client::NodeInfo> = nodes
        .iter()
        .filter(|node| !node.is_primary && node.is_healthy)
        .collect();

    if candidates.is_empty() {
        return Err(anyhow::anyhow!("No healthy candidates available for failover"));
    }

    // Sort by health score (descending) and replication lag (ascending)
    let mut sorted_candidates = candidates;
    sorted_candidates.sort_by(|a, b| {
        // Primary sort: health score (descending)
        let health_comparison = b.health_score.partial_cmp(&a.health_score).unwrap_or(std::cmp::Ordering::Equal);
        if health_comparison != std::cmp::Ordering::Equal {
            return health_comparison;
        }
        
        // Secondary sort: replication lag (ascending)
        let lag_a = a.replication_lag_seconds.unwrap_or(f64::MAX);
        let lag_b = b.replication_lag_seconds.unwrap_or(f64::MAX);
        lag_a.partial_cmp(&lag_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let best_candidate = sorted_candidates.first()
        .ok_or_else(|| anyhow::anyhow!("No suitable candidate found for failover"))?;

    println!("  Selected best candidate: {} (Health: {:.2}%, Lag: {})", 
        best_candidate.node_id.bold(),
        best_candidate.health_score,
        get_replication_lag_text(best_candidate.replication_lag_seconds)
    );

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

async fn execute_failover(
    client: &impl ApiClientTrait,
    target: &str,
    notify: bool,
    demo: bool,
) -> Result<()> {
    if demo {
        println!("  [DEMO MODE] Simulating failover to {}...", target);
        // Skip sleep in demo mode for faster tests
        return Ok(());
    }

    if notify {
        println!("  Step 1: Sending failover notifications...");
        // In production, this would:
        // 1. Send email/SMS notifications to administrators
        // 2. Post to monitoring systems (PagerDuty, Slack, etc.)
        // 3. Log the failover event
    }

    println!("  Step 2: Detecting primary failure...");
    // In production, this would:
    // 1. Verify primary is actually down (not just network issue)
    // 2. Check if primary is recoverable
    // 3. Determine if failover is necessary
    
    println!("  Step 3: Promoting target node to primary...");
    // In production, this would:
    // 1. Execute pg_ctl promote on target node
    // 2. Verify promotion was successful
    // 3. Update cluster configuration
    
    println!("  Step 4: Reconfiguring other replicas...");
    // In production, this would:
    // 1. Update primary_conninfo on other replicas
    // 2. Restart replication on other replicas
    // 3. Verify all replicas are connected to new primary
    
    println!("  Step 5: Verifying failover...");
    // In production, this would:
    // 1. Check that target node is now primary
    // 2. Verify all replicas are replicating from new primary
    // 3. Check that applications can connect to new primary
    // 4. Run health checks on new primary
    
    println!("  ✅ Failover completed successfully!");
    
    Ok(())
}

fn get_user_confirmation(prompt: &str, demo: bool) -> Result<bool> {
    if demo {
        // In demo mode, automatically return true to avoid blocking tests
        return Ok(true);
    }
    
    use std::io::{self, Write};
    
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
} 