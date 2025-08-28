#!/bin/bash

# BLC PostgreSQL HA - Manual Deployment Script
# Usage: ./scripts/deploy-manual.sh [server] [action]
# 
# Examples:
#   ./scripts/deploy-manual.sh all install    # Install on all servers
#   ./scripts/deploy-manual.sh all update     # Update all servers
#   ./scripts/deploy-manual.sh postgre1 install  # Install on master only
#   ./scripts/deploy-manual.sh postgresql2 health # Check health of replica 1

set -e

# Server configurations
declare -A SERVERS
SERVERS[postgre1]="187.33.155.182"
SERVERS[postgresql2]="187.33.144.18"
SERVERS[postgresql3]="187.33.147.49"

SSH_KEY="~/.ssh/binlogic"
SSH_USER="root"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

build_project() {
    log "Building project for Linux x86_64..."
    
    if ! command -v cargo &> /dev/null; then
        error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Build release binaries
    cargo build --release --target x86_64-unknown-linux-gnu
    
    # Create deployment package
    log "Creating deployment package..."
    rm -rf deploy/
    mkdir -p deploy/{bin,config,scripts}
    
    # Copy binaries
    cp target/x86_64-unknown-linux-gnu/release/blcpg-ha deploy/bin/
    cp target/x86_64-unknown-linux-gnu/release/blcpg-cli deploy/bin/
    cp target/x86_64-unknown-linux-gnu/release/blcpg-gui deploy/bin/
    
    # Copy config files
    cp blcpg-ha/config.toml deploy/config/blcpg-ha.toml
    cp blcpg-cli/config.toml deploy/config/blcpg-cli.toml
    cp blcpg-gui/config.toml deploy/config/blcpg-gui.toml
    
    # Create install script
    cat > deploy/scripts/install.sh << 'EOF'
#!/bin/bash
set -e

echo "Installing BLC PostgreSQL HA..."

# Create directories
sudo mkdir -p /opt/blcpgha/{bin,config,logs}

# Copy binaries
sudo cp bin/* /opt/blcpgha/bin/
sudo chmod +x /opt/blcpgha/bin/*

# Copy config files
sudo cp config/* /opt/blcpgha/config/

# Create systemd services
sudo tee /etc/systemd/system/blcpg-ha.service > /dev/null << 'SYSTEMD'
[Unit]
Description=BLC PostgreSQL HA Agent
After=network.target postgresql.service

[Service]
Type=simple
User=postgres
Group=postgres
ExecStart=/opt/blcpgha/bin/blcpg-ha --config /opt/blcpgha/config/blcpg-ha.toml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SYSTEMD

sudo tee /etc/systemd/system/blcpg-gui.service > /dev/null << 'SYSTEMD'
[Unit]
Description=BLC PostgreSQL HA GUI
After=network.target blcpg-ha.service

[Service]
Type=simple
User=postgres
Group=postgres
ExecStart=/opt/blcpgha/bin/blcpg-gui --config /opt/blcpgha/config/blcpg-gui.toml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SYSTEMD

# Reload systemd and enable services
sudo systemctl daemon-reload
sudo systemctl enable blcpg-ha blcpg-gui

# Set permissions
sudo chown -R postgres:postgres /opt/blcpgha

echo "Installation completed!"
echo "Start services with:"
echo "  sudo systemctl start blcpg-ha"
echo "  sudo systemctl start blcpg-gui"
EOF
    
    chmod +x deploy/scripts/install.sh
    
    # Create update script
    cat > deploy/scripts/update.sh << 'EOF'
#!/bin/bash
set -e

echo "Updating BLC PostgreSQL HA..."

# Stop services
sudo systemctl stop blcpg-gui blcpg-ha || true

# Backup current binaries
sudo cp /opt/blcpgha/bin/blcpg-ha /opt/blcpgha/bin/blcpg-ha.backup.$(date +%Y%m%d-%H%M%S) || true
sudo cp /opt/blcpgha/bin/blcpg-gui /opt/blcpgha/bin/blcpg-gui.backup.$(date +%Y%m%d-%H%M%S) || true
sudo cp /opt/blcpgha/bin/blcpg-cli /opt/blcpgha/bin/blcpg-cli.backup.$(date +%Y%m%d-%H%M%S) || true

# Update binaries
sudo cp bin/* /opt/blcpgha/bin/
sudo chmod +x /opt/blcpgha/bin/*

# Set permissions
sudo chown -R postgres:postgres /opt/blcpgha

# Start services
sudo systemctl start blcpg-ha
sleep 5
sudo systemctl start blcpg-gui

# Check status
sudo systemctl status blcpg-ha --no-pager
sudo systemctl status blcpg-gui --no-pager

echo "Update completed!"
EOF
    
    chmod +x deploy/scripts/update.sh
    
    # Create health check script
    cat > deploy/scripts/health-check.sh << 'EOF'
#!/bin/bash

echo "BLC PostgreSQL HA Health Check"
echo "================================"

# Check services
echo "Service Status:"
sudo systemctl is-active blcpg-ha || echo "blcpg-ha: INACTIVE"
sudo systemctl is-active blcpg-gui || echo "blcpg-gui: INACTIVE"

# Check ports
echo -e "\nPort Status:"
netstat -tlnp 2>/dev/null | grep -E ":(8080|3000)" || echo "No services listening on expected ports"

# Check API health
echo -e "\nAPI Health:"
curl -s -m 5 http://localhost:8080/api/v1/health 2>/dev/null | jq . || echo "API health check failed"

# Check logs for errors
echo -e "\nRecent Errors:"
sudo journalctl -u blcpg-ha --since "5 minutes ago" --grep ERROR || echo "No recent errors in blcpg-ha"
sudo journalctl -u blcpg-gui --since "5 minutes ago" --grep ERROR || echo "No recent errors in blcpg-gui"
EOF
    
    chmod +x deploy/scripts/health-check.sh
    
    # Create package
    tar -czf blcpgha-deploy.tar.gz -C deploy .
    
    success "Deployment package created: blcpgha-deploy.tar.gz"
}

deploy_to_server() {
    local server_name=$1
    local action=$2
    local server_ip=${SERVERS[$server_name]}
    
    if [ -z "$server_ip" ]; then
        error "Unknown server: $server_name"
        return 1
    fi
    
    log "Deploying to $server_name ($server_ip) - Action: $action"
    
    case $action in
        "install"|"update")
            # Upload deployment package
            log "Uploading deployment package..."
            ssh -i $SSH_KEY $SSH_USER@$server_ip 'mkdir -p /tmp/blcpgha-deploy'
            scp -i $SSH_KEY blcpgha-deploy.tar.gz $SSH_USER@$server_ip:/tmp/blcpgha-deploy/
            
            # Extract and run action
            ssh -i $SSH_KEY $SSH_USER@$server_ip "
                cd /tmp/blcpgha-deploy
                tar -xzf blcpgha-deploy.tar.gz
                ./scripts/$action.sh
                cd /
                rm -rf /tmp/blcpgha-deploy
            "
            
            success "Deployment to $server_name completed!"
            ;;
            
        "health")
            log "Running health check on $server_name..."
            ssh -i $SSH_KEY $SSH_USER@$server_ip '
                if [ -f "/opt/blcpgha/scripts/health-check.sh" ]; then
                    /opt/blcpgha/scripts/health-check.sh
                else
                    echo "Health check script not found. Service may not be installed."
                fi
            '
            ;;
            
        "status")
            log "Checking service status on $server_name..."
            ssh -i $SSH_KEY $SSH_USER@$server_ip '
                echo "=== Service Status ==="
                sudo systemctl status blcpg-ha --no-pager || echo "blcpg-ha not found"
                echo ""
                sudo systemctl status blcpg-gui --no-pager || echo "blcpg-gui not found"
            '
            ;;
            
        "logs")
            log "Showing recent logs from $server_name..."
            ssh -i $SSH_KEY $SSH_USER@$server_ip '
                echo "=== blcpg-ha logs ==="
                sudo journalctl -u blcpg-ha --since "1 hour ago" --no-pager | tail -20
                echo ""
                echo "=== blcpg-gui logs ==="
                sudo journalctl -u blcpg-gui --since "1 hour ago" --no-pager | tail -20
            '
            ;;
            
        "restart")
            log "Restarting services on $server_name..."
            ssh -i $SSH_KEY $SSH_USER@$server_ip '
                sudo systemctl restart blcpg-ha
                sleep 5
                sudo systemctl restart blcpg-gui
                echo "Services restarted"
            '
            ;;
            
        *)
            error "Unknown action: $action"
            return 1
            ;;
    esac
}

show_usage() {
    cat << EOF
BLC PostgreSQL HA - Manual Deployment Script

Usage: $0 [server] [action]

Servers:
  all          - Deploy to all servers
  postgre1     - Master server (187.33.155.182)
  postgresql2  - Replica 1 (187.33.144.18)
  postgresql3  - Replica 2 (187.33.147.49)

Actions:
  install      - Install BLC PostgreSQL HA (first time)
  update       - Update existing installation
  health       - Run health check
  status       - Show service status
  logs         - Show recent logs
  restart      - Restart services

Examples:
  $0 all install           # Install on all servers
  $0 all update            # Update all servers
  $0 postgre1 health       # Check health of master
  $0 postgresql2 logs      # Show logs from replica 1
  $0 all status            # Check status on all servers

EOF
}

main() {
    local server=$1
    local action=$2
    
    if [ -z "$server" ] || [ -z "$action" ]; then
        show_usage
        exit 1
    fi
    
    # Check if deployment package exists for install/update actions
    if [[ "$action" == "install" || "$action" == "update" ]]; then
        if [ ! -f "blcpgha-deploy.tar.gz" ]; then
            warning "Deployment package not found. Building..."
            build_project
        fi
    fi
    
    if [ "$server" == "all" ]; then
        log "Deploying to all servers..."
        for server_name in "${!SERVERS[@]}"; do
            deploy_to_server "$server_name" "$action"
            echo ""
        done
        success "All deployments completed!"
    else
        deploy_to_server "$server" "$action"
    fi
}

# Run main function with all arguments
main "$@"
