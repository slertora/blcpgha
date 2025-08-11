// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::ClusterConfig;
use crate::health::HealthStatus;
use crate::raft::RaftState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info};
use std::str::FromStr;
use reqwest;

// Proto definitions (simplified for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub address: String,
    pub health_status: Option<HealthStatus>,
    pub raft_state: Option<RaftState>,
    pub last_seen: i64,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub node_id: String,
    pub peers: HashMap<String, PeerInfo>,
    pub leader_id: Option<String>,
    pub healthy_replicas: Vec<String>,
    pub best_replica: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterManager {
    pub config: ClusterConfig,
    state: Arc<RwLock<ClusterState>>,
    shutdown_tx: mpsc::Sender<()>,
}

impl ClusterManager {
    pub fn new(config: ClusterConfig) -> Self {
        let state = Arc::new(RwLock::new(ClusterState {
            node_id: config.node_id.clone(),
            peers: HashMap::new(),
            leader_id: None,
            healthy_replicas: vec![],
            best_replica: None,
        }));

        let (shutdown_tx, _) = mpsc::channel::<()>(1);

        Self {
            config,
            state,
            shutdown_tx,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting cluster manager on port {}", self.config.cluster_port);
        
        // For now, just run the peer discovery loop
        self.run_peer_discovery().await?;
        
        Ok(())
    }

    async fn run_peer_discovery(&self) -> Result<()> {
        let interval_duration = humantime::Duration::from_str(&self.config.heartbeat_interval)
            .map_err(|e| anyhow::anyhow!("Invalid heartbeat interval: {}", e))?;
        
        let mut interval = interval(interval_duration.into());

        loop {
            interval.tick().await;
            
            if let Err(e) = self.discover_peers().await {
                error!("Peer discovery failed: {}", e);
            }
        }
    }

    async fn discover_peers(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Update peer list from config
        for peer_addr in &self.config.peers {
            let peer_id = self.extract_peer_id(peer_addr);
            
            if !state.peers.contains_key(&peer_id) {
                state.peers.insert(peer_id.clone(), PeerInfo {
                    node_id: peer_id,
                    address: peer_addr.clone(),
                    health_status: None,
                    raft_state: None,
                    last_seen: chrono::Utc::now().timestamp(),
                    is_online: false,
                });
            }
        }

        // Try to connect to each peer
        for (peer_id, peer_info) in &mut state.peers {
            if let Err(e) = self.ping_peer(peer_info).await {
                debug!("Failed to ping peer {}: {}", peer_id, e);
                peer_info.is_online = false;
            } else {
                peer_info.is_online = true;
                peer_info.last_seen = chrono::Utc::now().timestamp();
            }
        }

        // Update healthy replicas
        self.update_healthy_replicas(&mut state).await;
        
        // Find best replica
        self.find_best_replica(&mut state).await;

        debug!("Peer discovery completed. Online peers: {}", 
               state.peers.values().filter(|p| p.is_online).count());

        Ok(())
    }

    async fn ping_peer(&self, peer_info: &mut PeerInfo) -> Result<()> {
        debug!("Pinging peer: {} at {}", peer_info.node_id, peer_info.address);
        
        match self.ping_peer_http(&peer_info.address).await {
            Ok(alive) => {
                peer_info.is_online = alive;
                peer_info.last_seen = chrono::Utc::now().timestamp();
                if alive {
                    debug!("Peer {} is alive", peer_info.node_id);
                } else {
                    debug!("Peer {} is not responding", peer_info.node_id);
                }
            }
            Err(e) => {
                peer_info.is_online = false;
                error!("Failed to ping peer {}: {}", peer_info.node_id, e);
            }
        }
        
        Ok(())
    }

    async fn update_healthy_replicas(&self, state: &mut ClusterState) {
        state.healthy_replicas.clear();
        
        for (peer_id, peer_info) in &state.peers {
            if peer_info.is_online {
                if let Some(health) = &peer_info.health_status {
                    if health.is_healthy {
                        state.healthy_replicas.push(peer_id.clone());
                    }
                }
            }
        }
    }

    async fn find_best_replica(&self, state: &mut ClusterState) {
        let mut best_replica: Option<(String, u64)> = None;
        
        for (peer_id, peer_info) in &state.peers {
            if !peer_info.is_online {
                continue;
            }
            
            if let Some(health) = &peer_info.health_status {
                if !health.is_healthy {
                    continue;
                }
                
                // Prefer replicas with lower lag
                let lag = health.replication_lag.unwrap_or(u64::MAX);
                
                if let Some((_, current_lag)) = best_replica {
                    if lag < current_lag {
                        best_replica = Some((peer_id.clone(), lag));
                    }
                } else {
                    best_replica = Some((peer_id.clone(), lag));
                }
            }
        }
        
        state.best_replica = best_replica.map(|(peer_id, _)| peer_id);
    }

    fn extract_peer_id(&self, peer_addr: &str) -> String {
        // Extract peer ID from address (e.g., "127.0.0.1:9702" -> "node-9702")
        if let Some(port) = peer_addr.split(':').last() {
            format!("node-{}", port)
        } else {
            peer_addr.to_string()
        }
    }

    pub async fn get_state(&self) -> ClusterState {
        self.state.read().await.clone()
    }

    pub async fn update_peer_health(&self, peer_id: &str, health_status: HealthStatus) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(peer_info) = state.peers.get_mut(peer_id) {
            peer_info.health_status = Some(health_status);
            peer_info.last_seen = chrono::Utc::now().timestamp();
        }
        
        Ok(())
    }

