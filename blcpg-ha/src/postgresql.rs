// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::PostgreSQLConfig;
use anyhow::Result;
use tokio_postgres::NoTls;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct PostgreSQLConnection {
    client: Arc<Mutex<tokio_postgres::Client>>,
    config: PostgreSQLConfig,
}

impl PostgreSQLConnection {
    pub async fn new(config: PostgreSQLConfig) -> Result<Self> {
        let connection_string = format!(
            "host={} port={} user={} password={} dbname={}",
            config.host, config.port, config.user, config.password, config.database
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;
        
        // Spawn the connection to run in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("PostgreSQL connection error: {}", e);
            }
        });

        let client = Arc::new(Mutex::new(client));

        info!("PostgreSQL connection established to {}:{}", config.host, config.port);

        Ok(Self { client, config })
    }

    pub async fn health_check(&self) -> Result<bool> {
        let client = self.client.lock().await;
        
        match client.simple_query("SELECT 1").await {
            Ok(_) => {
                debug!("PostgreSQL health check passed");
                Ok(true)
            }
            Err(e) => {
                error!("PostgreSQL health check failed: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn get_replication_lag(&self) -> Result<Option<u64>> {
        let client = self.client.lock().await;
        
        let query = "
            SELECT 
                CASE 
                    WHEN pg_is_in_recovery() THEN 
                        EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) * 1000000
                    ELSE 0
                END as lag_microseconds
        ";

        match client.query_one(query, &[]).await {
            Ok(row) => {
                let lag: Option<f64> = row.get("lag_microseconds");
                Ok(lag.map(|l| l as u64))
            }
            Err(e) => {
                error!("Failed to get replication lag: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn is_primary(&self) -> Result<bool> {
        let client = self.client.lock().await;
        
        match client.query_one("SELECT pg_is_in_recovery()", &[]).await {
            Ok(row) => {
                let in_recovery: bool = row.get(0);
                Ok(!in_recovery)
            }
            Err(e) => {
                error!("Failed to check if primary: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn promote(&self) -> Result<bool> {
        let client = self.client.lock().await;
        
        // Check if we're in recovery mode
        let row = client.query_one("SELECT pg_is_in_recovery()", &[]).await?;
        let in_recovery: bool = row.get(0);
        
        if !in_recovery {
            info!("Already primary, no promotion needed");
            return Ok(true);
        }

        // Execute promotion
        match client.simple_query("SELECT pg_promote()").await {
            Ok(_) => {
                info!("PostgreSQL promotion successful");
                Ok(true)
            }
            Err(e) => {
                error!("PostgreSQL promotion failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn get_config(&self) -> &PostgreSQLConfig {
        &self.config
    }

    pub async fn get_server_version(&self) -> Result<Option<String>> {
        let client = self.client.lock().await;
        
        match client.query_one("SELECT version()", &[]).await {
            Ok(row) => {
                let version: String = row.get(0);
                Ok(Some(version))
            }
            Err(e) => {
                warn!("Failed to get server version: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn get_current_wal_lsn(&self) -> Result<Option<String>> {
        let client = self.client.lock().await;
        
        let query = "
            SELECT 
                CASE 
                    WHEN pg_is_in_recovery() THEN 
                        pg_last_wal_receive_lsn()::text
                    ELSE 
                        pg_current_wal_lsn()::text
                END as current_lsn
        ";

        match client.query_one(query, &[]).await {
            Ok(row) => {
                let lsn: Option<String> = row.get("current_lsn");
                Ok(lsn)
            }
            Err(e) => {
                error!("Failed to get current WAL LSN: {}", e);
                Ok(None)
            }
        }
    }
}

pub async fn connect(config: &PostgreSQLConfig) -> Result<PostgreSQLConnection> {
    PostgreSQLConnection::new(config.clone()).await
} 