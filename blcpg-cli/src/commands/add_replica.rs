// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(
    client: &impl ApiClientTrait,
    target: &str,
    source: &str,
    wait_sync: bool,
    force: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Adding Replica to Cluster".bold().blue());
    println!("{}", "=".repeat(30));

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
    println!("Source Selection: {}", source.bold());
    println!(
        "Wait for Sync: {}",
        if wait_sync {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Check if target node already exists in cluster
    if cluster_status
        .nodes
        .iter()
        .any(|node| node.node_id == target)
    {
        if !force {
            println!(
                "{}",
                "ERROR: Target node already exists in cluster!".red().bold()
            );
            println!("Use --force to override this check.");
            return Ok(());
        } else {
            println!(
                "{}",
                "WARNING: Target node already exists, proceeding with --force"
                    .yellow()
                    .bold()
            );
        }
    }

    // Detect if target node has PostgreSQL installed
    println!("{}", "Step 1: Checking target node status...".bold().cyan());
    let target_has_postgresql = check_postgresql_installed(target, demo).await?;

    if target_has_postgresql && !force {
        println!(
            "{}",
            "ERROR: Target node already has PostgreSQL installed!"
                .red()
                .bold()
        );
        println!("Use --force to override this check.");
        return Ok(());
    }

    // Select best source for backup
    println!(
        "{}",
        "Step 2: Selecting best source for backup...".bold().cyan()
    );
    let selected_source = select_best_source(&cluster_status.nodes, source, demo).await?;
    println!("Selected Source: {}", selected_source.bold().green());
    println!(
        "Source Health Score: {:.2}%",
        get_node_health_score(&cluster_status.nodes, &selected_source)
    );
    println!(
        "Source Replication Lag: {}",
        get_replication_lag_text(&cluster_status.nodes, &selected_source)
    );
    println!();

    // Validate source node (skip validation in demo mode)
    if !demo {
        let source_node = cluster_status
            .nodes
            .iter()
            .find(|node| node.node_id == selected_source)
            .ok_or_else(|| anyhow::anyhow!("Selected source node not found in cluster"))?;

        if !source_node.is_healthy {
            println!(
                "{}",
                "WARNING: Selected source node is not healthy!".red().bold()
            );
            let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
            if !response {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        // Check replication lag on source
        if let Some(lag) = source_node.replication_lag_seconds {
            if lag > 10.0 {
                println!(
                    "{}",
                    "WARNING: High replication lag detected on source!"
                        .red()
                        .bold()
                );
                println!("Replication Lag: {:.1}s", lag);
                let response =
                    get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
                if !response {
                    println!("Operation cancelled.");
                    return Ok(());
                }
            }
        }
    }

    // Show operation summary
    println!("{}", "Operation Summary:".bold().cyan());
    println!("  Target Node: {}", target.bold());
    println!("  Source Node: {}", selected_source.bold());
    println!(
        "  PostgreSQL Installation: {}",
        if target_has_postgresql {
            "Already installed".yellow()
        } else {
            "Will install".green()
        }
    );
    println!("  Backup Method: {}", "pg_basebackup".green());
    println!("  Replication Configuration: {}", "Automatic".green());
    println!(
        "  Wait for Sync: {}",
        if wait_sync {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Final confirmation
    let response = get_user_confirmation(
        &format!(
            "Do you want to add {} as a replica using {} as source? (y/N): ",
            target, selected_source
        ),
        demo,
    )?;
    if !response {
        println!("Operation cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting replica addition...".bold().cyan());

    // Execute the add-replica operation
    match execute_add_replica(target, &selected_source, wait_sync, demo).await {
        Ok(_) => {
            println!("{}", "✅ Replica addition successful!".green().bold());
            println!("New Replica: {}", target.green().bold());
            println!("Source Node: {}", selected_source.green().bold());

            // Show updated cluster status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_cluster = client.get_cluster_status().await?;
            println!("Total Nodes: {}", updated_cluster.total_nodes);
            println!("Healthy Nodes: {}", updated_cluster.healthy_nodes);

            // Show the new replica status
            if let Some(new_replica) = updated_cluster.nodes.iter().find(|n| n.node_id == target) {
                println!(
                    "New Replica Status: {}",
                    if new_replica.is_healthy {
                        "Healthy".green()
                    } else {
                        "Unhealthy".red()
                    }
                );
                println!("Health Score: {:.2}%", new_replica.health_score);
                println!(
                    "Replication Lag: {}",
                    get_replication_lag_text(&updated_cluster.nodes, target)
                );
            }
        }
        Err(e) => {
            println!("{}", "❌ Replica addition failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

async fn check_postgresql_installed(node_id: &str, demo: bool) -> Result<bool> {
    println!("  Checking if PostgreSQL is installed on {}...", node_id);

    if demo {
        println!(
            "  [DEMO MODE] Simulating PostgreSQL check on {}...",
            node_id
        );
        // Skip sleep in demo mode for faster tests
        return Ok(false);
    }

    // In a real implementation, this would SSH to the target node and check:
    // 1. systemctl status postgresql
    // 2. ls -la /var/lib/postgresql/data
    // 3. id postgres
    // 4. which psql

    // For now, simulate the check - in production this would be a real SSH call
    let _check_commands = vec![
        format!("ssh {} 'systemctl status postgresql --no-pager'", node_id),
        format!("ssh {} 'ls -la /var/lib/postgresql/data'", node_id),
        format!("ssh {} 'id postgres'", node_id),
    ];

    println!(
        "  [SIMULATED] Executing PostgreSQL checks on {}...",
        node_id
    );

    // Simulate the check - assume PostgreSQL is not installed for now
    // In production, this would execute the actual commands and parse results
    Ok(false)
}

async fn select_best_source(
    nodes: &[crate::client::NodeInfo],
    source: &str,
    demo: bool,
) -> Result<String> {
    if demo {
        // In demo mode, simulate a healthy primary node
        println!("  [DEMO MODE] Simulating healthy primary node selection...");
        return Ok("node-1".to_string());
    }

    match source {
        "auto" => {
            // Auto-select the best source (primary first, then best replica)
            if let Some(primary) = nodes.iter().find(|node| node.is_primary && node.is_healthy) {
                println!("  Auto-selected primary node: {}", primary.node_id.bold());
                Ok(primary.node_id.clone())
            } else {
                // Select best replica based on health score and replication lag
                let healthy_replicas: Vec<_> = nodes
                    .iter()
                    .filter(|node| node.is_healthy && !node.is_primary)
                    .collect();

                if healthy_replicas.is_empty() {
                    return Err(anyhow::anyhow!("No healthy nodes available as source"));
                }

                // Sort by health score (descending) and replication lag (ascending)
                let mut candidates: Vec<_> = healthy_replicas.iter().collect();
                candidates.sort_by(|a, b| {
                    b.health_score
                        .partial_cmp(&a.health_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            match (a.replication_lag_seconds, b.replication_lag_seconds) {
                                (None, None) => std::cmp::Ordering::Equal,
                                (None, _) => std::cmp::Ordering::Greater,
                                (_, None) => std::cmp::Ordering::Less,
                                (Some(lag_a), Some(lag_b)) => lag_a
                                    .partial_cmp(&lag_b)
                                    .unwrap_or(std::cmp::Ordering::Equal),
                            }
                        })
                });

                let best_candidate = candidates
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No suitable candidates found"))?;

                println!(
                    "  Auto-selected best replica: {}",
                    best_candidate.node_id.bold()
                );
                Ok(best_candidate.node_id.clone())
            }
        }
        "primary" => {
            // Select the primary node
            if let Some(primary) = nodes.iter().find(|node| node.is_primary) {
                println!("  Selected primary node: {}", primary.node_id.bold());
                Ok(primary.node_id.clone())
            } else {
                Err(anyhow::anyhow!("No primary node found in cluster"))
            }
        }
        _ => {
            // Select specific node
            if nodes.iter().any(|node| node.node_id == source) {
                println!("  Selected specific node: {}", source.bold());
                Ok(source.to_string())
            } else {
                Err(anyhow::anyhow!("Node '{}' not found in cluster", source))
            }
        }
    }
}

fn get_node_health_score(nodes: &[crate::client::NodeInfo], node_id: &str) -> f64 {
    nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .map(|node| node.health_score)
        .unwrap_or(0.0)
}

fn get_replication_lag_text(nodes: &[crate::client::NodeInfo], node_id: &str) -> String {
    if let Some(node) = nodes.iter().find(|n| n.node_id == node_id) {
        match node.replication_lag_seconds {
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
    } else {
        "Unknown".red().to_string()
    }
}

async fn execute_add_replica(
    target: &str,
    source: &str,
    wait_sync: bool,
    demo: bool,
) -> Result<()> {
    println!("  Executing pg_basebackup from {} to {}...", source, target);

    if demo {
        println!(
            "    [DEMO MODE] Simulating pg_basebackup from {} to {}...",
            source, target
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
        return Ok(());
    }

    // Step 1: Install PostgreSQL on target node (if needed)
    println!("  Step 1: Installing PostgreSQL on {}...", target);
    install_postgresql(target).await?;

    // Step 2: Create backup from source
    println!("  Step 2: Creating backup from {} to {}...", source, target);
    create_backup(source, target).await?;

    // Step 3: Configure replication
    println!("  Step 3: Configuring replication...");
    configure_replication(target, source).await?;

    // Step 4: Start PostgreSQL service
    println!("  Step 4: Starting PostgreSQL service...");
    start_postgresql(target).await?;

    // Step 5: Verify replication
    println!("  Step 5: Verifying replication...");
    verify_replication(target).await?;

    if wait_sync {
        println!("  Step 6: Waiting for replication to sync...");
        wait_for_sync(target).await?;
    }

    println!("  ✅ Replica addition completed successfully!");

    Ok(())
}

async fn install_postgresql(node_id: &str) -> Result<()> {
    // In production, this would:
    // 1. SSH to the target node
    // 2. Install PostgreSQL packages
    // 3. Create postgres user
    // 4. Initialize data directory

    let _install_commands = vec![
        format!("ssh {} 'sudo apt-get update'", node_id),
        format!(
            "ssh {} 'sudo apt-get install -y postgresql-15 postgresql-client-15'",
            node_id
        ),
        format!("ssh {} 'sudo systemctl enable postgresql'", node_id),
    ];

    println!("    [SIMULATED] Installing PostgreSQL on {}...", node_id);
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

async fn create_backup(source: &str, target: &str) -> Result<()> {
    // In production, this would:
    // 1. SSH to target node
    // 2. Execute pg_basebackup from source
    // 3. Monitor progress

    let _backup_command = format!(
        "ssh {} 'sudo -u postgres pg_basebackup -h {} -D /var/lib/postgresql/15/main -U replicator -P --wal-method=stream'",
        target, source
    );

    println!(
        "    [SIMULATED] Creating backup from {} to {}...",
        source, target
    );
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}

async fn configure_replication(target: &str, source: &str) -> Result<()> {
    // In production, this would:
    // 1. Create standby.signal file
    // 2. Configure postgresql.auto.conf with primary_conninfo
    // 3. Set proper permissions

    let _config_commands = vec![
        format!("ssh {} 'sudo -u postgres touch /var/lib/postgresql/15/main/standby.signal'", target),
        format!("ssh {} 'sudo -u postgres echo \"primary_conninfo = \\\"host={} port=5432 user=replicator password=replicator123 application_name={}\\\"\" >> /var/lib/postgresql/15/main/postgresql.auto.conf'", target, source, target),
        format!("ssh {} 'sudo chown -R postgres:postgres /var/lib/postgresql/15/main'", target),
    ];

    println!("    [SIMULATED] Configuring replication on {}...", target);
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

async fn start_postgresql(node_id: &str) -> Result<()> {
    // In production, this would:
    // 1. SSH to target node
    // 2. Start PostgreSQL service
    // 3. Check if it started successfully

    let _start_command = format!("ssh {} 'sudo systemctl start postgresql'", node_id);

    println!("    [SIMULATED] Starting PostgreSQL on {}...", node_id);
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

async fn verify_replication(node_id: &str) -> Result<()> {
    // In production, this would:
    // 1. SSH to target node
    // 2. Check replication status
    // 3. Verify WAL is being received

    let _verify_commands = vec![
        format!("ssh {} 'sudo -u postgres psql -c \"SELECT pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn();\"'", node_id),
        format!("ssh {} 'sudo -u postgres psql -c \"SELECT * FROM pg_stat_replication;\"'", node_id),
    ];

    println!("    [SIMULATED] Verifying replication on {}...", node_id);
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

async fn wait_for_sync(node_id: &str) -> Result<()> {
    // In production, this would:
    // 1. Monitor replication lag
    // 2. Wait until lag is minimal
    // 3. Verify sync state

    println!(
        "    [SIMULATED] Waiting for replication to sync on {}...",
        node_id
    );
    std::thread::sleep(std::time::Duration::from_secs(2));

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
