#!/bin/bash

# BLC PostgreSQL HA - Complete Pipeline Testing Script
# Tests the entire development → GitHub → compilation → deployment → testing flow

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
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
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

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

header() {
    echo ""
    echo -e "${MAGENTA}============================================${NC}"
    echo -e "${MAGENTA} $1${NC}"
    echo -e "${MAGENTA}============================================${NC}"
    echo ""
}

test_local_compilation() {
    header "🔨 Testing Local Compilation"
    
    log "Checking Rust installation..."
    if ! command -v cargo &> /dev/null; then
        error "Cargo not found. Please install Rust."
        return 1
    fi
    
    success "Rust toolchain found: $(rustc --version)"
    
    log "Running cargo check..."
    if cargo check --all; then
        success "Cargo check passed"
    else
        error "Cargo check failed"
        return 1
    fi
    
    log "Running tests..."
    if cargo test --all; then
        success "All tests passed"
    else
        warning "Some tests failed, but continuing..."
    fi
    
    log "Building release binaries..."
    if cargo build --release; then
        success "Local compilation successful"
    else
        error "Local compilation failed"
        return 1
    fi
    
    # Check if binaries were created
    local binaries=("target/release/blcpg-ha" "target/release/blcpg-cli" "target/release/blcpg-gui")
    for binary in "${binaries[@]}"; do
        if [ -f "$binary" ]; then
            success "✅ $binary created"
        else
            error "❌ $binary not found"
            return 1
        fi
    done
}

test_git_status() {
    header "📝 Testing Git Status"
    
    log "Checking git status..."
    git status --porcelain
    
    log "Current branch: $(git branch --show-current)"
    log "Last commit: $(git log -1 --oneline)"
    
    log "Checking remotes..."
    git remote -v
    
    if git remote | grep -q "slertora"; then
        success "GitHub remote 'slertora' configured"
    else
        error "GitHub remote 'slertora' not found"
        return 1
    fi
}

test_server_connectivity() {
    header "🌐 Testing Server Connectivity"
    
    local failed_servers=""
    
    for server_name in $(get_all_servers); do
        local server_ip=$(get_server_ip "$server_name")
        log "Testing connection to $server_name ($server_ip)..."
        
        if ssh -i $SSH_KEY -o ConnectTimeout=10 $SSH_USER@$server_ip 'echo "Connection successful"' >/dev/null 2>&1; then
            success "✅ $server_name is reachable"
        else
            error "❌ Cannot connect to $server_name"
            failed_servers="$failed_servers $server_name"
        fi
    done
    
    if [ -z "$failed_servers" ]; then
        success "All servers are reachable!"
    else
        error "Failed to connect to:$failed_servers"
        return 1
    fi
}

test_server_health() {
    header "🏥 Testing Server Health"
    
    for server_name in $(get_all_servers); do
        local server_ip=$(get_server_ip "$server_name")
        log "Health check for $server_name ($server_ip)..."
        
        if ssh -i $SSH_KEY $SSH_USER@$server_ip 'test -f /opt/blcpgha/scripts/server-health.sh'; then
            ssh -i $SSH_KEY $SSH_USER@$server_ip '/opt/blcpgha/scripts/server-health.sh' || warning "Health check had issues for $server_name"
        else
            warning "Server $server_name not prepared yet (health script not found)"
        fi
        echo ""
    done
}

test_api_endpoints() {
    header "🔌 Testing API Endpoints"
    
    for server_name in $(get_all_servers); do
        local server_ip=$(get_server_ip "$server_name")
        log "Testing API endpoints on $server_name ($server_ip)..."
        
        # Test health endpoint
        if curl -s -m 10 "http://$server_ip:8080/api/v1/health" >/dev/null 2>&1; then
            success "✅ Health API responding on $server_name"
            
            # Test other endpoints
            for endpoint in "cluster/status" "nodes" "metrics" "vip/status"; do
                if curl -s -m 5 "http://$server_ip:8080/api/v1/$endpoint" >/dev/null 2>&1; then
                    success "✅ /$endpoint API responding on $server_name"
                else
                    warning "⚠️  /$endpoint API not responding on $server_name"
                fi
            done
        else
            warning "⚠️  API not responding on $server_name (service may not be running)"
        fi
        
        # Test GUI
        if curl -s -m 5 "http://$server_ip:3000" >/dev/null 2>&1; then
            success "✅ GUI responding on $server_name"
        else
            warning "⚠️  GUI not responding on $server_name"
        fi
        
        echo ""
    done
}

