// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(_client: &impl ApiClientTrait, detailed: bool) -> Result<()> {
    let status = _client.get_status().await?;

    if detailed {
        println!("{}", "VIP Status (Detailed)".bold().blue());
        println!("{}", "=".repeat(40));
        println!("VIP Management: {}", if status.health.is_primary { "ACTIVE".green().bold() } else { "INACTIVE".yellow() });
        println!("Current Primary: {}", if status.health.is_primary { "This node".green() } else { "None".yellow() });
        println!("Health Status: {}", if status.health.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
        println!("Health Score: {:.2}%", status.health.score);
        println!();
        
        println!("{}", "VIP Configuration:".bold().cyan());
        println!("  Type: {}", "HAProxy/ProxySQL/Keepalived/AWS".yellow());
        println!("  Fallback Enabled: {}", "Yes".green());
        println!("  Fallback Order: {}", "haproxy, proxysql, keepalived, aws_elastic_ip, aws_elb".yellow());
        println!();
        
        println!("{}", "Current Status:".bold().cyan());
        println!("  VIP Active: {}", if status.health.is_primary { "YES".green() } else { "NO".yellow() });
        println!("  Primary Node: {}", if status.health.is_primary { "This node".green() } else { "None".yellow() });
        println!("  Last Update: {}", "N/A".yellow());
        println!();
        
        println!("{}", "Health Check:".bold().cyan());
        println!("  PostgreSQL Primary: {}", if status.health.is_primary { "YES".green() } else { "NO".yellow() });
        println!("  Raft Leader: {}", if status.raft.is_leader { "YES".green() } else { "NO".yellow() });
        println!("  Overall Health: {}", if status.health.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
    } else {
        println!("{}", "VIP Status".bold().blue());
        println!("{}", "=".repeat(20));
        println!("Status: {}", if status.health.is_primary { "ACTIVE".green() } else { "INACTIVE".yellow() });
        println!("Primary: {}", if status.health.is_primary { "This node".green() } else { "None".yellow() });
        println!("Health: {}", if status.health.is_healthy { "Healthy".green() } else { "Unhealthy".red() });
        println!("Score: {:.2}%", status.health.score);
    }

    Ok(())
} 