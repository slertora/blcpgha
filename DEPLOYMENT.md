# BLC PostgreSQL HA - Deployment Guide

## 🚀 **GitHub Actions Automated Deployment**

### **Repository Setup**

1. **Add the GitHub remote:**
   ```bash
   git remote add slertora git@slertora-github:slertora/blcpgha.git
   ```

2. **Push to GitHub:**
   ```bash
   git push slertora rustmaster
   ```

### **GitHub Secrets Configuration**

Go to your GitHub repository settings and add these secrets:

#### **Required Secrets:**

| Secret Name | Description | Value |
|-------------|-------------|-------|
| `DEPLOY_SSH_KEY` | SSH private key for server access | Content of `~/.ssh/binlogic` |

#### **Setting up the SSH Key Secret:**

1. Copy your SSH private key:
   ```bash
   cat ~/.ssh/binlogic
   ```

2. In GitHub:
   - Go to **Settings** → **Secrets and variables** → **Actions**
   - Click **New repository secret**
   - Name: `DEPLOY_SSH_KEY`
   - Value: Paste the entire private key content

### **Automated Workflow**

The GitHub Actions workflow (`.github/workflows/ci-cd.yml`) will:

1. **Test**: Run tests and linting
2. **Build**: Compile for Linux x86_64
3. **Deploy**: Automatically deploy to all 3 servers on push to `rustmaster`

#### **Workflow Triggers:**
- ✅ Push to `rustmaster` branch
- ✅ Pull requests to `rustmaster`
- ✅ Manual workflow dispatch

---

## 🔧 **Manual Deployment**

For manual deployments or testing, use the provided script:

### **Quick Commands:**

```bash
# Deploy to all servers (first time)
./scripts/deploy-manual.sh all install

# Update all servers
./scripts/deploy-manual.sh all update

# Check health of all servers
./scripts/deploy-manual.sh all health

# Check status of master server
./scripts/deploy-manual.sh postgre1 status

# View logs from replica 1
./scripts/deploy-manual.sh postgresql2 logs

# Restart services on all servers
./scripts/deploy-manual.sh all restart
```

### **Individual Server Commands:**

```bash
# Master Server (postgre1 - 187.33.155.182)
./scripts/deploy-manual.sh postgre1 install
./scripts/deploy-manual.sh postgre1 update
./scripts/deploy-manual.sh postgre1 health

# Replica 1 (postgresql2 - 187.33.144.18)
./scripts/deploy-manual.sh postgresql2 install
./scripts/deploy-manual.sh postgresql2 update
./scripts/deploy-manual.sh postgresql2 health

# Replica 2 (postgresql3 - 187.33.147.49)
./scripts/deploy-manual.sh postgresql3 install
./scripts/deploy-manual.sh postgresql3 update
./scripts/deploy-manual.sh postgresql3 health
```

---

## 🖥️ **Server Preparation**

### **Prerequisites on Each Server:**

1. **System Dependencies:**
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev libpq-dev postgresql postgresql-contrib curl jq netstat-nat
   
   # Create postgres user if not exists
   sudo useradd -r -s /bin/bash postgres || true
   ```

2. **Directory Structure:**
   ```bash
   sudo mkdir -p /opt/blcpgha/{bin,config,logs}
   sudo chown -R postgres:postgres /opt/blcpgha
   ```

3. **Firewall Configuration:**
   ```bash
   # Allow required ports
   sudo ufw allow 8080/tcp   # blcpg-ha API
   sudo ufw allow 3000/tcp   # blcpg-gui Web Interface
   sudo ufw allow 5432/tcp   # PostgreSQL
   ```

### **First-Time Setup Script:**

Create this script on each server as `/tmp/prepare-server.sh`:

```bash
#!/bin/bash
set -e

echo "Preparing server for BLC PostgreSQL HA..."

# Install system dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev libpq-dev postgresql postgresql-contrib curl jq net-tools

# Create directories
sudo mkdir -p /opt/blcpgha/{bin,config,logs}

# Create postgres user if not exists
sudo useradd -r -s /bin/bash postgres || true
sudo chown -R postgres:postgres /opt/blcpgha

# Configure firewall
sudo ufw allow 8080/tcp
sudo ufw allow 3000/tcp
sudo ufw allow 5432/tcp

