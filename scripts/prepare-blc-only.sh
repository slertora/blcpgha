#!/bin/bash

# BLC PostgreSQL HA - Minimal Server Preparation Script
# Only prepares what's needed for BLC HA (PostgreSQL already running)

set -e

# Server configurations (compatible with older bash versions)
SERVERS_postgre1="187.33.155.182"
SERVERS_postgresql2="187.33.144.18"
SERVERS_postgresql3="187.33.147.49"

get_server_ip() {
    local server_name=$1
    case $server_name in
        "postgre1") echo "$SERVERS_postgre1" ;;
        "postgresql2") echo "$SERVERS_postgresql2" ;;
        "postgresql3") echo "$SERVERS_postgresql3" ;;
        *) echo "" ;;
    esac
}

get_all_servers() {
    echo "postgre1 postgresql2 postgresql3"
}

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

prepare_server() {
    local server_name=$1
    local server_ip=$(get_server_ip "$server_name")
    
    log "Preparing BLC HA on $server_name ($server_ip)..."
    
    # Create minimal preparation script
    cat > /tmp/prepare-blc-${server_name}.sh << 'EOF'
#!/bin/bash
set -e

echo "🚀 Preparing BLC PostgreSQL HA (minimal setup)..."

# Only install essential dependencies (not PostgreSQL!)
echo "📦 Installing minimal system dependencies..."
apt-get update -qq
apt-get install -y -qq \
    curl \
    jq \
    net-tools \
    systemd \
    ufw

# Create postgres user if not exists (for running BLC services)
echo "👤 Ensuring postgres user exists..."
id postgres >/dev/null 2>&1 || useradd -r -s /bin/bash postgres
usermod -d /var/lib/postgresql postgres 2>/dev/null || true

# Create BLC directories
echo "📁 Creating BLC directories..."
mkdir -p /opt/blcpgha/{bin,config,logs,scripts}
chown -R postgres:postgres /opt/blcpgha

# Configure firewall for BLC services only
echo "🔥 Configuring firewall for BLC services..."
ufw allow 8080/tcp    # blcpg-ha API
ufw allow 3000/tcp    # blcpg-gui Web Interface
ufw allow 7000:7010/tcp  # Raft consensus ports
ufw --force enable 2>/dev/null || echo "UFW already configured"

# Verify PostgreSQL is running (don't install, just check)
echo "🐘 Checking existing PostgreSQL..."
if systemctl is-active postgresql >/dev/null 2>&1; then
    echo "   ✅ PostgreSQL is running (good!)"
    sudo -u postgres psql -c "SELECT version();" | head -3 | tail -1 || echo "   ⚠️  Cannot connect to PostgreSQL"
else
    echo "   ⚠️  PostgreSQL not running - you may need to start it"
fi

# Create health check script
cat > /opt/blcpgha/scripts/server-health.sh << 'HEALTH'
#!/bin/bash

echo "🏥 BLC PostgreSQL HA Server Health Check"
echo "========================================"

# System info
echo "🖥️  System Information:"
echo "   OS: $(lsb_release -d 2>/dev/null | cut -f2 || echo 'Unknown')"
echo "   Uptime: $(uptime -p 2>/dev/null || echo 'Unknown')"
echo "   Load: $(cat /proc/loadavg 2>/dev/null | cut -d' ' -f1-3 || echo 'Unknown')"

# Memory and disk
echo ""
echo "💾 Resources:"
echo "   Memory: $(free -h 2>/dev/null | grep '^Mem:' | awk '{print $3 "/" $2}' || echo 'Unknown')"
echo "   Disk: $(df -h / 2>/dev/null | tail -1 | awk '{print $3 "/" $2 " (" $5 " used)"}' || echo 'Unknown')"

# Network
echo ""
echo "🌐 Network:"
ip addr show 2>/dev/null | grep -E "inet [0-9]" | grep -v 127.0.0.1 | awk '{print "   " $2}' || echo "   Unknown"

# Services
echo ""
echo "🔧 Services:"
systemctl is-active postgresql >/dev/null 2>&1 && echo "   PostgreSQL: ✅ Running" || echo "   PostgreSQL: ❌ Stopped"
systemctl is-active blcpg-ha >/dev/null 2>&1 && echo "   blcpg-ha: ✅ Running" || echo "   blcpg-ha: ⏸️  Not installed/stopped"
systemctl is-active blcpg-gui >/dev/null 2>&1 && echo "   blcpg-gui: ✅ Running" || echo "   blcpg-gui: ⏸️  Not installed/stopped"

# Ports
echo ""
echo "🔌 Listening Ports:"
netstat -tlnp 2>/dev/null | grep -E ":(22|5432|8080|3000|700[0-9])" | awk '{print "   " $1 " " $4}' | sort || echo "   netstat not available"

# PostgreSQL status (existing installation)
echo ""
echo "🐘 PostgreSQL (existing):"
if systemctl is-active postgresql >/dev/null 2>&1; then
    sudo -u postgres psql -c "SELECT current_database(), current_user, inet_server_addr(), inet_server_port();" 2>/dev/null || echo "   ⚠️  Cannot query PostgreSQL"
    echo "   📊 Database count: $(sudo -u postgres psql -t -c "SELECT count(*) FROM pg_database;" 2>/dev/null | tr -d ' ' || echo 'Unknown')"
else
    echo "   ❌ PostgreSQL not running"
fi

echo ""
echo "✅ Health check completed!"
HEALTH

chmod +x /opt/blcpgha/scripts/server-health.sh
chown postgres:postgres /opt/blcpgha/scripts/server-health.sh

echo ""
echo "✅ BLC HA preparation completed successfully!"
echo ""
echo "🎯 What was done:"
echo "   ✅ Created BLC directories: /opt/blcpgha/"
echo "   ✅ Configured firewall for BLC ports"
echo "   ✅ Verified PostgreSQL is running"
echo "   ✅ Created health check script"
echo ""
echo "🔍 Run health check: /opt/blcpgha/scripts/server-health.sh"
echo "📦 Ready for BLC HA deployment!"
EOF

    # Copy and execute preparation script
    scp -i $SSH_KEY /tmp/prepare-blc-${server_name}.sh $SSH_USER@$server_ip:/tmp/prepare-blc.sh
    ssh -i $SSH_KEY $SSH_USER@$server_ip 'chmod +x /tmp/prepare-blc.sh && /tmp/prepare-blc.sh && rm /tmp/prepare-blc.sh'
    
    # Cleanup local temp file
    rm /tmp/prepare-blc-${server_name}.sh
    
    success "Server $server_name prepared successfully!"
}

