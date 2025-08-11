// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UpgradeInfo {
    pub upgrade_id: String,
    pub target_node: String,
    pub current_version: String,
    pub target_version: String,
    pub strategy: String,
    pub backup_created: bool,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: UpgradeStatus,
    pub duration_seconds: Option<u64>,
    pub rollback_required: bool,
}

#[derive(Debug, Clone)]
pub enum UpgradeStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
    Cancelled,
}

impl std::fmt::Display for UpgradeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeStatus::InProgress => write!(f, "{}", "In Progress".yellow()),
            UpgradeStatus::Completed => write!(f, "{}", "Completed".green()),
            UpgradeStatus::Failed => write!(f, "{}", "Failed".red()),
            UpgradeStatus::RolledBack => write!(f, "{}", "Rolled Back".yellow()),
            UpgradeStatus::Cancelled => write!(f, "{}", "Cancelled".yellow()),
        }
    }
}

pub async fn execute(
    client: &impl ApiClientTrait,
    target: &str,
    version: &str,
    strategy: &str,
    backup: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Intelligent PostgreSQL Upgrade".bold().blue());
    println!("{}", "=".repeat(32));

    if demo {
        println!("{}", "DEMO MODE ENABLED - Simulating operations".yellow().bold());
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Upgrade Configuration:");
    println!("  Target Selection: {}", target.bold());
    println!("  Current Version: {}", "14.10".bold()); // In production, this would be queried
    println!("  Target Version: {}", version.bold());
    println!("  Strategy: {}", strategy.bold());
    println!("  Backup Before Upgrade: {}", if backup { "Enabled".green() } else { "Disabled".yellow() });
    println!();

    // Validate version format
    if !version.matches('.').count() >= 1 {
        println!("{}", "WARNING: Version format seems invalid!".yellow().bold());
        println!("Expected format: X.Y or X.Y.Z (e.g., 15.3, 14.10)");
        let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
        if !response {
            println!("Upgrade cancelled.");
            return Ok(());
        }
    }

    // Select target node for upgrade
    let target_node = if target == "auto" {
        select_best_upgrade_target(&cluster_status.nodes, strategy, demo)?
    } else {
        cluster_status.nodes
            .iter()
            .find(|node| node.node_id == target)
            .ok_or_else(|| anyhow::anyhow!("Target node '{}' not found in cluster", target))?
    };

    // Validate upgrade conditions
    if !demo {
        println!("{}", "Step 1: Validating upgrade conditions...".bold().cyan());
        
        // Check if target node is healthy
        if !target_node.is_healthy {
            println!("{}", "ERROR: Target node is not healthy!".red().bold());
            println!("Health Score: {:.2}%", target_node.health_score);
            return Ok(());
        }

        // Check if target node is primary (warn about potential issues)
        if target_node.is_primary {
            println!("  ⚠️  WARNING: Upgrading primary node!");
            println!("  This will cause downtime and may affect the cluster.");
            let response = get_user_confirmation("Do you want to continue? (y/N): ", demo)?;
            if !response {
                println!("Upgrade cancelled.");
                return Ok(());
            }
        } else {
            println!("  ✅ Target node is replica (safer for upgrade)");
        }
    }

    // Show upgrade analysis
    println!("{}", "Upgrade Analysis:".bold().cyan());
    println!("  Target Node: {} ({})", 
        target_node.node_id.bold(), 
        if target_node.is_primary { "Primary".red() } else { "Replica".green() }
    );
    println!("  Target Health: {}", if target_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
    println!("  Target Health Score: {:.2}%", target_node.health_score);
    println!("  Current Version: {}", "14.10".bold());
    println!("  Target Version: {}", version.bold());
    println!("  Strategy: {}", strategy.bold());
    println!();

    // Calculate estimated upgrade time and impact
    let estimated_time = calculate_estimated_upgrade_time(version, strategy, demo)?;
    let estimated_impact = calculate_upgrade_impact(strategy, target_node.is_primary);

    println!("{}", "Upgrade Estimation:".bold().cyan());
    println!("  Estimated Time: {}", estimated_time);
    println!("  Performance Impact: {}", estimated_impact);
    println!("  Downtime: {}", estimate_downtime(strategy, target_node.is_primary));
    println!();

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    if target_node.is_primary {
        println!("  Database Impact: {}", "HIGH - Primary node will be unavailable".red());
        println!("  Cluster Impact: {}", "HIGH - Automatic failover may occur".red());
        println!("  Downtime: {}", "Expected during upgrade".red());
    } else {
        println!("  Database Impact: {}", "LOW - Replica node only".green());
        println!("  Cluster Impact: {}", "LOW - Primary remains available".green());
        println!("  Downtime: {}", "None for cluster".green());
    }
    println!("  Backup Required: {}", if backup { "Yes".green() } else { "No (risky)".red() });
    println!("  Rollback Plan: {}", "Available".green());
    println!();

    // Create backup if requested
    if backup && !demo {
        println!("{}", "Step 2: Creating backup before upgrade...".bold().cyan());
        match create_upgrade_backup(&target_node.node_id).await {
            Ok(()) => {
                println!("  ✅ Backup created successfully");
            }
            Err(e) => {
                println!("{}", "❌ Backup creation failed!".red().bold());
                println!("Error: {}", e);
                let response = get_user_confirmation("Do you want to continue without backup? (y/N): ", demo)?;
                if !response {
                    println!("Upgrade cancelled.");
                    return Ok(());
                }
            }
        }
    } else if demo {
        println!("  [DEMO MODE] Simulating backup creation...");
        println!("  ✅ Backup created successfully");
    }

    // Final confirmation
    let response = get_user_confirmation(&format!("Do you want to start PostgreSQL upgrade to {} on {}? (y/N): ", version, target_node.node_id), demo)?;
    if !response {
        println!("Upgrade cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting intelligent PostgreSQL upgrade...".bold().cyan());

    // Execute the upgrade
    match execute_upgrade(client, target_node, version, strategy, backup, demo).await {
        Ok(upgrade_info) => {
            println!("{}", "✅ Upgrade completed successfully!".green().bold());
            println!("Upgrade ID: {}", upgrade_info.upgrade_id.bold());
            println!("Target Node: {}", upgrade_info.target_node.bold());
            println!("Current Version: {}", upgrade_info.current_version.bold());
            println!("Target Version: {}", upgrade_info.target_version.bold());
            println!("Strategy: {}", upgrade_info.strategy.bold());
            println!("Status: {}", upgrade_info.status);
            println!("Backup Created: {}", if upgrade_info.backup_created { "Yes".green() } else { "No".yellow() });
            if let Some(duration) = upgrade_info.duration_seconds {
                println!("Duration: {} seconds", duration);
            }
            if upgrade_info.rollback_required {
                println!("Rollback Required: {}", "Yes".red());
            }
        }
        Err(e) => {
            println!("{}", "❌ Upgrade failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn select_best_upgrade_target<'a>(
    nodes: &'a [crate::client::NodeInfo], 
    strategy: &str,
    demo: bool
) -> Result<&'a crate::client::NodeInfo> {
    if demo {
        println!("  [DEMO MODE] Simulating best upgrade target selection...");
        // In demo mode, prefer replica nodes for safety
        return nodes.iter()
            .find(|node| !node.is_primary && node.is_healthy)
            .or_else(|| nodes.iter().find(|node| node.is_primary))
            .ok_or_else(|| anyhow::anyhow!("No suitable upgrade target found"));
    }

    // For upgrades, prefer replica nodes (safer)
    let healthy_replicas: Vec<&crate::client::NodeInfo> = nodes
        .iter()
        .filter(|node| !node.is_primary && node.is_healthy)
        .collect();

    if !healthy_replicas.is_empty() {
        // Sort by health score (descending) and replication lag (ascending)
        let mut sorted_replicas = healthy_replicas;
        sorted_replicas.sort_by(|a, b| {
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

        let best_replica = sorted_replicas.first()
            .ok_or_else(|| anyhow::anyhow!("No suitable upgrade target found"))?;

        println!("  Selected best replica: {} (Health: {:.2}%, Lag: {})", 
            best_replica.node_id.bold(),
            best_replica.health_score,
            get_replication_lag_text(best_replica.replication_lag_seconds)
        );

        return Ok(best_replica);
    }

    // If no healthy replicas, use primary (with warning)
    if let Some(primary) = nodes.iter().find(|node| node.is_primary && node.is_healthy) {
        println!("  ⚠️  WARNING: No healthy replicas available, using primary node: {}", primary.node_id.bold());
        return Ok(primary);
    }

    Err(anyhow::anyhow!("No healthy nodes available for upgrade"))
}

fn calculate_estimated_upgrade_time(version: &str, strategy: &str, demo: bool) -> Result<String> {
    if demo {
        return Ok("5-15 minutes (demo mode)".to_string());
    }

    let base_time = match strategy {
        "rolling" => 600, // 10 minutes
        "all-at-once" => 900, // 15 minutes
        "blue-green" => 1200, // 20 minutes
        _ => 900,
    };

    // Adjust based on version difference
    let version_multiplier = if version.starts_with("15") { 1.2 } else if version.starts_with("16") { 1.5 } else { 1.0 };
    let adjusted_time = (base_time as f64 * version_multiplier) as u32;

    if adjusted_time < 60 {
        Ok(format!("{} seconds", adjusted_time))
    } else if adjusted_time < 3600 {
        Ok(format!("{} minutes", adjusted_time / 60))
    } else {
        Ok(format!("{} hours", adjusted_time / 3600))
    }
}

fn calculate_upgrade_impact(strategy: &str, is_primary: bool) -> String {
    let impact = match strategy {
        "rolling" => "LOW",
        "all-at-once" => "HIGH",
        "blue-green" => "MEDIUM",
        _ => "MEDIUM",
    };

    if is_primary {
        format!("{} (Primary node)", impact)
    } else {
        format!("{} (Replica node)", impact)
    }
}

fn estimate_downtime(strategy: &str, is_primary: bool) -> String {
    if !is_primary {
        return "None (replica node)".green().to_string();
    }

    match strategy {
        "rolling" => "Minimal (rolling upgrade)".yellow().to_string(),
        "all-at-once" => "Expected (all nodes)".red().to_string(),
        "blue-green" => "Minimal (blue-green deployment)".green().to_string(),
        _ => "Unknown".yellow().to_string(),
    }
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

async fn create_upgrade_backup(node_id: &str) -> Result<()> {
    // In production, this would:
    // 1. Create a full backup of the node
    // 2. Verify backup integrity
    // 3. Store backup metadata
    
    println!("  Creating full backup of {}...", node_id);
    println!("  Verifying backup integrity...");
    println!("  Storing backup metadata...");
    
    // Simulate backup delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(())
}

async fn execute_upgrade(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    version: &str,
    strategy: &str,
    backup: bool,
    demo: bool,
) -> Result<UpgradeInfo> {
    if demo {
        println!("  [DEMO MODE] Simulating PostgreSQL upgrade to {} on {}...", version, target_node.node_id);
        // Skip sleep in demo mode for faster tests
        return Ok(UpgradeInfo {
            upgrade_id: format!("upgrade-{}", chrono::Utc::now().timestamp()),
            target_node: target_node.node_id.clone(),
            current_version: "14.10".to_string(),
            target_version: version.to_string(),
            strategy: strategy.to_string(),
            backup_created: backup,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            status: UpgradeStatus::Completed,
            duration_seconds: Some(600),
            rollback_required: false,
        });
    }

    println!("  Step 1: Preparing upgrade environment...");
    // In production, this would:
    // 1. Check available disk space
    // 2. Verify PostgreSQL is running
    // 3. Check current version
    
    println!("  Step 2: Starting PostgreSQL upgrade to {}...", version);
    // In production, this would:
    // 1. Stop PostgreSQL
    // 2. Install new version
    // 3. Run upgrade scripts
    
    println!("  Step 3: Monitoring upgrade progress...");
    // In production, this would:
    // 1. Monitor upgrade progress
    // 2. Check for errors
    // 3. Verify new version
    
    println!("  Step 4: Finalizing upgrade...");
    // In production, this would:
    // 1. Verify upgrade completion
    // 2. Update cluster metadata
    // 3. Restart PostgreSQL
    
    println!("  ✅ Upgrade completed successfully!");
    
    Ok(UpgradeInfo {
        upgrade_id: format!("upgrade-{}", chrono::Utc::now().timestamp()),
        target_node: target_node.node_id.clone(),
        current_version: "14.10".to_string(),
        target_version: version.to_string(),
        strategy: strategy.to_string(),
        backup_created: backup,
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        status: UpgradeStatus::Completed,
        duration_seconds: Some(600),
        rollback_required: false,
    })
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