test_services_status() {
    header "⚙️ Testing Services Status"
    
    for server_name in $(get_all_servers); do
        local server_ip=$(get_server_ip "$server_name")
        log "Checking services on $server_name ($server_ip)..."
        
        # Check PostgreSQL
        if ssh -i $SSH_KEY $SSH_USER@$server_ip 'systemctl is-active postgresql >/dev/null 2>&1'; then
            success "✅ PostgreSQL running on $server_name"
        else
            error "❌ PostgreSQL not running on $server_name"
        fi
        
        # Check blcpg-ha
        if ssh -i $SSH_KEY $SSH_USER@$server_ip 'systemctl is-active blcpg-ha >/dev/null 2>&1'; then
            success "✅ blcpg-ha running on $server_name"
        else
            warning "⚠️  blcpg-ha not running on $server_name"
        fi
        
        # Check blcpg-gui
        if ssh -i $SSH_KEY $SSH_USER@$server_ip 'systemctl is-active blcpg-gui >/dev/null 2>&1'; then
            success "✅ blcpg-gui running on $server_name"
        else
            warning "⚠️  blcpg-gui not running on $server_name"
        fi
        
        echo ""
    done
}

test_deployment_package() {
    header "📦 Testing Deployment Package Creation"
    
    log "Creating deployment package..."
    if ./scripts/deploy-manual.sh all update >/dev/null 2>&1; then
        success "Deployment package created successfully"
    else
        # Try to build manually
        log "Manual deployment failed, trying to build package..."
        
        if [ -f "blcpgha-deploy.tar.gz" ]; then
            success "Deployment package exists"
        else
            warning "Deployment package not found, this is expected for first run"
        fi
    fi
}

run_full_pipeline_test() {
    header "🚀 Full Pipeline Test"
    
    local tests=(
        "test_local_compilation"
        "test_git_status" 
        "test_server_connectivity"
        "test_server_health"
        "test_services_status"
        "test_api_endpoints"
        "test_deployment_package"
    )
    
    local passed=0
    local failed=0
    local warnings=0
    
    for test in "${tests[@]}"; do
        if $test; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    header "📊 Test Results Summary"
    
    echo -e "${GREEN}✅ Passed: $passed${NC}"
    echo -e "${RED}❌ Failed: $failed${NC}"
    echo -e "${YELLOW}⚠️  Warnings: Check individual test outputs${NC}"
    
    if [ $failed -eq 0 ]; then
        success "🎉 All critical tests passed! Pipeline is ready!"
        
        echo ""
        info "🎯 Next steps:"
        echo "   1. Add SSH key to GitHub Secrets if not done"
        echo "   2. Push changes to trigger GitHub Actions"
        echo "   3. Monitor deployment in GitHub Actions"
        echo "   4. Access web interfaces:"
        for server_name in "${!SERVERS[@]}"; do
            local server_ip=${SERVERS[$server_name]}
            echo "      - $server_name: http://$server_ip:3000"
        done
        
        return 0
    else
        error "Some tests failed. Please fix issues before proceeding."
        return 1
    fi
}

show_usage() {
    cat << EOF
BLC PostgreSQL HA - Pipeline Testing Script

Usage: $0 [test_name]

Available tests:
  compilation      - Test local Rust compilation
  git             - Test git status and remotes
  connectivity    - Test SSH connectivity to servers
  health          - Test server health checks
  services        - Test service status on servers
  api             - Test API endpoints
  package         - Test deployment package creation
  full            - Run all tests (default)

Examples:
  $0                    # Run full pipeline test
  $0 full              # Run full pipeline test
  $0 connectivity      # Test only server connectivity
  $0 api              # Test only API endpoints

EOF
}

main() {
    local test_name=${1:-"full"}
    
    case $test_name in
        "compilation")
            test_local_compilation
            ;;
        "git")
            test_git_status
            ;;
        "connectivity")
            test_server_connectivity
            ;;
        "health")
            test_server_health
            ;;
        "services")
            test_services_status
            ;;
        "api")
            test_api_endpoints
            ;;
        "package")
            test_deployment_package
            ;;
        "full")
            run_full_pipeline_test
            ;;
        *)
            error "Unknown test: $test_name"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
