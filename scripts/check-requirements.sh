#!/bin/bash

# BLC PostgreSQL HA - Requirements Check Script
# This script verifies all prerequisites for the add-replica functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "OK")
            echo -e "${GREEN}✅ OK${NC} - $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  WARNING${NC} - $message"
            ;;
        "ERROR")
            echo -e "${RED}❌ ERROR${NC} - $message"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  INFO${NC} - $message"
            ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check SSH connectivity
check_ssh() {
    local target=$1
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "$target" 'echo "SSH OK"' 2>/dev/null; then
        print_status "OK" "SSH access to $target"
        return 0
    else
        print_status "ERROR" "SSH access to $target failed"
        return 1
    fi
}

# Function to check PostgreSQL installation
check_postgresql() {
    local target=$1
    if ssh "$target" 'systemctl status postgresql --no-pager >/dev/null 2>&1'; then
        print_status "OK" "PostgreSQL is installed and running on $target"
        return 0
    else
        print_status "WARN" "PostgreSQL is not installed or not running on $target"
        return 1
    fi
}

# Function to check disk space
check_disk_space() {
    local target=$1
    local required_space=20 # GB
    local available_space=$(ssh "$target" 'df -BG /var/lib/postgresql | tail -1 | awk "{print \$4}" | sed "s/G//"')
    
    if [ "$available_space" -ge "$required_space" ]; then
        print_status "OK" "Sufficient disk space on $target ($available_space GB available)"
        return 0
    else
        print_status "WARN" "Insufficient disk space on $target ($available_space GB available, $required_space GB required)"
        return 1
    fi
}

# Function to check replication user
check_replication_user() {
    local target=$1
    if ssh "$target" 'sudo -u postgres psql -c "SELECT rolname FROM pg_roles WHERE rolname = '\''replicator'\'';"' 2>/dev/null | grep -q replicator; then
        print_status "OK" "Replication user exists on $target"
        return 0
    else
        print_status "WARN" "Replication user does not exist on $target"
        return 1
    fi
}

# Function to check network connectivity
check_network() {
    local target=$1
    local port=$2
    if ssh "$target" "telnet localhost $port" </dev/null 2>&1 | grep -q "Connected"; then
        print_status "OK" "Network connectivity to $target:$port"
        return 0
    else
        print_status "WARN" "Network connectivity to $target:$port failed"
        return 1
    fi
}

# Function to check firewall rules
check_firewall() {
    local target=$1
    if ssh "$target" 'sudo ufw status | grep -q "5432.*ALLOW"'; then
        print_status "OK" "Firewall allows PostgreSQL port on $target"
        return 0
    else
        print_status "WARN" "Firewall may not allow PostgreSQL port on $target"
        return 1
    fi
}

# Function to check SSL certificates
check_ssl_certificates() {
    local target=$1
    if ssh "$target" '[ -f /var/lib/postgresql/server.crt ] && [ -f /var/lib/postgresql/server.key ]'; then
        print_status "OK" "SSL certificates exist on $target"
        return 0
    else
        print_status "WARN" "SSL certificates not found on $target"
        return 1
    fi
}

# Function to check system requirements
check_system_requirements() {
    local target=$1
    
    # Check OS version
    local os_version=$(ssh "$target" 'cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2')
    print_status "INFO" "OS version on $target: $os_version"
    
    # Check RAM
    local ram_gb=$(ssh "$target" 'free -g | grep Mem | awk "{print \$2}"')
    if [ "$ram_gb" -ge 4 ]; then
        print_status "OK" "Sufficient RAM on $target ($ram_gb GB)"
    else
        print_status "WARN" "Insufficient RAM on $target ($ram_gb GB, 4 GB recommended)"
    fi
    
    # Check architecture
    local arch=$(ssh "$target" 'uname -m')
    if [ "$arch" = "x86_64" ] || [ "$arch" = "aarch64" ]; then
        print_status "OK" "Supported architecture on $target ($arch)"
    else
        print_status "WARN" "Unsupported architecture on $target ($arch)"
    fi
}

