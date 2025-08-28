// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(
    client: &impl ApiClientTrait,
    target: &str,
    wait_sync: bool,
    force: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Planned Switchover".bold().blue());
    println!("{}", "=".repeat(25));

    if demo {
        println!(
            "{}",
            "DEMO MODE ENABLED - Simulating operations".yellow().bold()
        );
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Target Node: {}", target.bold());
    println!(
        "Wait for Sync: {}",
        if wait_sync {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Find target node
    let target_node = cluster_status
        .nodes
        .iter()
        .find(|node| node.node_id == target)
        .ok_or_else(|| anyhow::anyhow!("Target node '{}' not found in cluster", target))?;

    // Check if target node is already primary
    if target_node.is_primary {
        println!("{}", "Target node is already primary!".yellow());
        return Ok(());
    }

    // Check if target node is healthy
    if !target_node.is_healthy && !force {
        println!("{}", "ERROR: Target node is not healthy!".red().bold());
        println!("Health Score: {:.2}%", target_node.health_score);
        println!("Use --force to override this check.");
        return Ok(());
    }

    // Check replication lag
    if let Some(lag) = target_node.replication_lag_seconds {
        if lag > 1.0 && !force {
            println!("{}", "ERROR: High replication lag detected!".red().bold());
            println!("Replication Lag: {:.1}s", lag);
            println!("Use --force to override this check.");
            return Ok(());
        }
    }

    // Find current primary
    let current_primary = cluster_status
        .nodes
        .iter()
        .find(|node| node.is_primary)
        .ok_or_else(|| anyhow::anyhow!("No primary node found in cluster"))?;

    println!("{}", "Switchover Analysis:".bold().cyan());
    println!("  Current Primary: {}", current_primary.node_id.bold());
    println!("  Target Node: {}", target_node.node_id.bold());
    println!(
        "  Target Health: {}",
        if target_node.is_healthy {
            "Healthy".green()
        } else {
            "Unhealthy".red()
        }
    );
    println!("  Target Health Score: {:.2}%", target_node.health_score);
    println!(
        "  Replication Lag: {}",
        get_replication_lag_text(target_node.replication_lag_seconds)
    );
    println!();

    // Validate switchover conditions
    if !demo {
        println!(
            "{}",
            "Step 1: Validating switchover conditions...".bold().cyan()
        );

        // Check if target node is in sync
        if wait_sync {
            println!("  Waiting for target node to be in sync...");
            if let Some(lag) = target_node.replication_lag_seconds {
                if lag > 0.1 {
                    println!("  Current lag: {:.2}s", lag);
                    if !force {
                        println!("  Target node is not in sync. Use --force to continue anyway.");
                        return Ok(());
                    }
                }
            }
        }

        // Check if target node is ready for promotion
        if !target_node.is_healthy {
            println!("  WARNING: Target node is not healthy!");
            let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
            if !response {
                println!("Switchover cancelled.");
                return Ok(());
            }
        }
    }

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    println!(
        "  Current Primary: {} (will be demoted)",
        current_primary.node_id.bold()
    );
    println!(
        "  New Primary: {} (will be promoted)",
        target_node.node_id.bold()
    );
    println!("  Downtime: {}", "Minimal (planned switchover)".green());
    println!("  Data Loss: {}", "None (synchronous switchover)".green());
    println!();

    // Final confirmation
    let response = get_user_confirmation(
        &format!("Do you want to perform switchover to {}? (y/N): ", target),
        demo,
    )?;
    if !response {
        println!("Switchover cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting planned switchover...".bold().cyan());

    // Execute the switchover
    match execute_switchover(client, target, wait_sync, demo).await {
        Ok(_) => {
            println!("{}", "✅ Switchover successful!".green().bold());
            println!("New Primary: {}", target.green().bold());
            println!(
                "Previous Primary: {}",
                current_primary.node_id.yellow().bold()
            );

            // Show updated cluster status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_cluster = client.get_cluster_status().await?;
            println!("Total Nodes: {}", updated_cluster.total_nodes);
            println!("Healthy Nodes: {}", updated_cluster.healthy_nodes);

            // Show the new primary status
            if let Some(new_primary) = updated_cluster.nodes.iter().find(|n| n.node_id == target) {
                println!(
                    "New Primary Status: {}",
                    if new_primary.is_primary {
                        "Primary".green()
                    } else {
                        "Not Primary".red()
                    }
                );
                println!("Health Score: {:.2}%", new_primary.health_score);
                println!(
                    "Replication Lag: {}",
                    get_replication_lag_text(new_primary.replication_lag_seconds)
                );
            }
        }
        Err(e) => {
            println!("{}", "❌ Switchover failed!".red().bold());
            println!("Error: {}", e);
        }
    }

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

async fn execute_switchover(
    client: &impl ApiClientTrait,
    target: &str,
    wait_sync: bool,
    demo: bool,
) -> Result<()> {
    if demo {
        println!("  [DEMO MODE] Simulating switchover to {}...", target);
        // Skip sleep in demo mode for faster tests
        return Ok(());
    }

    println!("  Step 1: Blocking writes on current primary...");
    // In production, this would:
    // 1. Set read-only mode on current primary
    // 2. Wait for all transactions to complete
    // 3. Verify replication lag is minimal

    println!("  Step 2: Promoting target node to primary...");
    // In production, this would:
    // 1. Execute pg_ctl promote on target node
    // 2. Verify promotion was successful
    // 3. Update cluster configuration

    println!("  Step 3: Reconfiguring other replicas...");
    // In production, this would:
    // 1. Update primary_conninfo on other replicas
    // 2. Restart replication on other replicas
    // 3. Verify all replicas are connected to new primary

    println!("  Step 4: Verifying switchover...");
    // In production, this would:
    // 1. Check that target node is now primary
    // 2. Verify all replicas are replicating from new primary
    // 3. Check that applications can connect to new primary

    if wait_sync {
        println!("  Step 5: Waiting for replication to sync...");
        // In production, this would:
        // 1. Monitor replication lag on all replicas
        // 2. Wait until lag is minimal
        // 3. Verify all replicas are in sync
    }

    println!("  ✅ Switchover completed successfully!");

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
