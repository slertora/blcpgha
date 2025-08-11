// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::RaftConfig;
use crate::cluster::ClusterManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftState {
    pub node_id: String,
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub role: RaftRole,
    pub commit_index: u64,
    pub last_applied: u64,
    pub peers: Vec<String>,
    pub leader_id: Option<String>,
    pub election_timeout: Duration,
    pub heartbeat_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct RaftNode {
    config: RaftConfig,
    state: Arc<RwLock<RaftState>>,
    logs: Arc<Mutex<HashMap<u64, RaftLogEntry>>>,
    cluster_manager: Arc<ClusterManager>,
    shutdown_tx: mpsc::Sender<()>,
}

impl RaftNode {
    pub async fn new(config: RaftConfig, cluster_manager: Arc<ClusterManager>) -> Result<Self> {
        let (shutdown_tx, _shutdown_rx) = mpsc::channel(1);
        
        let state = Arc::new(RwLock::new(RaftState {
            node_id: config.node_id.clone(),
            current_term: 0,
            voted_for: None,
            role: RaftRole::Follower,
            commit_index: 0,
            last_applied: 0,
            peers: vec![],
            leader_id: None,
            election_timeout: Duration::from_millis(config.election_timeout_ms),
            heartbeat_interval: Duration::from_millis(config.heartbeat_interval_ms),
        }));

        let logs = Arc::new(Mutex::new(HashMap::new()));

        info!("Raft node initialized with node_id: {}", config.node_id);

        Ok(Self {
            config,
            state,
            logs,
            cluster_manager,
            shutdown_tx,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(self.config.heartbeat_interval_ms));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.tick().await?;
                }
                _ = self.shutdown_tx.closed() => {
                    info!("Raft node shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn tick(&self) -> Result<()> {
        let state = self.state.read().await;
        
        match state.role {
            RaftRole::Follower => {
                // Check if election timeout has passed
                // For now, just log the tick
                debug!("Raft follower tick");
            }
            RaftRole::Candidate => {
                debug!("Raft candidate tick");
                // Start election if timeout
            }
            RaftRole::Leader => {
                debug!("Raft leader tick");
                // Send heartbeat to followers
            }
        }

        // Broadcast Raft state to cluster
        if let Err(e) = self.cluster_manager.broadcast_raft_state(state.clone()).await {
            error!("Failed to broadcast Raft state: {}", e);
        }

        Ok(())
    }

    pub async fn propose_command(&self, command: Vec<u8>) -> Result<u64> {
        let mut state = self.state.write().await;
        
        if state.role != RaftRole::Leader {
            return Err(anyhow::anyhow!("Not leader"));
        }

        let index = state.last_applied + 1;
        let term = state.current_term;

        let entry = RaftLogEntry {
            term,
            index,
            data: command,
        };

        let mut logs = self.logs.lock().await;
        logs.insert(index, entry);
        state.last_applied = index;

        info!("Proposed command at index {}", index);
        Ok(index)
    }

    pub async fn add_peer(&self, peer_id: String, peer_addr: String) -> Result<()> {
        let mut state = self.state.write().await;
        state.peers.push(peer_addr.clone());
        
        info!("Added peer {} at {}", peer_id, peer_addr);
        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: String) -> Result<()> {
        let mut state = self.state.write().await;
        state.peers.retain(|p| p != &peer_id);
        
        info!("Removed peer {}", peer_id);
        Ok(())
    }

    pub async fn is_leader(&self) -> bool {
        let state = self.state.read().await;
        matches!(state.role, RaftRole::Leader)
    }

    pub async fn get_state(&self) -> RaftState {
        self.state.read().await.clone()
    }

    pub async fn get_leader_id(&self) -> Option<String> {
        let state = self.state.read().await;
        state.leader_id.clone()
    }

    pub async fn shutdown(&self) -> Result<()> {
        let _ = self.shutdown_tx.send(()).await;
        Ok(())
    }
}

 