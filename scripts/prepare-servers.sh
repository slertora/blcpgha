#!/bin/bash

# BLC PostgreSQL HA - Server Preparation Script
# This script prepares all 3 production servers for deployment

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
    
    log "Preparing server $server_name ($server_ip)..."
    
    # Create preparation script
    cat > /tmp/prepare-${server_name}.sh << 'EOF'
#!/bin/bash
set -e

echo "🚀 Preparing server for BLC PostgreSQL HA..."

# Update system
echo "📦 Updating system packages..."
apt-get update

# Install system dependencies
echo "📦 Installing system dependencies..."
apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    postgresql \
    postgresql-contrib \
    curl \
    jq \
    net-tools \
    ufw \
    systemd

# Create postgres user if not exists
echo "👤 Setting up postgres user..."
useradd -r -s /bin/bash postgres || true
usermod -d /var/lib/postgresql postgres || true

# Create BLC directories
echo "📁 Creating BLC directories..."
mkdir -p /opt/blcpgha/{bin,config,logs,scripts}
chown -R postgres:postgres /opt/blcpgha

# Configure firewall
echo "🔥 Configuring firewall..."
ufw allow 22/tcp      # SSH
ufw allow 8080/tcp    # blcpg-ha API
ufw allow 3000/tcp    # blcpg-gui Web Interface
ufw allow 5432/tcp    # PostgreSQL
ufw allow 7000:7010/tcp  # Raft consensus ports
ufw --force enable

# Start and enable PostgreSQL
echo "🐘 Configuring PostgreSQL..."
systemctl enable postgresql
systemctl start postgresql

# Configure PostgreSQL for replication
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';" || true

# Create basic PostgreSQL configuration for HA
cat > /etc/postgresql/*/main/postgresql.conf.blc << 'PGCONF'
# BLC PostgreSQL HA Configuration
listen_addresses = '*'
port = 5432
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200

# Replication settings
wal_level = replica
max_wal_senders = 10
max_replication_slots = 10
hot_standby = on
hot_standby_feedback = on
PGCONF

# Configure pg_hba.conf for replication
cat >> /etc/postgresql/*/main/pg_hba.conf << 'PGHBA'

# BLC PostgreSQL HA Replication
host    replication     postgres        187.33.155.182/32       md5
host    replication     postgres        187.33.144.18/32        md5
host    replication     postgres        187.33.147.49/32         md5
host    all             postgres        187.33.155.182/32       md5
host    all             postgres        187.33.144.18/32        md5
host    all             postgres        187.33.147.49/32         md5
PGHBA

# Restart PostgreSQL to apply configuration
systemctl restart postgresql

# Create health check script
cat > /opt/blcpgha/scripts/server-health.sh << 'HEALTH'
#!/bin/bash

echo "🏥 BLC PostgreSQL HA Server Health Check"
echo "========================================"

# System info
echo "🖥️  System Information:"
echo "   OS: $(lsb_release -d | cut -f2)"
echo "   Kernel: $(uname -r)"
echo "   Uptime: $(uptime -p)"
echo "   Load: $(cat /proc/loadavg | cut -d' ' -f1-3)"

# Memory and disk
echo ""
echo "💾 Resources:"
echo "   Memory: $(free -h | grep '^Mem:' | awk '{print $3 "/" $2}')"
echo "   Disk: $(df -h / | tail -1 | awk '{print $3 "/" $2 " (" $5 " used)"}')"

# Network
echo ""
echo "🌐 Network:"
ip addr show | grep -E "inet [0-9]" | grep -v 127.0.0.1 | awk '{print "   " $2}'

# Services
echo ""
echo "🔧 Services:"
systemctl is-active postgresql && echo "   PostgreSQL: ✅ Running" || echo "   PostgreSQL: ❌ Stopped"
systemctl is-active blcpg-ha 2>/dev/null && echo "   blcpg-ha: ✅ Running" || echo "   blcpg-ha: ⏸️  Not installed/stopped"
systemctl is-active blcpg-gui 2>/dev/null && echo "   blcpg-gui: ✅ Running" || echo "   blcpg-gui: ⏸️  Not installed/stopped"

# Ports
echo ""
echo "🔌 Listening Ports:"
netstat -tlnp 2>/dev/null | grep -E ":(22|5432|8080|3000|700[0-9])" | awk '{print "   " $1 " " $4}' | sort

