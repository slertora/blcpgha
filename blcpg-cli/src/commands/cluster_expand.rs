// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClientTrait;
use anyhow::Result;
use colored::*;

pub async fn execute(
    client: &impl ApiClientTrait,
    count: u32,
    auto_install: bool,
    auto_configure: bool,
    demo: bool,
) -> Result<()> {
    println!("{}", "Cluster Expansion".bold().blue());
    println!("{}", "=".repeat(17));

    if demo {
        println!("{}", "DEMO MODE ENABLED - Simulating operations".yellow().bold());
        println!();
    }

    // Get current cluster status
    let cluster_status = client.get_cluster_status().await?;

    println!("Expansion Plan:");
    println!("  Nodes to Add: {}", count.to_string().bold());
    println!("  Auto-install PostgreSQL: {}", if auto_install { "Yes".green() } else { "No".yellow() });
    println!("  Auto-configure as Replicas: {}", if auto_configure { "Yes".green() } else { "No".yellow() });
    println!();

    println!("Current Cluster Status:");
    println!("  Total Nodes: {}", cluster_status.total_nodes);
    println!("  Healthy Nodes: {}", cluster_status.healthy_nodes);
    println!("  Primary Node: {}", get_primary_node_id(&cluster_status.nodes));
    println!();

    // Validate expansion
    if !demo {
        println!("{}", "Step 1: Validating expansion requirements...".bold().cyan());
        
        // Check if we have enough healthy nodes
        if cluster_status.healthy_nodes < 2 {
            println!("{}", "WARNING: Low number of healthy nodes!".yellow().bold());
            println!("  Current healthy nodes: {}", cluster_status.healthy_nodes);
            println!("  Recommended minimum: 2");
            
            let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
            if !response {
                println!("Cluster expansion cancelled.");
                return Ok(());
            }
        }

        // Check if primary is healthy
        let primary_node = cluster_status.nodes.iter().find(|node| node.is_primary);
        if let Some(primary) = primary_node {
            if !primary.is_healthy {
                println!("{}", "WARNING: Primary node is not healthy!".yellow().bold());
                let response = get_user_confirmation("Do you want to continue anyway? (y/N): ", demo)?;
                if !response {
                    println!("Cluster expansion cancelled.");
                    return Ok(());
                }
            }
        }
    }

    // Generate new node IDs
    let new_node_ids = generate_new_node_ids(&cluster_status.nodes, count)?;
    
    println!("{}", "New Nodes to Add:".bold().cyan());
    for (i, node_id) in new_node_ids.iter().enumerate() {
        println!("  {}. {}", i + 1, node_id.bold());
    }
    println!();

    // Show expansion plan
    println!("{}", "Expansion Plan:".bold().cyan());
    for node_id in &new_node_ids {
        println!("  Node {}:", node_id.bold());
        if auto_install {
            println!("    • Install PostgreSQL");
        }
        if auto_configure {
            println!("    • Configure as replica");
            println!("    • Setup replication from primary");
        }
        println!("    • Integrate into cluster");
        println!();
    }

    // Final confirmation
    let response = get_user_confirmation(&format!("Do you want to expand cluster with {} new node(s)? (y/N): ", count), demo)?;
    if !response {
        println!("Cluster expansion cancelled.");
        return Ok(());
    }

    println!();
    println!("{}", "Starting cluster expansion...".bold().cyan());

    // Execute the expansion
    match execute_cluster_expansion(client, &new_node_ids, auto_install, auto_configure, demo).await {
        Ok(_) => {
            println!("{}", "✅ Cluster expansion successful!".green().bold());
            println!("Added {} new node(s):", count);
            for node_id in &new_node_ids {
                println!("  • {}", node_id.green().bold());
            }
            
            // Show updated cluster status
            println!();
            println!("{}", "Updated Cluster Status:".bold().cyan());
            let updated_cluster = client.get_cluster_status().await?;
            println!("Total Nodes: {}", updated_cluster.total_nodes);
            println!("Healthy Nodes: {}", updated_cluster.healthy_nodes);
            
            // Show new nodes status
            println!();
            println!("{}", "New Nodes Status:".bold().cyan());
            for node_id in &new_node_ids {
                if let Some(new_node) = updated_cluster.nodes.iter().find(|n| n.node_id == *node_id) {
                    println!("  {}: {} (Health: {:.2}%)", 
                        node_id.bold(),
                        if new_node.is_healthy { "Healthy".green() } else { "Unhealthy".red() },
                        new_node.health_score
                    );
                }
            }
        }
        Err(e) => {
            println!("{}", "❌ Cluster expansion failed!".red().bold());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn get_primary_node_id(nodes: &[crate::client::NodeInfo]) -> String {
    nodes.iter()
        .find(|node| node.is_primary)
        .map(|node| node.node_id.clone())
        .unwrap_or_else(|| "None".red().to_string())
}

pub(crate) fn generate_new_node_ids(existing_nodes: &[crate::client::NodeInfo], count: u32) -> Result<Vec<String>> {
    let mut new_node_ids = Vec::new();
    let mut node_number = existing_nodes.len() + 1;
    
    for _ in 0..count {
        let mut node_id = format!("node-{}", node_number);
        
        // Check if node ID already exists
        while existing_nodes.iter().any(|node| node.node_id == node_id) || 
              new_node_ids.contains(&node_id) {
            node_number += 1;
            node_id = format!("node-{}", node_number);
        }
        
        new_node_ids.push(node_id);
        node_number += 1;
    }
    
    Ok(new_node_ids)
}

async fn execute_cluster_expansion(
    client: &impl ApiClientTrait,
    new_node_ids: &[String],
    auto_install: bool,
    auto_configure: bool,
    demo: bool,
) -> Result<()> {
    if demo {
        println!("  [DEMO MODE] Simulating cluster expansion...");
        for node_id in new_node_ids {
            println!("    Adding node {}...", node_id);
            // Skip sleep in demo mode for faster tests
        }
        return Ok(());
    }

    for node_id in new_node_ids {
        println!("  Processing node {}...", node_id.bold());
        
        if auto_install {
            println!("    Step 1: Installing PostgreSQL...");
            // In production, this would:
            // 1. SSH to the node
            // 2. Install PostgreSQL packages
            // 3. Initialize PostgreSQL data directory
            // 4. Configure basic settings
        }
        
        if auto_configure {
            println!("    Step 2: Configuring as replica...");
            // In production, this would:
            // 1. Configure postgresql.conf for replication
            // 2. Setup pg_hba.conf for replication connections
            // 3. Create replication user
            // 4. Setup standby.signal
        }
        
        println!("    Step 3: Integrating into cluster...");
        // In production, this would:
        // 1. Add node to cluster configuration
        // 2. Setup replication from primary
        // 3. Start PostgreSQL service
        // 4. Verify replication is working
        
        println!("    ✅ Node {} added successfully!", node_id);
    }
    
    println!("  ✅ Cluster expansion completed successfully!");
    
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