# Main function
main() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}BLC PostgreSQL HA Requirements Check${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
    
    # Check if running as root
    if [ "$EUID" -eq 0 ]; then
        print_status "WARN" "Running as root is not recommended"
    fi
    
    # Check required commands
    print_status "INFO" "Checking required commands..."
    local missing_commands=()
    
    for cmd in ssh scp telnet; do
        if command_exists "$cmd"; then
            print_status "OK" "$cmd is available"
        else
            missing_commands+=("$cmd")
            print_status "ERROR" "$cmd is not available"
        fi
    done
    
    if [ ${#missing_commands[@]} -gt 0 ]; then
        echo
        print_status "ERROR" "Missing required commands: ${missing_commands[*]}"
        print_status "INFO" "Please install the missing commands and try again"
        exit 1
    fi
    
    # Get target nodes from command line or environment
    if [ $# -eq 0 ]; then
        echo
        print_status "INFO" "No target nodes specified. Using environment variable TARGET_NODES"
        if [ -z "$TARGET_NODES" ]; then
            print_status "ERROR" "TARGET_NODES environment variable not set"
            print_status "INFO" "Usage: $0 <node1> [node2] [node3] ..."
            print_status "INFO" "Or set TARGET_NODES environment variable"
            exit 1
        fi
        TARGETS=($TARGET_NODES)
    else
        TARGETS=("$@")
    fi
    
    echo
    print_status "INFO" "Checking requirements for ${#TARGETS[@]} target node(s): ${TARGETS[*]}"
    echo
    
    local overall_status=0
    
    # Check each target node
    for target in "${TARGETS[@]}"; do
        echo -e "${BLUE}Checking node: $target${NC}"
        echo "----------------------------------------"
        
        # Check SSH connectivity
        if ! check_ssh "$target"; then
            overall_status=1
            continue
        fi
        
        # Check system requirements
        check_system_requirements "$target"
        
        # Check disk space
        if ! check_disk_space "$target"; then
            overall_status=1
        fi
        
        # Check PostgreSQL installation
        if ! check_postgresql "$target"; then
            overall_status=1
        fi
        
        # Check replication user (only if PostgreSQL is running)
        if ssh "$target" 'systemctl status postgresql --no-pager >/dev/null 2>&1'; then
            if ! check_replication_user "$target"; then
                overall_status=1
            fi
        fi
        
        # Check network connectivity
        if ! check_network "$target" 5432; then
            overall_status=1
        fi
        
        # Check firewall rules
        if ! check_firewall "$target"; then
            overall_status=1
        fi
        
        # Check SSL certificates
        if ! check_ssl_certificates "$target"; then
            overall_status=1
        fi
        
        echo
    done
    
    # Summary
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}Requirements Check Summary${NC}"
    echo -e "${BLUE}================================${NC}"
    
    if [ $overall_status -eq 0 ]; then
        print_status "OK" "All requirements met! Ready to use add-replica functionality"
        echo
        print_status "INFO" "Next steps:"
        print_status "INFO" "1. Run: blcpg-cli add-replica --target=<node> --source=auto"
        print_status "INFO" "2. Monitor the process with: blcpg-cli cluster-info"
        print_status "INFO" "3. Check replication status with: blcpg-cli replication-status"
    else
        print_status "ERROR" "Some requirements are not met. Please fix the issues above and run this script again"
        echo
        print_status "INFO" "For detailed information, see: docs/requirements.md"
    fi
    
    exit $overall_status
}

# Help function
show_help() {
    echo "BLC PostgreSQL HA - Requirements Check Script"
    echo
    echo "Usage: $0 [OPTIONS] <node1> [node2] [node3] ..."
    echo
    echo "Options:"
    echo "  -h, --help    Show this help message"
    echo
    echo "Examples:"
    echo "  $0 node-1 node-2 node-3"
    echo "  TARGET_NODES=\"node-1 node-2 node-3\" $0"
    echo
    echo "This script checks all prerequisites for the add-replica functionality:"
    echo "  - SSH access and authentication"
    echo "  - PostgreSQL installation and configuration"
    echo "  - Network connectivity and firewall rules"
    echo "  - Disk space and system requirements"
    echo "  - SSL certificates and security settings"
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac 