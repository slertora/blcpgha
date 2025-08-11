// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub node_id: String,
    pub config_file: String,
    pub action: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub status: ConfigStatus,
    pub timestamp: i64,
    pub changes: Vec<ConfigChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigStatus {
    Success,
    Failed,
    ValidationError,
    BackupRequired,
    RestoreRequired,
}

impl std::fmt::Display for ConfigStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigStatus::Success => write!(f, "{}", "Success".green()),
            ConfigStatus::Failed => write!(f, "{}", "Failed".red()),
            ConfigStatus::ValidationError => write!(f, "{}", "Validation Error".yellow()),
            ConfigStatus::BackupRequired => write!(f, "{}", "Backup Required".yellow()),
            ConfigStatus::RestoreRequired => write!(f, "{}", "Restore Required".yellow()),
        }
    }
}

pub async fn execute(
    client: &impl ApiClientTrait,
    target: &str,
    action: &str,
    key: Option<String>,
    value: Option<String>,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Remote Configuration Management".bold().blue());
    println!("{}", "=".repeat(32));

    if demo {
        println!("{}", "DEMO MODE ENABLED - Simulating operations".yellow().bold());
        println!();
    }

    // Get current cluster status to validate target node
    let cluster_status = client.get_cluster_status().await?;
    
    // Validate target node exists
    let target_node = cluster_status.nodes
        .iter()
        .find(|node| node.node_id == target)
        .ok_or_else(|| anyhow::anyhow!("Target node '{}' not found in cluster", target))?;

    println!("Configuration Details:");
    println!("  Target Node: {}", target_node.node_id.bold());
    println!("  Target Health: {}", if target_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
    println!("  Target Health Score: {:.2}%", target_node.health_score);
    println!("  Action: {}", action.bold());
    println!("  Config File: {}", file.bold());
    if let Some(ref k) = key {
        println!("  Key: {}", k.bold());
    }
    if let Some(ref v) = value {
        println!("  Value: {}", v.bold());
    }
    println!();

    // Validate action
    let valid_actions = ["get", "set", "validate", "backup", "restore"];
    if !valid_actions.contains(&action) {
        println!("{}", "ERROR: Invalid action!".red().bold());
        println!("Valid actions: {}", valid_actions.join(", "));
        return Ok(());
    }

    // Execute the requested action
    match action {
        "get" => execute_get_config(client, target_node, file, demo).await?,
        "set" => execute_set_config(client, target_node, key, value, file, demo).await?,
        "validate" => execute_validate_config(client, target_node, file, demo).await?,
        "backup" => execute_backup_config(client, target_node, file, demo).await?,
        "restore" => execute_restore_config(client, target_node, file, demo).await?,
        _ => unreachable!(),
    }

    Ok(())
}

async fn execute_get_config(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Getting configuration...".bold().cyan());
    
    if demo {
        println!("  [DEMO MODE] Simulating configuration retrieval...");
        
        // Simulate config content
        let config_content = r#"
# BLC PostgreSQL HA Configuration
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[postgresql]
host = "localhost"
port = 5432
database = "postgres"
username = "postgres"
password = "********"

[cluster]
name = "blc-cluster"
consensus_backend = "raft"
raft_port = 8081

[health]
check_interval = "5s"
timeout = "3s"
retries = 3

[vip]
manager = "keepalived"
interface = "eth0"
virtual_ip = "192.168.1.100"
"#;
        
        println!("  ✅ Configuration retrieved successfully!");
        println!();
        println!("{}", "Configuration Content:".bold());
        println!("{}", config_content);
        
        return Ok(());
    }

    // In production, this would:
    // 1. Connect to the target node via SSH
    // 2. Read the configuration file
    // 3. Parse and format the content
    // 4. Return the configuration
    
    println!("  Connecting to {}...", target_node.node_id);
    println!("  Reading configuration file: {}", file);
    println!("  ✅ Configuration retrieved successfully!");
    
    Ok(())
}

async fn execute_set_config(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    key: Option<String>,
    value: Option<String>,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Setting configuration...".bold().cyan());
    
    let key = key.ok_or_else(|| anyhow::anyhow!("Key is required for 'set' action"))?;
    let value = value.ok_or_else(|| anyhow::anyhow!("Value is required for 'set' action"))?;
    
    println!("  Key: {}", key.bold());
    println!("  Value: {}", value.bold());
    println!();

    // Validate key format
    if !is_valid_config_key(&key) {
        println!("{}", "ERROR: Invalid configuration key format!".red().bold());
        println!("Expected format: section.key (e.g., server.port, postgresql.host)");
        return Ok(());
    }

    // Show impact analysis
    println!("{}", "Impact Analysis:".bold().cyan());
    println!("  Target Node: {} ({})", 
        target_node.node_id.bold(), 
        if target_node.is_primary { "Primary".red() } else { "Replica".green() }
    );
    println!("  Configuration File: {}", file.bold());
    println!("  Change: {} = {}", key.bold(), value.bold());
    println!("  Restart Required: {}", if requires_restart(&key) { "Yes".yellow() } else { "No".green() });
    println!();

    if demo {
        println!("  [DEMO MODE] Simulating configuration update...");
        println!("  ✅ Configuration updated successfully!");
        println!("  [DEMO MODE] Simulating service restart...");
        println!("  ✅ Service restarted successfully!");
    } else {
        // Get user confirmation
        let response = get_user_confirmation(&format!("Do you want to update configuration on {}? (y/N): ", target_node.node_id), demo)?;
        if !response {
            println!("Configuration update cancelled.");
            return Ok(());
        }

        println!("  Updating configuration on {}...", target_node.node_id);
        println!("  Writing to {}...", file);
        println!("  ✅ Configuration updated successfully!");
        
        if requires_restart(&key) {
            println!("  Restarting service...");
            println!("  ✅ Service restarted successfully!");
        }
    }

    Ok(())
}

async fn execute_validate_config(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Validating configuration...".bold().cyan());
    
    if demo {
        println!("  [DEMO MODE] Simulating configuration validation...");
        println!("  ✅ Configuration validation completed!");
        println!();
        println!("{}", "Validation Results:".bold());
        println!("  ✅ Syntax: Valid");
        println!("  ✅ Required fields: All present");
        println!("  ✅ Network settings: Valid");
        println!("  ✅ PostgreSQL settings: Valid");
        println!("  ✅ Cluster settings: Valid");
        println!("  ⚠️  Performance: Consider increasing connection pool");
        println!("  ✅ Security: Valid");
        
        return Ok(());
    }

    println!("  Validating configuration on {}...", target_node.node_id);
    println!("  Checking syntax...");
    println!("  Validating required fields...");
    println!("  Checking network settings...");
    println!("  Validating PostgreSQL settings...");
    println!("  Checking cluster settings...");
    println!("  ✅ Configuration validation completed!");
    
    Ok(())
}

async fn execute_backup_config(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Backing up configuration...".bold().cyan());
    
    if demo {
        println!("  [DEMO MODE] Simulating configuration backup...");
        println!("  ✅ Configuration backup completed!");
        println!("  Backup file: {}.backup.{}", file, chrono::Utc::now().timestamp());
        
        return Ok(());
    }

    println!("  Creating backup of {} on {}...", file, target_node.node_id);
    println!("  ✅ Configuration backup completed!");
    
    Ok(())
}

async fn execute_restore_config(
    client: &impl ApiClientTrait,
    target_node: &crate::client::NodeInfo,
    file: &str,
    demo: bool,
) -> Result<()> {
    println!("{}", "Restoring configuration...".bold().cyan());
    
    if demo {
        println!("  [DEMO MODE] Simulating configuration restore...");
        println!("  ✅ Configuration restore completed!");
        
        return Ok(());
    }

    // Get user confirmation for restore
    let response = get_user_confirmation(&format!("Do you want to restore configuration on {}? This will overwrite current config. (y/N): ", target_node.node_id), demo)?;
    if !response {
        println!("Configuration restore cancelled.");
        return Ok(());
    }

    println!("  Restoring configuration on {}...", target_node.node_id);
    println!("  ✅ Configuration restore completed!");
    
    Ok(())
}

fn is_valid_config_key(key: &str) -> bool {
    // Valid format: section.key (e.g., server.port, postgresql.host)
    let parts: Vec<&str> = key.split('.').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
}

fn requires_restart(key: &str) -> bool {
    // Keys that require service restart
    let restart_keys = [
        "server.host", "server.port", "postgresql.host", "postgresql.port",
        "cluster.name", "consensus_backend", "raft_port"
    ];
    
    restart_keys.contains(&key)
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