# BLC PostgreSQL HA

**Enterprise-Grade PostgreSQL High Availability Solution**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-12+-green.svg)](https://www.postgresql.org/)

## 🚀 Overview

BLC PostgreSQL HA is a modern, high-performance high availability solution for PostgreSQL clusters, written in Rust. It provides automatic failover, intelligent replica management, and enterprise-grade monitoring through a unified platform.

## ✨ Features

### 🔧 Core Features
- **Automatic Failover**: Intelligent detection and promotion of healthy replicas
- **Raft Consensus**: Distributed consensus for cluster coordination
- **Health Monitoring**: Comprehensive health checks and metrics
- **Replica Management**: Automated replica addition and maintenance
- **Zero-Downtime Operations**: Planned switchovers and maintenance

### 🎛️ Management Tools
- **`blcpg-ha`**: High-performance agent for each PostgreSQL node
- **`blcpg-cli`**: Command-line interface for cluster management
- **`blcpg-gui`**: Web-based dashboard for real-time monitoring

### 🏗️ Architecture
- **Single Binary**: One executable per node for easy deployment
- **HTTP REST API**: Modern API for integration and automation
- **WebSocket Support**: Real-time updates and notifications
- **Cross-Platform**: Linux and macOS support

## 🏁 Quick Start

### Prerequisites
- Rust 1.70+ 
- PostgreSQL 12+
- Linux or macOS

### Installation

```bash
# Clone the repository
git clone git@github.com:binlogic/postgresqlha.git
cd postgresqlha

# Build all components
cargo build --release

# The binaries will be in target/release/
# - blcpg-ha (agent)
# - blcpg-cli (CLI tool)
# - blcpg-gui (web GUI)
```

### Basic Usage

```bash
# Start the agent on a PostgreSQL node
./blcpg-ha --config config.toml

# Check cluster status
./blcpg-cli status

# Open the web GUI
./blcpg-gui --port 3000
```

## 📁 Project Structure

```
blcpgha/
├── blcpg-ha/          # High availability agent
├── blcpg-cli/         # Command-line interface
├── blcpg-gui/         # Web-based dashboard
├── docs/              # Documentation
├── scripts/           # Utility scripts
└── release/           # Release binaries
```

## 🔧 Configuration

### Agent Configuration (`blcpg-ha/config.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8080

[postgresql]
host = "localhost"
port = 5432
database = "postgres"
username = "postgres"

[raft]
data_dir = "/var/lib/blcpg-ha"
bind_addr = "0.0.0.0:5000"
```

### CLI Configuration (`blcpg-cli/config.toml`)

```toml
[api]
host = "localhost"
port = 8080
timeout = 30
```

## 📊 Monitoring & Metrics

- **Real-time Dashboard**: Live cluster status and metrics
- **Health Checks**: Comprehensive node and cluster health monitoring
- **Performance Metrics**: Query performance, replication lag, resource usage
- **Event Logging**: Detailed audit trail of all operations

## 🚀 Advanced Features

### Replica Management
```bash
# Add a new replica
blcpg-cli add-replica --target node-4 --source auto

# Perform planned switchover
blcpg-cli switchover --target node-2

# Emergency failover
blcpg-cli failover
```

### Backup & Recovery
```bash
# Create intelligent backup
blcpg-cli backup --target node-1 --type full

# Restore from backup
blcpg-cli restore --target node-2 --backup-id backup-123

# Maintenance operations
blcpg-cli maintenance --target node-1 --type vacuum
```

### Version Management
```bash
# Upgrade PostgreSQL version
blcpg-cli upgrade --target node-1 --version 15.3

# Configuration management
blcpg-cli config --target node-1 --action get --key max_connections
```

## 🔒 Security

- **SSH Key Authentication**: Secure node-to-node communication
- **TLS Support**: Encrypted API communications
- **Role-Based Access**: Granular permissions for different operations
- **Audit Logging**: Complete audit trail of all changes

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific component tests
cargo test -p blcpg-cli
cargo test -p blcpg-ha
cargo test -p blcpg-gui

# Integration tests
cargo test --test integration
```

## 📈 Performance

- **Low Latency**: Sub-second failover detection
- **High Throughput**: Efficient consensus and replication
- **Resource Efficient**: Minimal CPU and memory footprint
- **Scalable**: Support for large cluster deployments

## 🌐 Deployment

### Docker
```bash
docker run -d --name blcpg-ha \
  -v /etc/blcpg-ha:/config \
  -p 8080:8080 \
  binlogic/blcpg-ha:latest
```

### Kubernetes
```bash
# Apply Helm chart
helm install blcpg-ha ./charts/blcpg-ha

# Or use the operator
kubectl apply -f deploy/operator.yaml
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
```bash
# Clone and setup
git clone git@github.com:binlogic/postgresqlha.git
cd postgresqlha

# Install development dependencies
cargo install cargo-watch
cargo install cross

# Development workflow
cargo watch -x check -x test -x run
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏢 About Binlogic

BLC PostgreSQL HA is developed and maintained by [Binlogic](https://binlogic.com), a leading provider of database solutions and services.

## 📞 Support

- **Documentation**: [docs.binlogic.com](https://docs.binlogic.com)
- **Issues**: [GitHub Issues](https://github.com/binlogic/postgresqlha/issues)
- **Discussions**: [GitHub Discussions](https://github.com/binlogic/postgresqlha/discussions)
- **Email**: support@binlogic.com

---

**Built with ❤️ by the Binlogic Team** 