    pub async fn update_peer_raft_state(&self, peer_id: &str, raft_state: RaftState) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(peer_info) = state.peers.get_mut(peer_id) {
            peer_info.raft_state = Some(raft_state);
            peer_info.last_seen = chrono::Utc::now().timestamp();
        }
        
        Ok(())
    }



    pub async fn get_healthy_replicas(&self) -> Vec<String> {
        let state = self.state.read().await;
        state.healthy_replicas.clone()
    }

    pub async fn shutdown(&self) -> Result<()> {
        let _ = self.shutdown_tx.send(()).await;
        Ok(())
    }

    // Broadcast health status to all peers
    pub async fn broadcast_health(&self, health_status: HealthStatus) -> Result<()> {
        let peers = {
            let state = self.state.read().await;
            state.peers.iter().map(|(id, info)| (id.clone(), info.address.clone())).collect::<Vec<_>>()
        };

        let peer_count = peers.len();
        for (peer_id, peer_addr) in &peers {
            if let Err(e) = self.broadcast_health_to_peer(peer_addr, &health_status).await {
                error!("Failed to broadcast health to peer {} at {}: {}", peer_id, peer_addr, e);
            }
        }

        info!("Broadcasted health status to {} peers", peer_count);
        Ok(())
    }

    // Broadcast Raft state to all peers
    pub async fn broadcast_raft_state(&self, raft_state: RaftState) -> Result<()> {
        let peers = {
            let state = self.state.read().await;
            state.peers.iter().map(|(id, info)| (id.clone(), info.address.clone())).collect::<Vec<_>>()
        };

        let peer_count = peers.len();
        for (peer_id, peer_addr) in &peers {
            if let Err(e) = self.broadcast_raft_to_peer(peer_addr, &raft_state).await {
                error!("Failed to broadcast Raft state to peer {} at {}: {}", peer_id, peer_addr, e);
            }
        }

        info!("Broadcasted Raft state to {} peers", peer_count);
        Ok(())
    }

    // Get best replica based on lag and health
    pub async fn get_best_replica(&self) -> Option<String> {
        let state = self.state.read().await;
        
        let mut best_replica = None;
        let mut best_score = f64::MAX;

        for (peer_id, peer_info) in &state.peers {
            if let Some(health) = &peer_info.health_status {
                if health.is_healthy && !health.is_primary {
                    let score = health.replication_lag.unwrap_or(u64::MAX) as f64;
                    if score < best_score {
                        best_score = score;
                        best_replica = Some(peer_id.clone());
                    }
                }
            }
        }

        best_replica
    }

    // HTTP communication methods
    async fn broadcast_health_to_peer(&self, peer_addr: &str, health_status: &HealthStatus) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/cluster/broadcast/health", peer_addr);
        
        let payload = serde_json::json!({
            "node_id": self.config.node_id,
            "health_status": health_status,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let response = client.post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}: {}", response.status(), response.text().await?));
        }

        Ok(())
    }

    async fn broadcast_raft_to_peer(&self, peer_addr: &str, raft_state: &RaftState) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/cluster/broadcast/raft", peer_addr);
        
        let payload = serde_json::json!({
            "node_id": self.config.node_id,
            "raft_state": raft_state,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let response = client.post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}: {}", response.status(), response.text().await?));
        }

        Ok(())
    }

    async fn ping_peer_http(&self, peer_addr: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/cluster/ping", peer_addr);
        
        let payload = serde_json::json!({
            "node_id": self.config.node_id,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let response = client.post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
} 