# BLC PostgreSQL HA

**Enterprise-Grade PostgreSQL High Availability Solution**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-12+-green.svg)](https://www.postgresql.org/)
[![CI/CD](https://img.shields.io/badge/CI%2FCD-GitHub%20Actions-brightgreen.svg)](https://github.com/slertora/blcpgha/actions)

## 🚀 Overview

BLC PostgreSQL HA is a modern, high-performance high availability solution for PostgreSQL clusters, written in Rust. It provides automatic failover, intelligent replica management, and enterprise-grade monitoring through a unified platform.

**🎉 NOW WITH COMPLETE CI/CD PIPELINE!** - Automated testing, compilation, and deployment to production servers.

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

### 🚀 DevOps & Production
- **GitHub Actions CI/CD**: Automated testing and deployment
- **Production Ready**: Deployed on 3 production servers
- **Health Checks**: Automated post-deployment verification
- **Manual Scripts**: Complete deployment automation tools

## 🏁 Quick Start

### 🔧 Development Workflow

The complete development workflow is now automated! Use our workflow script:

```bash
# Complete setup (one-time)
./scripts/dev-workflow.sh full

# Daily development flow
./scripts/dev-workflow.sh quick-test    # Test your changes
./scripts/dev-workflow.sh quick-push    # Push to GitHub (auto-deploys)
./scripts/dev-workflow.sh status       # Check deployment status
```

### 📋 Prerequisites
- Rust 1.70+ 
- PostgreSQL 12+
- SSH access to production servers
- GitHub repository configured

### 🚀 Production Deployment

#### Automated Deployment (Recommended)
```bash
# Push to rustmaster branch triggers automatic deployment
git push slertora rustmaster
```

#### Manual Deployment
```bash
# Deploy to all servers
./scripts/deploy-manual.sh all install

# Update existing deployment
./scripts/deploy-manual.sh all update

# Check health
./scripts/deploy-manual.sh all health
```

## 🖥️ Production Servers

Our solution is deployed on 3 production servers:

| Server | Role | IP Address | Web Interface |
|--------|------|------------|---------------|
| `postgre1` | **Master** | `187.33.155.182` | http://187.33.155.182:3000 |
| `postgresql2` | **Replica 1** | `187.33.144.18` | http://187.33.144.18:3000 |
| `postgresql3` | **Replica 2** | `187.33.147.49` | http://187.33.147.49:3000 |

## 📊 Components

### 🔥 blcpg-ha (High Availability Agent)
The core agent that runs on each PostgreSQL node, providing:
- Raft consensus coordination
- Health monitoring and metrics
- REST API endpoints
- Automatic failover logic

```bash
# Start the agent
./target/release/blcpg-ha --config config.toml

# Available at: http://localhost:8080/api/v1/
```

### 💻 blcpg-cli (Command Line Interface)
Complete CLI tool for cluster management:

```bash
# Check cluster status
./target/release/blcpg-cli status

# Add a replica
./target/release/blcpg-cli add-replica --host 192.168.1.100

# Perform switchover
./target/release/blcpg-cli switchover --target replica1

# Full command list
./target/release/blcpg-cli --help
```

### 🌐 blcpg-gui (Web Interface)
Modern web dashboard for real-time monitoring:

```bash
# Start the web interface
./target/release/blcpg-gui --config config.toml

# Available at: http://localhost:3000
```

**Features:**
- Real-time cluster status
- Interactive node management
- Performance metrics
- Configuration management
- Health monitoring

## 🛠️ Development Scripts

### 📦 Server Management
```bash
# Prepare servers with dependencies
./scripts/prepare-servers.sh prepare all

# Test connectivity
./scripts/prepare-servers.sh test all

# Run health checks
./scripts/prepare-servers.sh health all
```

### 🔍 Testing & Verification
```bash
# Test complete pipeline
./scripts/test-pipeline.sh full

# Test specific components
./scripts/test-pipeline.sh compilation
./scripts/test-pipeline.sh connectivity
./scripts/test-pipeline.sh api
```

### ⚡ Quick Actions
```bash
# Quick deployment test
./scripts/dev-workflow.sh quick-deploy

# Quick connectivity test
./scripts/dev-workflow.sh quick-test

# Quick commit and push
./scripts/dev-workflow.sh quick-push

# System status overview
./scripts/dev-workflow.sh status
```

## 🔧 Configuration

### Environment Variables
```bash
# blcpg-ha
export BLCPG_SERVER_HOST=0.0.0.0
export BLCPG_SERVER_PORT=8080
export BLCPG_LOG_LEVEL=info

# blcpg-gui  
export BLCGUI_SERVER_HOST=0.0.0.0
export BLCGUI_SERVER_PORT=3000
export BLCGUI_API_BASE_URL=http://localhost:8080/api/v1
```

### Configuration Files
Each component uses TOML configuration files:
- `blcpg-ha/config.toml` - Agent configuration
- `blcpg-cli/config.toml` - CLI defaults
- `blcpg-gui/config.toml` - Web interface settings

## 📡 API Reference

### Health Check
```bash
curl http://localhost:8080/api/v1/health
```

### Cluster Status
```bash
curl http://localhost:8080/api/v1/cluster/status
```

### Node Management
```bash
# List nodes
curl http://localhost:8080/api/v1/nodes

# Node details
curl http://localhost:8080/api/v1/nodes/{node_id}
```

### Metrics
```bash
curl http://localhost:8080/api/v1/metrics
```

### VIP Status
```bash
curl http://localhost:8080/api/v1/vip/status
```

## 🚀 GitHub Actions CI/CD

Our automated pipeline includes:

1. **Testing**: Rust compilation, tests, and linting
2. **Building**: Linux x86_64 binaries
3. **Deployment**: Automatic deployment to all 3 servers
4. **Verification**: Post-deployment health checks

**Workflow triggers:**
- Push to `rustmaster` branch
- Pull requests
- Manual workflow dispatch

**Monitor deployments:** https://github.com/slertora/blcpgha/actions

## 📋 Troubleshooting

### Common Issues

**Services not starting:**
```bash
# Check logs
sudo journalctl -u blcpg-ha -f
sudo journalctl -u blcpg-gui -f

# Check permissions
sudo chown -R postgres:postgres /opt/blcpgha
```

**Port conflicts:**
```bash
# Check what's using ports
sudo netstat -tlnp | grep -E ":(8080|3000)"

# Kill conflicting processes
sudo fuser -k 8080/tcp
sudo fuser -k 3000/tcp
```

**API not responding:**
```bash
# Check service status
systemctl status blcpg-ha

# Test connectivity
curl -v http://localhost:8080/api/v1/health
```

### Log Locations
- **System logs**: `journalctl -u blcpg-ha` and `journalctl -u blcpg-gui`
- **Application logs**: `/opt/blcpgha/logs/`
- **PostgreSQL logs**: `/var/log/postgresql/`

## 📚 Documentation

- **[Deployment Guide](DEPLOYMENT.md)** - Complete deployment instructions
- **[Server Connections](connections.md)** - Server access and configuration
- **[Roadmap](roadmap.md)** - Development roadmap and phases
- **[Requirements](docs/requirements.md)** - Technical requirements

## 🎯 Development Status

### ✅ Completed
- **Core Agent (blcpg-ha)**: 100% - Raft, APIs, health checks
- **CLI Tool (blcpg-cli)**: 100% - All management commands
- **Web GUI (blcpg-gui)**: 100% - Dynamic, real-time interface
- **GitHub Actions**: 100% - Automated CI/CD pipeline
- **Production Deployment**: 100% - 3 servers configured

### 🔄 Current Phase
- **Testing & Optimization**: Performance tuning and stability
- **Real Operations**: Connecting to actual PostgreSQL operations
- **Monitoring**: Enhanced metrics and alerting

### 🎯 Next Steps
- Advanced replica management
- Enterprise cloud integration
- Kubernetes operators
- Performance optimization

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and test: `./scripts/dev-workflow.sh quick-test`
4. Commit: `git commit -m 'Add amazing feature'`
5. Push: `git push origin feature/amazing-feature`
6. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏢 Production Infrastructure

**Servers:** 3 Ubuntu Linux servers with PostgreSQL HA cluster  
**Monitoring:** Real-time health checks and metrics  
**Deployment:** Automated CI/CD with GitHub Actions  
**Access:** SSH key-based authentication  

---

**Built with ❤️ by the Binlogic Team**  
**Ready for Production • Fully Automated • Enterprise Grade**