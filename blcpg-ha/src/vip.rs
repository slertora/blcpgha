// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use crate::config::{
    AwsElasticIpConfig, AwsElbConfig, HaproxyConfig, KeepalivedConfig, ProxySqlConfig, VipConfig,
};
use anyhow::Result;
use aws_sdk_ec2::{config::Credentials, config::Region, Client as Ec2Client};
use aws_sdk_elasticloadbalancingv2::Client as ElbClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct VipManager {
    config: VipConfig,
    client: Client,
    ec2_client: Option<Ec2Client>,
    elb_client: Option<ElbClient>,
    current_primary: Arc<RwLock<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HaproxyServer {
    name: String,
    address: String,
    port: u16,
    check: String,
    inter: String,
    rise: u32,
    fall: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct HaproxyBackend {
    name: String,
    balance: String,
    mode: String,
    servers: Vec<HaproxyServer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxySqlServer {
    hostgroup_id: u32,
    hostname: String,
    port: u16,
    status: String,
    weight: u32,
    compression: u32,
    max_connections: u32,
    max_replication_lag: u32,
    use_ssl: u32,
    max_latency_ms: u32,
    comment: String,
}

impl VipManager {
    pub async fn new(config: VipConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Initialize AWS EC2 client if AWS Elastic IP is configured
        let ec2_client = if config.enabled
            && (config.r#type == "aws_elastic_ip" || config.r#type == "aws_elb")
        {
            if let Some(aws_config) = &config.aws_elastic_ip {
                let credentials = Credentials::new(
                    &aws_config.access_key_id,
                    &aws_config.secret_access_key,
                    None,
                    None,
                    "blcpg-ha",
                );

                let config = aws_sdk_ec2::Config::builder()
                    .region(Region::new(aws_config.region.clone()))
                    .credentials_provider(credentials)
                    .build();

                Some(Ec2Client::from_conf(config))
            } else if let Some(aws_config) = &config.aws_elb {
                let credentials = Credentials::new(
                    &aws_config.access_key_id,
                    &aws_config.secret_access_key,
                    None,
                    None,
                    "blcpg-ha",
                );

                let config = aws_sdk_ec2::Config::builder()
                    .region(Region::new(aws_config.region.clone()))
                    .credentials_provider(credentials)
                    .build();

                Some(Ec2Client::from_conf(config))
            } else {
                None
            }
        } else {
            None
        };

        // Initialize AWS ELB client if AWS ELB is configured
        let elb_client = if config.enabled && config.r#type == "aws_elb" {
            if let Some(aws_config) = &config.aws_elb {
                let credentials = Credentials::new(
                    &aws_config.access_key_id,
                    &aws_config.secret_access_key,
                    None,
                    None,
                    "blcpg-ha",
                );

                let config = aws_sdk_elasticloadbalancingv2::Config::builder()
                    .region(Region::new(aws_config.region.clone()))
                    .credentials_provider(credentials)
                    .build();

                Some(ElbClient::from_conf(config))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            config,
            client,
            ec2_client,
            elb_client,
            current_primary: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn update_primary(&self, primary_node: &str) -> Result<()> {
        if !self.config.enabled {
            debug!("VIP management is disabled");
            return Ok(());
        }

        if self.config.fallback_enabled {
            self.update_primary_with_fallback(primary_node).await
        } else {
            self.update_primary_single(primary_node).await
        }
    }

    async fn update_primary_single(&self, primary_node: &str) -> Result<()> {
        match self.config.r#type.as_str() {
            "haproxy" => self.update_haproxy_primary(primary_node).await,
            "proxysql" => self.update_proxysql_primary(primary_node).await,
            "keepalived" => self.update_keepalived_primary(primary_node).await,
            "aws_elastic_ip" => self.update_aws_elastic_ip_primary(primary_node).await,
            "aws_elb" => self.update_aws_elb_primary(primary_node).await,
            _ => {
                warn!("Unsupported VIP type: {}", self.config.r#type);
                Ok(())
            }
        }
    }

    async fn update_primary_with_fallback(&self, primary_node: &str) -> Result<()> {
        let mut last_error = None;

        for vip_type in &self.config.fallback_order {
            info!("Attempting to update VIP using: {}", vip_type);

            let result = match vip_type.as_str() {
                "haproxy" => self.update_haproxy_primary(primary_node).await,
                "proxysql" => self.update_proxysql_primary(primary_node).await,
                "keepalived" => self.update_keepalived_primary(primary_node).await,
                "aws_elastic_ip" => self.update_aws_elastic_ip_primary(primary_node).await,
                "aws_elb" => self.update_aws_elb_primary(primary_node).await,
                _ => {
                    warn!("Unsupported VIP type in fallback order: {}", vip_type);
                    continue;
                }
            };

            match result {
                Ok(_) => {
                    info!("Successfully updated VIP using: {}", vip_type);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to update VIP using {}: {}", vip_type, e);
                    last_error = Some(e);
                }
            }
        }

        // If we get here, all fallback methods failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All VIP update methods failed")))
    }

    async fn update_aws_elb_primary(&self, primary_node: &str) -> Result<()> {
        let aws_config = self
            .config
            .aws_elb
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS ELB configuration not found"))?;

        let elb_client = self
            .elb_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS ELB client not initialized"))?;

        info!(
            "Updating AWS ELB target group to instance: {}",
            primary_node
        );

        // Parse primary node address (assuming format: host:port)
        let (host, _port) = self.parse_node_address(primary_node)?;

        // First, deregister all current targets from the target group
        info!("Deregistering current targets from target group");
        match elb_client
            .describe_target_health()
            .target_group_arn(aws_config.target_group_arn.clone())
            .send()
            .await
        {
            Ok(response) => {
                if let Some(target_health_descriptions) = response.target_health_descriptions {
                    for target in target_health_descriptions {
                        if let Some(target_id) = target.target {
                            if let Some(id) = &target_id.id {
                                info!("Deregistering target: {}", id);
                                if let Err(e) = elb_client
                                    .deregister_targets()
                                    .target_group_arn(aws_config.target_group_arn.clone())
                                    .targets(aws_sdk_elasticloadbalancingv2::types::TargetDescription::builder()
                                        .id(id.clone())
                                        .build())
                                    .send()
                                    .await
                                {
                                    warn!("Failed to deregister target {}: {}", id, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to describe target health: {}", e);
            }
        }

        // Register the new primary instance as a target
        info!(
            "Registering new primary instance: {}:{}",
            host, aws_config.port
        );
        match elb_client
            .register_targets()
            .target_group_arn(aws_config.target_group_arn.clone())
            .targets(
                aws_sdk_elasticloadbalancingv2::types::TargetDescription::builder()
                    .id(host.to_string())
                    .port(aws_config.port.into())
                    .build(),
            )
            .send()
            .await
        {
            Ok(_) => {
                info!(
                    "Successfully registered target {}:{} with ELB",
                    host, aws_config.port
                );

                // Update current primary
                let mut current = self.current_primary.write().await;
                *current = Some(primary_node.to_string());

                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to register target with ELB: {}", e)),
        }
    }

    async fn update_haproxy_primary(&self, primary_node: &str) -> Result<()> {
        let haproxy_config = self
            .config
            .haproxy
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HAProxy configuration not found"))?;

        info!("Updating HAProxy primary to: {}", primary_node);

        // Parse primary node address (assuming format: host:port)
        let (host, port) = self.parse_node_address(primary_node)?;
        let port = port.parse::<u16>()?;

        // Create server configuration
        let server = HaproxyServer {
            name: haproxy_config.server_name.clone(),
            address: host.to_string(),
            port,
            check: "enabled".to_string(),
            inter: "2s".to_string(),
            rise: 2,
            fall: 3,
        };

        // Create backend configuration
        let backend = HaproxyBackend {
            name: haproxy_config.backend_name.clone(),
            balance: "roundrobin".to_string(),
            mode: "tcp".to_string(),
            servers: vec![server],
        };

        // Update HAProxy via Data Plane API
        self.update_haproxy_backend(haproxy_config, &backend)
            .await?;

        // Update current primary
        let mut current = self.current_primary.write().await;
        *current = Some(primary_node.to_string());

        info!("HAProxy primary updated successfully to: {}", primary_node);
        Ok(())
    }

    async fn update_proxysql_primary(&self, primary_node: &str) -> Result<()> {
        let proxysql_config = self
            .config
            .proxysql
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("ProxySQL configuration not found"))?;

        info!("Updating ProxySQL primary to: {}", primary_node);

        // Parse primary node address (assuming format: host:port)
        let (host, port) = self.parse_node_address(primary_node)?;
        let port = port.parse::<u16>()?;

        // Create server configuration
        let server = ProxySqlServer {
            hostgroup_id: proxysql_config.hostgroup_id,
            hostname: host.to_string(),
            port,
            status: "ONLINE".to_string(),
            weight: 1000,
            compression: 0,
            max_connections: 1000,
            max_replication_lag: 10,
            use_ssl: 0,
            max_latency_ms: 1000,
            comment: "Primary server managed by BLC PostgreSQL HA".to_string(),
        };

        // Update ProxySQL via REST API
        self.update_proxysql_server(proxysql_config, &server)
            .await?;

        // Update current primary
        let mut current = self.current_primary.write().await;
        *current = Some(primary_node.to_string());

        info!("ProxySQL primary updated successfully to: {}", primary_node);
        Ok(())
    }

    async fn update_keepalived_primary(&self, primary_node: &str) -> Result<()> {
        let keepalived_config = self
            .config
            .keepalived
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Keepalived configuration not found"))?;

        info!("Updating Keepalived primary to: {}", primary_node);

        // Parse primary node address (assuming format: host:port)
        let (host, _port) = self.parse_node_address(primary_node)?;

        // Generate Keepalived configuration
        let config_content = self.generate_keepalived_config(keepalived_config, host)?;

        // Write configuration file
        tokio::fs::write(&keepalived_config.config_file, config_content).await?;

        // Reload Keepalived service
        self.reload_keepalived_service().await?;

        // Update current primary
        let mut current = self.current_primary.write().await;
        *current = Some(primary_node.to_string());

        info!(
            "Keepalived primary updated successfully to: {}",
            primary_node
        );
        Ok(())
    }

    async fn update_haproxy_backend(
        &self,
        config: &HaproxyConfig,
        backend: &HaproxyBackend,
    ) -> Result<()> {
        let url = format!(
            "{}/services/haproxy/configuration/backends/{}",
            config.api_url, backend.name
        );

        // First, try to delete existing backend
        let delete_url = format!(
            "{}/services/haproxy/configuration/backends/{}",
            config.api_url, backend.name
        );
        let _ = self
            .client
            .delete(&delete_url)
            .basic_auth(&config.username, Some(&config.password))
            .send()
            .await;

        // Create new backend
        let response = self
            .client
            .post(&url)
            .basic_auth(&config.username, Some(&config.password))
            .json(backend)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Failed to update HAProxy backend: {}",
                error_text
            ));
        }

        debug!("HAProxy backend updated successfully");
        Ok(())
    }

    async fn update_proxysql_server(
        &self,
        config: &ProxySqlConfig,
        server: &ProxySqlServer,
    ) -> Result<()> {
        let url = format!("{}/servers/{}", config.api_url, config.server_id);

        // First, try to delete existing server
        let delete_url = format!("{}/servers/{}", config.api_url, config.server_id);
        let _ = self
            .client
            .delete(&delete_url)
            .basic_auth(&config.username, Some(&config.password))
            .send()
            .await;

        // Create new server
        let response = self
            .client
            .post(&url)
            .basic_auth(&config.username, Some(&config.password))
            .json(server)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Failed to update ProxySQL server: {}",
                error_text
            ));
        }

        debug!("ProxySQL server updated successfully");
        Ok(())
    }

    fn generate_keepalived_config(
        &self,
        config: &KeepalivedConfig,
        primary_host: &str,
    ) -> Result<String> {
        let config_content = format!(
            r#"global_defs {{
    router_id BLC_PG_HA
}}

vrrp_script check_postgresql {{
    script "pg_isready -h {} -p 5432"
    interval 2
    weight 2
    fall 2
    rise 2
}}

vrrp_instance VI_1 {{
    state MASTER
    interface {}
    virtual_router_id {}
    priority {}
    advert_int {}
    authentication {{
        auth_type PASS
        auth_pass {}
    }}
    virtual_ipaddress {{
        {}
    }}
    track_script {{
        check_postgresql
    }}
    nopreempt {}
}}"#,
            primary_host,
            config.interface,
            config.virtual_router_id,
            config.priority,
            config.advert_int,
            config.auth_pass,
            config.virtual_ip,
            if config.nopreempt { "yes" } else { "no" }
        );

        Ok(config_content)
    }

    async fn reload_keepalived_service(&self) -> Result<()> {
        // Try to reload keepalived configuration
        let output = Command::new("systemctl")
            .args(&["reload", "keepalived"])
            .output()?;

        if !output.status.success() {
            // If reload fails, try restart
            let restart_output = Command::new("systemctl")
                .args(&["restart", "keepalived"])
                .output()?;

            if !restart_output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to reload/restart keepalived: {}",
                    String::from_utf8_lossy(&restart_output.stderr)
                ));
            }
        }

        debug!("Keepalived service reloaded successfully");
        Ok(())
    }

    fn parse_node_address<'a>(&self, node_addr: &'a str) -> Result<(&'a str, &'a str)> {
        let parts: Vec<&str> = node_addr.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid node address format: {}",
                node_addr
            ));
        }
        Ok((parts[0], parts[1]))
    }

    pub async fn get_current_primary(&self) -> Option<String> {
        self.current_primary.read().await.clone()
    }

    pub async fn health_check(&self) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        if self.config.fallback_enabled {
            self.health_check_with_fallback().await
        } else {
            self.health_check_single().await
        }
    }

    async fn health_check_single(&self) -> Result<bool> {
        match self.config.r#type.as_str() {
            "haproxy" => self.haproxy_health_check().await,
            "proxysql" => self.proxysql_health_check().await,
            "keepalived" => self.keepalived_health_check().await,
            "aws_elastic_ip" => self.aws_elastic_ip_health_check().await,
            "aws_elb" => self.aws_elb_health_check().await,
            _ => {
                warn!(
                    "Unsupported VIP type for health check: {}",
                    self.config.r#type
                );
                Ok(true)
            }
        }
    }

    async fn health_check_with_fallback(&self) -> Result<bool> {
        for vip_type in &self.config.fallback_order {
            debug!("Checking health of VIP type: {}", vip_type);

            let result = match vip_type.as_str() {
                "haproxy" => self.haproxy_health_check().await,
                "proxysql" => self.proxysql_health_check().await,
                "keepalived" => self.keepalived_health_check().await,
                "aws_elastic_ip" => self.aws_elastic_ip_health_check().await,
                "aws_elb" => self.aws_elb_health_check().await,
                _ => {
                    warn!("Unsupported VIP type in fallback order: {}", vip_type);
                    continue;
                }
            };

            match result {
                Ok(true) => {
                    debug!("VIP type {} is healthy", vip_type);
                    return Ok(true);
                }
                Ok(false) => {
                    debug!("VIP type {} is unhealthy", vip_type);
                    continue;
                }
                Err(e) => {
                    warn!("Error checking health of VIP type {}: {}", vip_type, e);
                    continue;
                }
            }
        }

        // If we get here, all VIP types are unhealthy
        Ok(false)
    }

    async fn haproxy_health_check(&self) -> Result<bool> {
        let haproxy_config = self
            .config
            .haproxy
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HAProxy configuration not found"))?;

        let url = format!("{}/services/haproxy/stats/native", haproxy_config.api_url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&haproxy_config.username, Some(&haproxy_config.password))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn proxysql_health_check(&self) -> Result<bool> {
        let proxysql_config = self
            .config
            .proxysql
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("ProxySQL configuration not found"))?;

        let url = format!("{}/status", proxysql_config.api_url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&proxysql_config.username, Some(&proxysql_config.password))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn keepalived_health_check(&self) -> Result<bool> {
        // Check if keepalived process is running
        let output = Command::new("systemctl")
            .args(&["is-active", "keepalived"])
            .output()?;

        let is_running =
            output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "active";

        if !is_running {
            return Ok(false);
        }

        // Check if keepalived configuration file exists
        let keepalived_config = self
            .config
            .keepalived
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Keepalived configuration not found"))?;

        let config_exists = tokio::fs::try_exists(&keepalived_config.config_file).await?;

        Ok(config_exists)
    }

    async fn aws_elastic_ip_health_check(&self) -> Result<bool> {
        let aws_config = self
            .config
            .aws_elastic_ip
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS Elastic IP configuration not found"))?;

        let ec2_client = self
            .ec2_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS EC2 client not initialized"))?;

        // Check if the Elastic IP allocation exists and is available
        match ec2_client
            .describe_addresses()
            .allocation_ids(aws_config.allocation_id.clone())
            .send()
            .await
        {
            Ok(response) => {
                if let Some(addresses) = response.addresses {
                    if addresses.is_empty() {
                        return Ok(false);
                    }

                    // Check if the allocation exists (in older AWS SDK, we just check if it's not empty)
                    if !addresses.is_empty() {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }

    async fn aws_elb_health_check(&self) -> Result<bool> {
        let aws_config = self
            .config
            .aws_elb
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS ELB configuration not found"))?;

        let elb_client = self
            .elb_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS ELB client not initialized"))?;

        // Check if the load balancer exists and is active
        match elb_client
            .describe_load_balancers()
            .load_balancer_arns(aws_config.load_balancer_arn.clone())
            .send()
            .await
        {
            Ok(response) => {
                if let Some(load_balancers) = response.load_balancers {
                    if load_balancers.is_empty() {
                        return Ok(false);
                    }

                    // Check if the load balancer is active
                    for lb in load_balancers {
                        if let Some(state) = lb.state {
                            if let Some(code) = state.code {
                                if code == "active".into() {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }

    async fn update_aws_elastic_ip_primary(&self, primary_node: &str) -> Result<()> {
        let aws_config = self
            .config
            .aws_elastic_ip
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS Elastic IP configuration not found"))?;

        let ec2_client = self
            .ec2_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AWS EC2 client not initialized"))?;

        info!("Updating AWS Elastic IP to instance: {}", primary_node);

        // Parse primary node address (assuming format: host:port)
        let (host, _port) = self.parse_node_address(primary_node)?;

        // First, disassociate the Elastic IP from any current instance
        match ec2_client
            .describe_addresses()
            .allocation_ids(aws_config.allocation_id.clone())
            .send()
            .await
        {
            Ok(response) => {
                if let Some(addresses) = response.addresses {
                    for address in addresses {
                        if let Some(association_id) = address.association_id {
                            info!("Disassociating Elastic IP from current instance");
                            if let Err(e) = ec2_client
                                .disassociate_address()
                                .association_id(association_id)
                                .send()
                                .await
                            {
                                warn!("Failed to disassociate Elastic IP: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to describe addresses: {}", e);
            }
        }

        // Associate the Elastic IP with the new primary instance
        info!("Associating Elastic IP with instance: {}", host);
        match ec2_client
            .associate_address()
            .allocation_id(aws_config.allocation_id.clone())
            .instance_id(host.to_string())
            .send()
            .await
        {
            Ok(response) => {
                if let Some(association_id) = response.association_id {
                    info!(
                        "Elastic IP associated successfully with association ID: {}",
                        association_id
                    );

                    // Update current primary
                    let mut current = self.current_primary.write().await;
                    *current = Some(primary_node.to_string());

                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to get association ID from AWS response"
                    ))
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to associate Elastic IP: {}", e)),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down VIP manager");
        Ok(())
    }
}