check_postgresql_status() {
    local server_name=$1
    local server_ip=$(get_server_ip "$server_name")
    
    log "Checking PostgreSQL status on $server_name ($server_ip)..."
    
    if ssh -i $SSH_KEY $SSH_USER@$server_ip 'systemctl is-active postgresql >/dev/null 2>&1'; then
        success "✅ PostgreSQL running on $server_name"
        
        # Get more details
        local pg_version=$(ssh -i $SSH_KEY $SSH_USER@$server_ip 'sudo -u postgres psql -t -c "SELECT version();" 2>/dev/null | head -1' || echo "Unknown")
        local db_count=$(ssh -i $SSH_KEY $SSH_USER@$server_ip 'sudo -u postgres psql -t -c "SELECT count(*) FROM pg_database;" 2>/dev/null | tr -d " "' || echo "Unknown")
        
        info "   Version: $pg_version"
        info "   Databases: $db_count"
    else
        warning "⚠️  PostgreSQL not running on $server_name"
    fi
}

show_usage() {
    cat << EOF
BLC PostgreSQL HA - Minimal Server Preparation Script
(For servers that already have PostgreSQL running)

Usage: $0 [action] [server]

Actions:
  prepare [server]  - Prepare server(s) for BLC HA (minimal setup)
  check [server]    - Check PostgreSQL status on server(s)
  health [server]   - Run health checks on server(s)

Servers:
  postgre1     - Master server (187.33.155.182)
  postgresql2  - Replica 1 (187.33.144.18)
  postgresql3  - Replica 2 (187.33.147.49)
  all          - All servers

Examples:
  $0 prepare all      # Prepare all servers (minimal)
  $0 check all        # Check PostgreSQL status on all
  $0 health all       # Run health checks on all

EOF
}

main() {
    local action=${1:-"help"}
    local target=${2:-"all"}
    
    case $action in
        "prepare")
            if [ "$target" == "all" ]; then
                log "Preparing all servers (minimal BLC setup)..."
                for server_name in $(get_all_servers); do
                    prepare_server "$server_name"
                    echo ""
                done
                success "All servers prepared successfully!"
            elif [ -n "$(get_server_ip "$target")" ]; then
                prepare_server "$target"
            else
                error "Unknown server: $target"
                exit 1
            fi
            ;;
            
        "check")
            if [ "$target" == "all" ]; then
                log "Checking PostgreSQL on all servers..."
                for server_name in $(get_all_servers); do
                    check_postgresql_status "$server_name"
                done
            elif [ -n "$(get_server_ip "$target")" ]; then
                check_postgresql_status "$target"
            else
                error "Unknown server: $target"
                exit 1
            fi
            ;;
            
        "health")
            if [ "$target" == "all" ]; then
                log "Running health checks on all servers..."
                for server_name in $(get_all_servers); do
                    local server_ip=$(get_server_ip "$server_name")
                    log "Health check for $server_name ($server_ip):"
                    ssh -i $SSH_KEY $SSH_USER@$server_ip '/opt/blcpgha/scripts/server-health.sh' || warning "Health check failed for $server_name"
                    echo ""
                done
            elif [ -n "$(get_server_ip "$target")" ]; then
                local server_ip=$(get_server_ip "$target")
                log "Health check for $target ($server_ip):"
                ssh -i $SSH_KEY $SSH_USER@$server_ip '/opt/blcpgha/scripts/server-health.sh'
            else
                error "Unknown server: $target"
                exit 1
            fi
            ;;
            
        *)
            show_usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
