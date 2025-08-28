# BLC PostgreSQL HA - Server Connections

## 🖥️ **Production Servers**

### **PostgreSQL Cluster Configuration:**

| Server | Role | IP Address | SSH Access |
|--------|------|------------|------------|
| `postgre1` | **MASTER** (Primary) | `187.33.155.182` | `ssh -i ~/.ssh/binlogic root@187.33.155.182` |
| `postgresql2` | **SLAVE** (Replica) | `187.33.144.18` | `ssh -i ~/.ssh/binlogic root@187.33.144.18` |
| `postgresql3` | **SLAVE** (Replica) | `187.33.147.49` | `ssh -i ~/.ssh/binlogic root@187.33.147.49` |

### **Server Details:**
- **OS**: Ubuntu Linux
- **SSH Key**: `~/.ssh/binlogic`
- **User**: `root`
- **Architecture**: x86_64 (assumed)

---

## 🔐 **SSH Configuration**

### **Local SSH Config (`~/.ssh/config`):**

```
# GitHub Access
Host slertora-github
  HostName github.com
  User git
  IdentityFile ~/.ssh/slertora-github

# PostgreSQL Production Servers
Host postgre1
  HostName 187.33.155.182
  User root
  IdentityFile ~/.ssh/binlogic

Host postgresql2
  HostName 187.33.144.18
  User root
  IdentityFile ~/.ssh/binlogic

Host postgresql3
  HostName 187.33.147.49
  User root
  IdentityFile ~/.ssh/binlogic
```

### **Quick SSH Commands:**
```bash
# Master Server
ssh postgre1

# Replica Servers
ssh postgresql2
ssh postgresql3
```

---

## 📦 **Deployment Strategy**

### **GitHub Repository:**
- **URL**: `git@github.com:slertora/blcpgha.git`
- **Branch**: `rustmaster`
- **SSH Host**: `slertora-github`

### **Deployment Flow:**
1. **GitHub Actions** compiles for Linux x86_64
2. **Automated deployment** to all 3 servers
3. **Service management** with systemd
4. **Health checks** post-deployment

### **Target Directories on Servers:**
```
/opt/blcpgha/
├── bin/
│   ├── blcpg-ha
│   ├── blcpg-cli
│   └── blcpg-gui
├── config/
│   └── config.toml
└── logs/
```

---

## 🚀 **GitHub Actions Workflow**

### **Triggers:**
- Push to `rustmaster` branch
- Manual dispatch
- Release tags

### **Jobs:**
1. **Build**: Compile for Linux x86_64
2. **Test**: Run test suite
3. **Deploy**: Deploy to all 3 servers
4. **Verify**: Health checks post-deployment

---

## 📝 **Server Preparation Checklist**

### **Each Server Needs:**
- [ ] Rust toolchain installed
- [ ] PostgreSQL installed and configured
- [ ] System dependencies (build-essential, etc.)
- [ ] Systemd service files
- [ ] Log rotation configured
- [ ] Firewall rules configured
- [ ] User permissions set

### **Network Configuration:**
- [ ] Port 8080 (blcpg-ha API)
- [ ] Port 3000 (blcpg-gui Web Interface)
- [ ] PostgreSQL ports (5432)
- [ ] Raft consensus ports
- [ ] VIP management

---

## ⚙️ **Environment Variables**

### **Production Configuration:**
```bash
# blcpg-ha
BLCPG_SERVER_HOST=0.0.0.0
BLCPG_SERVER_PORT=8080
BLCPG_LOG_LEVEL=info

# blcpg-gui
BLCGUI_SERVER_HOST=0.0.0.0
BLCGUI_SERVER_PORT=3000
BLCGUI_API_BASE_URL=http://localhost:8080/api/v1
```

---

## 🔍 **Monitoring & Logs**

### **Log Locations:**
- **blcpg-ha**: `/opt/blcpgha/logs/blcpg-ha.log`
- **blcpg-gui**: `/opt/blcpgha/logs/blcpg-gui.log`
- **System logs**: `journalctl -u blcpg-ha`

### **Health Check Endpoints:**
- **Master**: `http://187.33.155.182:8080/api/v1/health`
- **Replica 1**: `http://187.33.144.18:8080/api/v1/health`
- **Replica 2**: `http://187.33.147.49:8080/api/v1/health`

---

## 📋 **Deployment Commands**

### **Manual Deployment:**
```bash
# Build locally
cargo build --release --target x86_64-unknown-linux-gnu

# Copy to servers
scp target/x86_64-unknown-linux-gnu/release/blcpg-ha postgre1:/opt/blcpgha/bin/
scp target/x86_64-unknown-linux-gnu/release/blcpg-cli postgre1:/opt/blcpgha/bin/
scp target/x86_64-unknown-linux-gnu/release/blcpg-gui postgre1:/opt/blcpgha/bin/

# Restart services
ssh postgre1 'systemctl restart blcpg-ha'
ssh postgre1 'systemctl restart blcpg-gui'
```

### **Service Management:**
```bash
# Check status
ssh postgre1 'systemctl status blcpg-ha'

# View logs
ssh postgre1 'journalctl -u blcpg-ha -f'

# Restart service
ssh postgre1 'systemctl restart blcpg-ha'
```

---

## 🎯 **Next Steps**

1. **Configure GitHub repository** with proper remotes
2. **Set up GitHub Actions** for automated CI/CD
3. **Prepare servers** with required dependencies
4. **Deploy and test** the complete solution
5. **Monitor and optimize** performance

---

**Last Updated:** August 12, 2025  
**Status:** Ready for GitHub Actions setup
