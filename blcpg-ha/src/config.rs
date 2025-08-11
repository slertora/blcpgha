// MIT License
// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::Result;
use config::{Config, File};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub postgresql: PostgreSQLConfig,
    pub api: ApiConfig,
    pub cluster: ClusterConfig,
    pub raft: RaftConfig,
    pub locker: LockerConfig,
    pub logging: LoggingConfig,
    pub redis: RedisConfig,
    pub health: HealthConfig,
    pub critical_failover: CriticalFailoverConfig,
    pub metrics: MetricsConfig,
    pub vip: VipConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub data_dir: String,
    pub bin_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub node_id: String,
    pub cluster_port: u16,
    pub peers: Vec<String>,
    pub heartbeat_interval: String,
    pub timeout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    pub bind_addr: String,
    pub peers: Vec<String>,
    pub node_id: String,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockerConfig {
    pub backend: String,
    pub ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub addrs: Vec<String>,
    pub username: String,
    pub password: String,
    pub db: i64,
    pub sentinel: bool,
    pub master_name: String,
    pub cluster: bool,
    pub tls_enable: bool,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled: bool,
    pub interval: String,
    pub timeout: String,
    pub auto_promote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalFailoverConfig {
    pub enabled: bool,
    pub health_check_interval: String,
    pub failover_timeout: String,
    pub verification_attempts: u32,
    pub min_health_score: f64,
    pub max_lag_bytes: u64,
    pub fencing_timeout: String,
    pub rollback_timeout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipConfig {
    pub r#type: String,
    pub enabled: bool,
    pub fallback_enabled: bool,
    pub fallback_order: Vec<String>,
    pub haproxy: Option<HaproxyConfig>,
    pub proxysql: Option<ProxySqlConfig>,
    pub keepalived: Option<KeepalivedConfig>,
    pub aws_elastic_ip: Option<AwsElasticIpConfig>,
    pub aws_elb: Option<AwsElbConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaproxyConfig {
    pub api_url: String,
    pub username: String,
    pub password: String,
    pub frontend_name: String,
    pub backend_name: String,
    pub server_name: String,
    pub timeout: String,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySqlConfig {
    pub api_url: String,
    pub username: String,
    pub password: String,
    pub hostgroup_id: u32,
    pub server_id: u32,
    pub timeout: String,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepalivedConfig {
    pub config_file: String,
    pub interface: String,
    pub virtual_ip: String,
    pub virtual_router_id: u32,
    pub priority: u32,
    pub advert_int: u32,
    pub auth_pass: String,
    pub nopreempt: bool,
    pub timeout: String,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsElasticIpConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub allocation_id: String,
    pub instance_id: String,
    pub timeout: String,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsElbConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub load_balancer_arn: String,
    pub target_group_arn: String,
    pub instance_id: String,
    pub port: u16,
    pub timeout: String,
    pub retry_attempts: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("/etc/blcpg-ha/config.toml").required(false))
            .add_source(config::Environment::with_prefix("BLCPG").separator("_"))
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        
        // Validate configuration
        app_config.validate()?;
        
        Ok(app_config)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate PostgreSQL configuration
        if self.postgresql.host.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL host must be set"));
        }
        if self.postgresql.port == 0 {
            return Err(anyhow::anyhow!("PostgreSQL port must be set"));
        }
        if self.postgresql.password.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL password must be set"));
        }
        if self.postgresql.user.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL user must be set"));
        }
        if self.postgresql.database.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL database must be set"));
        }
        if self.postgresql.data_dir.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL data_dir must be set"));
        }
        if self.postgresql.bin_dir.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL bin_dir must be set"));
        }

        // Validate API configuration
        if self.api.host.is_empty() {
            return Err(anyhow::anyhow!("API host must be set"));
        }
        if self.api.port == 0 {
            return Err(anyhow::anyhow!("API port must be set"));
        }
        if self.api.auth_token.is_empty() {
            return Err(anyhow::anyhow!("API auth token must be set"));
        }

        // Validate cluster configuration
        if self.cluster.node_id.is_empty() {
            return Err(anyhow::anyhow!("Cluster node_id must be set"));
        }
        if self.cluster.cluster_port == 0 {
            return Err(anyhow::anyhow!("Cluster port must be set"));
        }

        // Validate Raft configuration
        if self.raft.bind_addr.is_empty() {
            return Err(anyhow::anyhow!("Raft bind_addr must be set"));
        }

        // Validate locker configuration
        if self.locker.backend.is_empty() {
            return Err(anyhow::anyhow!("Locker backend must be set"));
        }
        if self.locker.ttl == 0 {
            return Err(anyhow::anyhow!("Locker TTL must be set"));
        }

        // Validate logging configuration
        if self.logging.level.is_empty() {
            return Err(anyhow::anyhow!("Logging level must be set"));
        }
        if self.logging.format.is_empty() {
            return Err(anyhow::anyhow!("Logging format must be set"));
        }
        if self.logging.output.is_empty() {
            return Err(anyhow::anyhow!("Logging output must be set"));
        }

        // Validate Redis configuration
        if self.redis.addrs.is_empty() {
            return Err(anyhow::anyhow!("Redis addrs must be set"));
        }

        // Validate health configuration
        if self.health.interval.is_empty() {
            return Err(anyhow::anyhow!("Health interval must be set"));
        }
        if self.health.timeout.is_empty() {
            return Err(anyhow::anyhow!("Health timeout must be set"));
        }

        // Validate critical failover configuration
        if self.critical_failover.health_check_interval.is_empty() {
            return Err(anyhow::anyhow!("Critical failover health_check_interval must be set"));
        }
        if self.critical_failover.failover_timeout.is_empty() {
            return Err(anyhow::anyhow!("Critical failover failover_timeout must be set"));
        }
        if self.critical_failover.fencing_timeout.is_empty() {
            return Err(anyhow::anyhow!("Critical failover fencing_timeout must be set"));
        }
        if self.critical_failover.rollback_timeout.is_empty() {
            return Err(anyhow::anyhow!("Critical failover rollback_timeout must be set"));
        }

        // Validate metrics configuration
        if self.metrics.prometheus.is_empty() {
            return Err(anyhow::anyhow!("Metrics prometheus endpoint must be set"));
        }

        // Validate VIP configuration
        if self.vip.r#type.is_empty() {
            return Err(anyhow::anyhow!("VIP type must be set"));
        }

        // Validate fallback configuration if enabled
        if self.vip.fallback_enabled {
            if self.vip.fallback_order.is_empty() {
                return Err(anyhow::anyhow!("Fallback order must be specified when fallback is enabled"));
            }
            
            // Validate that all fallback types are supported
            for vip_type in &self.vip.fallback_order {
                match vip_type.as_str() {
                    "haproxy" | "proxysql" | "keepalived" | "aws_elastic_ip" | "aws_elb" => {},
                    _ => return Err(anyhow::anyhow!("Unsupported VIP type in fallback order: {}", vip_type)),
                }
            }
        }
        
        // Validate HAProxy configuration if enabled
        if self.vip.enabled && self.vip.r#type == "haproxy" {
            if let Some(haproxy) = &self.vip.haproxy {
                if haproxy.api_url.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy API URL must be set"));
                }
                if haproxy.username.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy username must be set"));
                }
                if haproxy.password.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy password must be set"));
                }
                if haproxy.frontend_name.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy frontend name must be set"));
                }
                if haproxy.backend_name.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy backend name must be set"));
                }
                if haproxy.server_name.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy server name must be set"));
                }
                if haproxy.timeout.is_empty() {
                    return Err(anyhow::anyhow!("HAProxy timeout must be set"));
                }
                self.validate_duration_string(&haproxy.timeout, "vip.haproxy.timeout")?;
            } else {
                return Err(anyhow::anyhow!("HAProxy configuration is required when VIP type is haproxy"));
            }
        }

        // Validate ProxySQL configuration if enabled
        if self.vip.enabled && self.vip.r#type == "proxysql" {
            if let Some(proxysql) = &self.vip.proxysql {
                if proxysql.api_url.is_empty() {
                    return Err(anyhow::anyhow!("ProxySQL API URL must be set"));
                }
                if proxysql.username.is_empty() {
                    return Err(anyhow::anyhow!("ProxySQL username must be set"));
                }
                if proxysql.password.is_empty() {
                    return Err(anyhow::anyhow!("ProxySQL password must be set"));
                }
                if proxysql.hostgroup_id == 0 {
                    return Err(anyhow::anyhow!("ProxySQL hostgroup_id must be set"));
                }
                if proxysql.server_id == 0 {
                    return Err(anyhow::anyhow!("ProxySQL server_id must be set"));
                }
                if proxysql.timeout.is_empty() {
                    return Err(anyhow::anyhow!("ProxySQL timeout must be set"));
                }
                self.validate_duration_string(&proxysql.timeout, "vip.proxysql.timeout")?;
            } else {
                return Err(anyhow::anyhow!("ProxySQL configuration is required when VIP type is proxysql"));
            }
        }

        // Validate Keepalived configuration if enabled
        if self.vip.enabled && self.vip.r#type == "keepalived" {
            if let Some(keepalived) = &self.vip.keepalived {
                if keepalived.config_file.is_empty() {
                    return Err(anyhow::anyhow!("Keepalived config_file must be set"));
                }
                if keepalived.interface.is_empty() {
                    return Err(anyhow::anyhow!("Keepalived interface must be set"));
                }
                if keepalived.virtual_ip.is_empty() {
                    return Err(anyhow::anyhow!("Keepalived virtual_ip must be set"));
                }
                if keepalived.virtual_router_id == 0 {
                    return Err(anyhow::anyhow!("Keepalived virtual_router_id must be set"));
                }
                if keepalived.priority == 0 {
                    return Err(anyhow::anyhow!("Keepalived priority must be set"));
                }
                if keepalived.advert_int == 0 {
                    return Err(anyhow::anyhow!("Keepalived advert_int must be set"));
                }
                if keepalived.auth_pass.is_empty() {
                    return Err(anyhow::anyhow!("Keepalived auth_pass must be set"));
                }
                if keepalived.timeout.is_empty() {
                    return Err(anyhow::anyhow!("Keepalived timeout must be set"));
                }
                self.validate_duration_string(&keepalived.timeout, "vip.keepalived.timeout")?;
            } else {
                return Err(anyhow::anyhow!("Keepalived configuration is required when VIP type is keepalived"));
            }
        }

        // Validate AWS Elastic IP configuration if enabled
        if self.vip.enabled && self.vip.r#type == "aws_elastic_ip" {
            if let Some(aws_eip) = &self.vip.aws_elastic_ip {
                if aws_eip.region.is_empty() {
                    return Err(anyhow::anyhow!("AWS region cannot be empty"));
                }
                if aws_eip.access_key_id.is_empty() {
                    return Err(anyhow::anyhow!("AWS access key ID cannot be empty"));
                }
                if aws_eip.secret_access_key.is_empty() {
                    return Err(anyhow::anyhow!("AWS secret access key cannot be empty"));
                }
                if aws_eip.allocation_id.is_empty() {
                    return Err(anyhow::anyhow!("AWS allocation ID cannot be empty"));
                }
                if aws_eip.instance_id.is_empty() {
                    return Err(anyhow::anyhow!("AWS instance ID cannot be empty"));
                }
                if aws_eip.retry_attempts == 0 {
                    return Err(anyhow::anyhow!("AWS retry attempts must be greater than 0"));
                }
                self.validate_duration_string(&aws_eip.timeout, "vip.aws_elastic_ip.timeout")?;
            } else {
                return Err(anyhow::anyhow!("AWS Elastic IP configuration is required when VIP type is aws_elastic_ip"));
            }
        }

        // Validate AWS ELB configuration if enabled
        if self.vip.enabled && self.vip.r#type == "aws_elb" {
            if let Some(aws_elb) = &self.vip.aws_elb {
                if aws_elb.region.is_empty() {
                    return Err(anyhow::anyhow!("AWS region cannot be empty"));
                }
                if aws_elb.access_key_id.is_empty() {
                    return Err(anyhow::anyhow!("AWS access key ID cannot be empty"));
                }
                if aws_elb.secret_access_key.is_empty() {
                    return Err(anyhow::anyhow!("AWS secret access key cannot be empty"));
                }
                if aws_elb.load_balancer_arn.is_empty() {
                    return Err(anyhow::anyhow!("AWS load balancer ARN cannot be empty"));
                }
                if aws_elb.target_group_arn.is_empty() {
                    return Err(anyhow::anyhow!("AWS target group ARN cannot be empty"));
                }
                if aws_elb.instance_id.is_empty() {
                    return Err(anyhow::anyhow!("AWS instance ID cannot be empty"));
                }
                if aws_elb.port == 0 {
                    return Err(anyhow::anyhow!("AWS ELB port must be greater than 0"));
                }
                if aws_elb.retry_attempts == 0 {
                    return Err(anyhow::anyhow!("AWS retry attempts must be greater than 0"));
                }
                self.validate_duration_string(&aws_elb.timeout, "vip.aws_elb.timeout")?;
            } else {
                return Err(anyhow::anyhow!("AWS ELB configuration is required when VIP type is aws_elb"));
            }
        }

        // Validate timeouts and intervals
        self.validate_duration_string(&self.cluster.heartbeat_interval, "cluster.heartbeat_interval")?;
        self.validate_duration_string(&self.cluster.timeout, "cluster.timeout")?;
        self.validate_duration_string(&self.health.interval, "health.interval")?;
        self.validate_duration_string(&self.health.timeout, "health.timeout")?;
        self.validate_duration_string(&self.critical_failover.health_check_interval, "critical_failover.health_check_interval")?;
        self.validate_duration_string(&self.critical_failover.failover_timeout, "critical_failover.failover_timeout")?;
        self.validate_duration_string(&self.critical_failover.fencing_timeout, "critical_failover.fencing_timeout")?;
        self.validate_duration_string(&self.critical_failover.rollback_timeout, "critical_failover.rollback_timeout")?;

        Ok(())
    }

    fn validate_duration_string(&self, duration_str: &str, field_name: &str) -> Result<()> {
        if let Err(_) = humantime::Duration::from_str(duration_str) {
            return Err(anyhow::anyhow!("Invalid duration format for {}: {}", field_name, duration_str));
        }
        Ok(())
    }
} 