# Start PostgreSQL
sudo systemctl enable postgresql
sudo systemctl start postgresql

echo "Server preparation completed!"
```

Run on each server:
```bash
chmod +x /tmp/prepare-server.sh
sudo /tmp/prepare-server.sh
```

---

## 📊 **Post-Deployment Verification**

### **Service Status Check:**

```bash
# On each server, check services
sudo systemctl status blcpg-ha
sudo systemctl status blcpg-gui

# Check if services are listening
netstat -tlnp | grep -E ":(8080|3000)"
```

### **API Health Checks:**

```bash
# Master Server
curl http://187.33.155.182:8080/api/v1/health

# Replica 1
curl http://187.33.144.18:8080/api/v1/health

# Replica 2
curl http://187.33.147.49:8080/api/v1/health
```

### **Web Interface Access:**

- **Master**: http://187.33.155.182:3000
- **Replica 1**: http://187.33.144.18:3000
- **Replica 2**: http://187.33.147.49:3000

---

## 🔍 **Troubleshooting**

### **Common Issues:**

1. **Service won't start:**
   ```bash
   # Check logs
   sudo journalctl -u blcpg-ha -f
   sudo journalctl -u blcpg-gui -f
   
   # Check permissions
   sudo chown -R postgres:postgres /opt/blcpgha
   ```

2. **Port conflicts:**
   ```bash
   # Check what's using the ports
   sudo netstat -tlnp | grep -E ":(8080|3000)"
   
   # Kill conflicting processes if needed
   sudo fuser -k 8080/tcp
   sudo fuser -k 3000/tcp
   ```

3. **PostgreSQL connection issues:**
   ```bash
   # Check PostgreSQL status
   sudo systemctl status postgresql
   
   # Check PostgreSQL logs
   sudo tail -f /var/log/postgresql/postgresql-*.log
   ```

### **Log Locations:**

- **System logs**: `journalctl -u blcpg-ha` and `journalctl -u blcpg-gui`
- **Application logs**: `/opt/blcpgha/logs/`
- **PostgreSQL logs**: `/var/log/postgresql/`

### **Manual Service Management:**

```bash
# Start services
sudo systemctl start blcpg-ha
sudo systemctl start blcpg-gui

# Stop services
sudo systemctl stop blcpg-gui
sudo systemctl stop blcpg-ha

# Restart services
sudo systemctl restart blcpg-ha
sudo systemctl restart blcpg-gui

# Enable auto-start
sudo systemctl enable blcpg-ha
sudo systemctl enable blcpg-gui
```

---

## 🎯 **Deployment Checklist**

### **Pre-Deployment:**
- [ ] GitHub repository configured with correct remotes
- [ ] SSH keys added to GitHub Secrets
- [ ] All servers prepared with dependencies
- [ ] PostgreSQL installed and running on all servers
- [ ] Firewall configured on all servers

### **Deployment:**
- [ ] Code pushed to `rustmaster` branch
- [ ] GitHub Actions workflow completed successfully
- [ ] All services started on all servers
- [ ] Health checks passing

### **Post-Deployment:**
- [ ] Web interfaces accessible
- [ ] API endpoints responding
- [ ] Logs clean of errors
- [ ] Services enabled for auto-start
- [ ] Cluster communication working

---

## 📝 **Environment Configuration**

### **Production Environment Variables:**

Create `/opt/blcpgha/config/.env` on each server:

```bash
# blcpg-ha configuration
BLCPG_SERVER_HOST=0.0.0.0
BLCPG_SERVER_PORT=8080
BLCPG_LOG_LEVEL=info

# blcpg-gui configuration  
BLCGUI_SERVER_HOST=0.0.0.0
BLCGUI_SERVER_PORT=3000
BLCGUI_API_BASE_URL=http://localhost:8080/api/v1

# PostgreSQL configuration
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=postgres
POSTGRES_USER=postgres
```

### **Server-Specific Configuration:**

Each server should have its role configured in the config files:

- **postgre1 (Master)**: `role = "primary"`
- **postgresql2 (Replica)**: `role = "replica"`
- **postgresql3 (Replica)**: `role = "replica"`

---

**Last Updated:** August 12, 2025  
**Status:** Ready for deployment