# PostgreSQL status
echo ""
echo "🐘 PostgreSQL:"
sudo -u postgres psql -c "SELECT version();" 2>/dev/null | head -3 | tail -1 || echo "   ❌ Cannot connect to PostgreSQL"
sudo -u postgres psql -c "SELECT current_database(), current_user, inet_server_addr(), inet_server_port();" 2>/dev/null || echo "   ❌ Cannot query PostgreSQL"

echo ""
echo "✅ Health check completed!"
HEALTH

chmod +x /opt/blcpgha/scripts/server-health.sh
chown postgres:postgres /opt/blcpgha/scripts/server-health.sh

echo ""
echo "✅ Server preparation completed successfully!"
echo ""
echo "🎯 Next steps:"
echo "   1. Deploy BLC PostgreSQL HA binaries"
echo "   2. Configure cluster settings"
echo "   3. Start BLC services"
echo ""
echo "🔍 Run health check: /opt/blcpgha/scripts/server-health.sh"
EOF

    # Copy and execute preparation script
    scp -i $SSH_KEY /tmp/prepare-${server_name}.sh $SSH_USER@$server_ip:/tmp/prepare-server.sh
    ssh -i $SSH_KEY $SSH_USER@$server_ip 'chmod +x /tmp/prepare-server.sh && /tmp/prepare-server.sh'
    
    # Cleanup local temp file
    rm /tmp/prepare-${server_name}.sh
    
    success "Server $server_name prepared successfully!"
}

test_server_connectivity() {
    local server_name=$1
    local server_ip=$(get_server_ip "$server_name")
    
    log "Testing connectivity to $server_name ($server_ip)..."
    
    if ssh -i $SSH_KEY -o ConnectTimeout=10 $SSH_USER@$server_ip 'echo "Connection successful"' >/dev/null 2>&1; then
        success "✅ $server_name is reachable"
        return 0
    else
        error "❌ Cannot connect to $server_name"
        return 1
    fi
}

run_health_checks() {
    log "Running health checks on all servers..."
    
    for server_name in $(get_all_servers); do
        local server_ip=$(get_server_ip "$server_name")
        
        log "Health check for $server_name ($server_ip):"
        ssh -i $SSH_KEY $SSH_USER@$server_ip '/opt/blcpgha/scripts/server-health.sh' || warning "Health check failed for $server_name"
        echo ""
    done
}

show_usage() {
    cat << EOF
BLC PostgreSQL HA - Server Preparation Script

Usage: $0 [action] [server]

Actions:
  prepare [server]  - Prepare server(s) with dependencies and configuration
  test [server]     - Test SSH connectivity to server(s)  
  health [server]   - Run health checks on server(s)
  all              - Run action on all servers

Servers:
  postgre1     - Master server (187.33.155.182)
  postgresql2  - Replica 1 (187.33.144.18)
  postgresql3  - Replica 2 (187.33.147.49)
  all          - All servers

Examples:
  $0 prepare all           # Prepare all servers
  $0 test all              # Test connectivity to all servers
  $0 health all            # Run health checks on all servers
  $0 prepare postgre1      # Prepare only master server

EOF
}

main() {
    local action=$1
    local target=$2
    
    if [ -z "$action" ]; then
        show_usage
        exit 1
    fi
    
    case $action in
        "prepare")
            if [ "$target" == "all" ] || [ -z "$target" ]; then
                log "Preparing all servers..."
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
            
        "test")
            if [ "$target" == "all" ] || [ -z "$target" ]; then
                log "Testing connectivity to all servers..."
                failed_servers=""
                for server_name in $(get_all_servers); do
                    if ! test_server_connectivity "$server_name"; then
                        failed_servers="$failed_servers $server_name"
                    fi
                done
                
                if [ -z "$failed_servers" ]; then
                    success "All servers are reachable!"
                else
                    error "Failed to connect to:$failed_servers"
                    exit 1
                fi
            elif [ -n "$(get_server_ip "$target")" ]; then
                test_server_connectivity "$target"
            else
                error "Unknown server: $target"
                exit 1
            fi
            ;;
            
        "health")
            if [ "$target" == "all" ] || [ -z "$target" ]; then
                run_health_checks
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
            error "Unknown action: $action"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
