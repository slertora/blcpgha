// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{
    routing::{get, post},
    Router,
};

pub mod cluster;
pub mod cluster_operations;
pub mod health;
pub mod metrics;
pub mod nodes;
pub mod postgresql_operations;
pub mod vip;

pub fn create_api_router() -> Router {
    Router::new()
        .route("/cluster/status", get(cluster::get_status))
        .route("/nodes", get(nodes::list_nodes))
        .route("/nodes/:node_id", get(nodes::get_node))
        .route("/nodes/:node_id/promote", post(nodes::promote_node))
        .route("/nodes/:node_id/demote", post(nodes::demote_node))
        .route("/nodes/:node_id/disable", post(nodes::disable_node))
        .route("/nodes/:node_id/enable", post(nodes::enable_node))
        .route("/metrics", get(metrics::get_metrics))
        .route("/health", get(health::get_health))
        .route("/vip/status", get(vip::get_status))
        // Nuevas rutas para operaciones del cluster
        .route("/cluster/failover", post(cluster_operations::failover))
        .route(
            "/cluster/add-replica",
            post(cluster_operations::add_replica),
        )
        .route("/cluster/expand", post(cluster_operations::cluster_expand))
        .route("/cluster/backup", post(cluster_operations::create_backup))
        // Nuevas rutas para operaciones avanzadas de PostgreSQL
        .route("/cluster/restore", post(postgresql_operations::restore))
        .route(
            "/cluster/maintenance",
            post(postgresql_operations::maintenance),
        )
        .route("/cluster/upgrade", post(postgresql_operations::upgrade))
}
