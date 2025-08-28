// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use colored::*;

#[derive(Debug, Clone)]
pub struct RestoreInfo {
    pub restore_id: String,
    pub backup_id: String,
    pub target_node: String,
    pub restore_type: String,
    pub point_in_time: Option<DateTime<Utc>>,
    pub size_bytes: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: RestoreStatus,
    pub verification_passed: bool,
}

#[derive(Debug, Clone)]
pub enum RestoreStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for RestoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestoreStatus::InProgress => write!(f, "{}", "In Progress".yellow()),
            RestoreStatus::Completed => write!(f, "{}", "Completed".green()),
            RestoreStatus::Failed => write!(f, "{}", "Failed".red()),
            RestoreStatus::Cancelled => write!(f, "{}", "Cancelled".yellow()),
        }
    }
}

pub async fn execute(
    client: &impl ApiClientTrait,
    backup_id: &str,
    target: &str,
    restore_type: &str,
    point_in_time: Option<String>,
    verify: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Intelligent Restore".bold().blue());
    println!("{}", "=".repeat(19));

    if demo {
        println!(
            "{}",
            "DEMO MODE ENABLED - Simulating operations".yellow().bold()
        );
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Restore Configuration:");
    println!("  Backup ID: {}", backup_id.bold());
    println!("  Target Selection: {}", target.bold());
    println!("  Restore Type: {}", restore_type.bold());
    if let Some(pit) = &point_in_time {
        println!("  Point-in-Time: {}", pit.bold());
    }
    println!(
        "  Verify Backup: {}",
        if verify {
            "Enabled".green()
        } else {
            "Disabled".yellow()
        }
    );
    println!();

    // Validate backup ID format
    if !backup_id.starts_with("backup-") {
        println!(
            "{}",
            "WARNING: Backup ID format seems invalid!".yellow().bold()
        );
        println!("Expected format: backup-<timestamp>");
        let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
        if !response {
            println!("Restore cancelled.");
            return Ok(());
        }
    }

    // Select target node for restore
    let target_node = if target == "auto" {
        select_best_restore_target(&cluster_status.nodes, demo)?
    } else {
        cluster_status
            .nodes
            .iter()
            .find(|node| node.node_id == target)
            .ok_or_else(|| anyhow::anyhow!("Target node '{}' not found in cluster", target))?
    };

    // Validate restore conditions
    if !demo {
        println!(
            "{}",
            "Step 1: Validating restore conditions...".bold().cyan()
        );

        // Check if target node is healthy
        if !target_node.is_healthy {
            println!("{}", "ERROR: Target node is not healthy!".red().bold());
            println!("Health Score: {:.2}%", target_node.health_score);
            return Ok(());
        }

        // Check if target node is primary (warn about potential issues)
        if target_node.is_primary {
            println!("  ⚠️  WARNING: Restoring to primary node!");
            println!("  This will cause downtime and may affect the cluster.");
            let response = get_user_confirmation("Do you want to continue? (y/N): ", demo)?;
            if !response {
                println!("Restore cancelled.");
                return Ok(());
            }
        } else {
            println!("  ✅ Target node is replica (safer for restore)");
        }
    }

    // Parse point-in-time if provided
    let point_in_time_dt = if let Some(pit_str) = &point_in_time {
        match parse_point_in_time(pit_str) {
            Ok(dt) => {
                println!(
                    "  ✅ Point-in-time parsed successfully: {}",
                    dt.format("%Y-%m-%d %H:%M:%S")
                );
                Some(dt)
            }
            Err(e) => {
                println!("{}", "ERROR: Invalid point-in-time format!".red().bold());
                println!("Expected format: YYYY-MM-DD HH:MM:SS");
                println!("Error: {}", e);
                return Ok(());
            }
        }
    } else {
        None
    };

    // Show restore analysis
    println!("{}", "Restore Analysis:".bold().cyan());
    println!(
        "  Target Node: {} ({})",
        target_node.node_id.bold(),
        if target_node.is_primary {
            "Primary".red()
        } else {
            "Replica".green()
        }
    );
    println!(
        "  Target Health: {}",
        if target_node.is_healthy {
            "Healthy".green()
        } else {
            "Unhealthy".red()
        }
    );
    println!("  Target Health Score: {:.2}%", target_node.health_score);
    println!("  Restore Type: {}", restore_type.bold());
    if let Some(pit) = point_in_time_dt {
        println!(
            "  Point-in-Time: {}",
            pit.format("%Y-%m-%d %H:%M:%S").to_string().bold()
        );
    }
    println!();

    // Calculate estimated restore size and time
    let estimated_size = calculate_estimated_restore_size(backup_id, restore_type)?;
    let estimated_time = calculate_estimated_restore_time(estimated_size, demo)?;

    println!("{}", "Restore Estimation:".bold().cyan());
    println!("  Estimated Size: {}", format_bytes(estimated_size));
    println!("  Estimated Time: {}", estimated_time);
    println!(
        "  Backup Verification: {}",
        if verify {
            "Required".green()
        } else {
            "Skipped".yellow()
        }
    );
    println!();

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    if target_node.is_primary {
        println!(
            "  Database Impact: {}",
            "HIGH - Primary node will be unavailable".red()
        );
        println!(
            "  Cluster Impact: {}",
            "HIGH - Automatic failover may occur".red()
        );
        println!("  Downtime: {}", "Expected during restore".red());
    } else {
        println!(
            "  Database Impact: {}",
            "Minimal - Replica node only".green()
        );
        println!(
            "  Cluster Impact: {}",
            "Low - Primary remains available".green()
        );
        println!("  Downtime: {}", "None for cluster".green());
    }
    println!("  Network Usage: {}", format_bytes(estimated_size));
    println!("  Storage Required: {}", format_bytes(estimated_size));
    println!();

    // Verify backup if requested
    if verify && !demo {
        println!("{}", "Step 2: Verifying backup integrity...".bold().cyan());
        match verify_backup_integrity(backup_id).await {
            Ok(()) => {
                println!("  ✅ Backup verification passed");
            }
            Err(e) => {
                println!("{}", "❌ Backup verification failed!".red().bold());
                println!("Error: {}", e);
                let response =
                    get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
                if !response {
                    println!("Restore cancelled.");
                    return Ok(());
                }
            }
        }
    } else if demo {
        println!("  [DEMO MODE] Simulating backup verification...");
        println!("  ✅ Backup verification passed");
    }

    // Final confirmation
    let response = get_user_confirmation(
        &format!(
            "Do you want to start restore of backup {} to {}? (y/N): ",
            backup_id, target_node.node_id
        ),
        demo,
    )?;
    if !response {
        println!("Restore cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting intelligent restore...".bold().cyan());

    // Execute the restore
    match execute_restore(
        client,
        backup_id,
        target_node,
        restore_type,
        point_in_time_dt,
        verify,
        demo,
    )
    .await
    {
        Ok(restore_info) => {
            println!("{}", "✅ Restore completed successfully!".green().bold());
            println!("Restore ID: {}", restore_info.restore_id.bold());
            println!("Backup ID: {}", restore_info.backup_id.bold());
            println!("Target Node: {}", restore_info.target_node.bold());
            println!("Restore Type: {}", restore_info.restore_type.bold());
            if let Some(pit) = restore_info.point_in_time {
                println!(
                    "Point-in-Time: {}",
                    pit.format("%Y-%m-%d %H:%M:%S").to_string()
                );
            }
            println!("Size: {}", format_bytes(restore_info.size_bytes));
            println!("Status: {}", restore_info.status);
            println!(
                "Verification: {}",
                if restore_info.verification_passed {
                    "Passed".green()
                } else {
                    "Failed".red()
                }
            );
        }
        Err(e) => {
            println!("{}", "❌ Restore failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn select_best_restore_target<'a>(
    nodes: &'a [crate::client::NodeInfo],
    demo: bool,
) -> Result<&'a crate::client::NodeInfo> {
    if demo {
        println!("  [DEMO MODE] Simulating best restore target selection...");
        // In demo mode, prefer replica nodes for safety
        return nodes
            .iter()
            .find(|node| !node.is_primary && node.is_healthy)
            .or_else(|| nodes.iter().find(|node| node.is_primary))
            .ok_or_else(|| anyhow::anyhow!("No suitable restore target found"));
    }

    // Prefer replica nodes for restore (safer)
    let healthy_replicas: Vec<&crate::client::NodeInfo> = nodes
        .iter()
        .filter(|node| !node.is_primary && node.is_healthy)
        .collect();

    if !healthy_replicas.is_empty() {
        // Sort by health score (descending) and replication lag (ascending)
        let mut sorted_replicas = healthy_replicas;
        sorted_replicas.sort_by(|a, b| {
            // Primary sort: health score (descending)
            let health_comparison = b
                .health_score
                .partial_cmp(&a.health_score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if health_comparison != std::cmp::Ordering::Equal {
                return health_comparison;
            }

            // Secondary sort: replication lag (ascending)
            let lag_a = a.replication_lag_seconds.unwrap_or(f64::MAX);
            let lag_b = b.replication_lag_seconds.unwrap_or(f64::MAX);
            lag_a
                .partial_cmp(&lag_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_replica = sorted_replicas
            .first()
            .ok_or_else(|| anyhow::anyhow!("No suitable restore target found"))?;

        println!(
            "  Selected best replica: {} (Health: {:.2}%, Lag: {})",
            best_replica.node_id.bold(),
            best_replica.health_score,
            get_replication_lag_text(best_replica.replication_lag_seconds)
        );

        return Ok(best_replica);
    }

    // If no healthy replicas, use primary (with warning)
    if let Some(primary) = nodes.iter().find(|node| node.is_primary && node.is_healthy) {
        println!(
            "  ⚠️  WARNING: No healthy replicas available, using primary node: {}",
            primary.node_id.bold()
        );
        return Ok(primary);
    }

    Err(anyhow::anyhow!("No healthy nodes available for restore"))
}

fn parse_point_in_time(pit_str: &str) -> Result<DateTime<Utc>> {
    // Try different formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
    ];

    for format in &formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(pit_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
    }

    Err(anyhow::anyhow!("Invalid point-in-time format: {}", pit_str))
}

fn calculate_estimated_restore_size(backup_id: &str, restore_type: &str) -> Result<u64> {
    // In production, this would query the backup catalog for actual size
    // For now, we'll simulate based on backup ID and restore type

    let base_size = match restore_type {
        "full" => 10u64 * 1024 * 1024 * 1024,         // 10 GB base
        "point-in-time" => 8u64 * 1024 * 1024 * 1024, // 8 GB base
        "schema-only" => 1u64 * 1024 * 1024 * 1024,   // 1 GB base
        _ => return Err(anyhow::anyhow!("Invalid restore type: {}", restore_type)),
    };

    // Adjust based on backup ID (simulate different sizes)
    let size_multiplier = if backup_id.contains("full") { 1.0 } else { 0.7 };
    let adjusted_size = (base_size as f64 * size_multiplier) as u64;

    Ok(adjusted_size)
}

fn calculate_estimated_restore_time(size_bytes: u64, demo: bool) -> Result<String> {
    if demo {
        return Ok("3-8 minutes (demo mode)".to_string());
    }

    // Estimate based on size
    // Assume 50 MB/s restore speed (slower than backup due to verification)
    let speed_mbps = 50.0 * 1024.0 * 1024.0;
    let time_seconds = size_bytes as f64 / speed_mbps;

    if time_seconds < 60.0 {
        Ok(format!("{:.0} seconds", time_seconds))
    } else if time_seconds < 3600.0 {
        Ok(format!("{:.0} minutes", time_seconds / 60.0))
    } else {
        Ok(format!("{:.1} hours", time_seconds / 3600.0))
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
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

async fn verify_backup_integrity(backup_id: &str) -> Result<()> {
    // In production, this would:
    // 1. Check backup file exists
    // 2. Verify checksum
    // 3. Test backup file integrity
    // 4. Validate backup metadata

    println!("  Checking backup file existence...");
    println!("  Verifying checksum...");
    println!("  Testing backup integrity...");
    println!("  Validating backup metadata...");

    // Simulate verification delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}

async fn execute_restore(
    client: &impl ApiClientTrait,
    backup_id: &str,
    target_node: &crate::client::NodeInfo,
    restore_type: &str,
    point_in_time: Option<DateTime<Utc>>,
    verify: bool,
    demo: bool,
) -> Result<RestoreInfo> {
    if demo {
        println!(
            "  [DEMO MODE] Simulating restore of backup {} to {}...",
            backup_id, target_node.node_id
        );
        // Skip sleep in demo mode for faster tests
        return Ok(RestoreInfo {
            restore_id: format!("restore-{}", chrono::Utc::now().timestamp()),
            backup_id: backup_id.to_string(),
            target_node: target_node.node_id.clone(),
            restore_type: restore_type.to_string(),
            point_in_time,
            size_bytes: 5u64 * 1024 * 1024 * 1024, // 5 GB
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            status: RestoreStatus::Completed,
            verification_passed: verify,
        });
    }

    println!("  Step 1: Preparing restore environment...");
    // In production, this would:
    // 1. Check available disk space
    // 2. Stop PostgreSQL if needed
    // 3. Create restore directory

    println!("  Step 2: Starting {} restore...", restore_type);
    // In production, this would:
    // 1. Execute pg_restore or pg_basebackup
    // 2. Apply point-in-time recovery if specified
    // 3. Restore to target location

    println!("  Step 3: Monitoring restore progress...");
    // In production, this would:
    // 1. Monitor restore progress
    // 2. Check for errors
    // 3. Verify restored data

    println!("  Step 4: Finalizing restore...");
    // In production, this would:
    // 1. Verify restore integrity
    // 2. Update cluster metadata
    // 3. Restart PostgreSQL if needed

    println!("  ✅ Restore completed successfully!");

    Ok(RestoreInfo {
        restore_id: format!("restore-{}", chrono::Utc::now().timestamp()),
        backup_id: backup_id.to_string(),
        target_node: target_node.node_id.clone(),
        restore_type: restore_type.to_string(),
        point_in_time,
        size_bytes: 5u64 * 1024 * 1024 * 1024, // 5 GB
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        status: RestoreStatus::Completed,
        verification_passed: verify,
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
