// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

pub mod cluster_info;
pub mod demote;
pub mod health;
pub mod metrics;
pub mod node_info;
pub mod nodes;
pub mod promote;
pub mod status;
pub mod vip;
pub mod add_replica;
pub mod switchover;
pub mod failover;
pub mod replication_status;
pub mod cluster_expand;
pub mod backup;
pub mod restore;
pub mod maintenance;
pub mod upgrade;
pub mod config;

#[cfg(test)]
pub mod tests; 