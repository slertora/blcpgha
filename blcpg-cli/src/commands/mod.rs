// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

pub mod add_replica;
pub mod backup;
pub mod cluster_expand;
pub mod cluster_info;
pub mod config;
pub mod demote;
pub mod failover;
pub mod health;
pub mod maintenance;
pub mod metrics;
pub mod node_info;
pub mod nodes;
pub mod promote;
pub mod replication_status;
pub mod restore;
pub mod status;
pub mod switchover;
pub mod upgrade;
pub mod vip;

#[cfg(test)]
pub mod tests;
