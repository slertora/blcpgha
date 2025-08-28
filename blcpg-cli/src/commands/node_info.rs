// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use crate::client::ApiClient;
use anyhow::Result;

pub async fn execute(_client: &ApiClient, node_id: &str) -> Result<()> {
    println!("Node info command for node: {}", node_id);
    // TODO: Implement node info
    Ok(())
}
