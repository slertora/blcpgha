// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(client: &impl ApiClientTrait, detailed: bool) -> Result<()> {
    let status = client.get_status().await?;

    if detailed {
        println!("{}", "Health Information (Detailed)".bold().blue());
        println!("{}", "=".repeat(40));
        println!(
            "Overall Health: {}",
            get_health_status_text(status.health.is_healthy)
        );
        println!("Health Score: {:.2}%", status.health.score);
        println!(
            "Is Primary: {}",
            if status.health.is_primary {
                "Yes".green()
            } else {
                "No".yellow()
            }
        );
        println!(
            "Replication Lag: {}",
            get_replication_lag_text(status.health.replication_lag_seconds)
        );
        println!(
            "Server Version: {}",
            status
                .health
                .server_version
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
        );
        println!(
            "Current LSN: {}",
            status
                .health
                .current_lsn
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
        );
        println!("Error Count: {}", status.health.error_count);
        println!(
            "Consecutive Failures: {}",
            status.health.consecutive_failures
        );
        println!("Last Check: {}", status.health.last_check);
    } else {
        println!("{}", "Health Information".bold().blue());
        println!("{}", "=".repeat(20));
        println!(
            "Status: {}",
            get_health_status_text(status.health.is_healthy)
        );
        println!("Score: {:.2}%", status.health.score);
        println!(
            "Primary: {}",
            if status.health.is_primary {
                "Yes".green()
            } else {
                "No".yellow()
            }
        );
        println!(
            "Replication Lag: {}",
            get_replication_lag_text(status.health.replication_lag_seconds)
        );
        println!("Last Check: {}", status.health.last_check);
    }

    Ok(())
}

fn get_health_status_text(is_healthy: bool) -> String {
    if is_healthy {
        "Healthy".green().to_string()
    } else {
        "Unhealthy".red().to_string()
    }
}

fn get_replication_lag_text(lag: Option<f64>) -> String {
    match lag {
        Some(lag_seconds) => {
            if lag_seconds < 1.0 {
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
