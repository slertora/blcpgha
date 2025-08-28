#!/bin/bash

# BLC PostgreSQL HA - Complete Development Workflow Script
# Orchestrates the entire development → GitHub → compilation → deployment → testing flow

set -e

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

confirm() {
    local message=$1
    echo -e "${YELLOW}$message${NC}"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Operation cancelled by user"
        exit 0
    fi
}

# Workflow phases
phase_setup() {
    header "🔧 Phase 1: Setup and Verification"
    
    log "Checking prerequisites..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    success "✅ Rust toolchain found: $(rustc --version)"
    
    # Check git
    if ! command -v git &> /dev/null; then
        error "Git not found. Please install Git."
        exit 1
    fi
    success "✅ Git found: $(git --version)"
    
    # Check SSH key
    if [ ! -f ~/.ssh/binlogic ]; then
        error "SSH key ~/.ssh/binlogic not found"
        exit 1
    fi
    success "✅ SSH key found"
    
    # Check GitHub remote
    if ! git remote | grep -q "slertora"; then
        error "GitHub remote 'slertora' not configured"
        info "Run: git remote add slertora git@slertora-github:slertora/blcpgha.git"
        exit 1
    fi
    success "✅ GitHub remote configured"
    
    log "Setup verification completed!"
}

phase_test_local() {
    header "🔍 Phase 2: Local Testing"
    
    log "Running local tests..."
    ./scripts/test-pipeline.sh compilation
    
    success "Local testing completed!"
}

phase_prepare_servers() {
    header "🖥️ Phase 3: Server Preparation"
    
    confirm "This will prepare all 3 production servers with dependencies and configuration."
    
    log "Testing server connectivity..."
    ./scripts/prepare-servers.sh test all
    
    log "Preparing servers..."
    ./scripts/prepare-servers.sh prepare all
    
    log "Running health checks..."
    ./scripts/prepare-servers.sh health all
    
    success "Server preparation completed!"
}

phase_manual_deploy() {
    header "🚀 Phase 4: Manual Deployment Test"
    
    confirm "This will deploy the application to all servers for testing."
    
    log "Creating deployment package..."
    if [ ! -f "blcpgha-deploy.tar.gz" ]; then
        log "Building deployment package..."
        # Create a simple deployment package for testing
        cargo build --release
        mkdir -p deploy/{bin,config,scripts}
        cp target/release/blcpg-* deploy/bin/ 2>/dev/null || warning "Some binaries not found"
        cp */config.toml deploy/config/ 2>/dev/null || true
        tar -czf blcpgha-deploy.tar.gz -C deploy . 2>/dev/null || warning "Package creation had issues"
    fi
    
    log "Deploying to all servers..."
    ./scripts/deploy-manual.sh all update
    
    log "Verifying deployment..."
    ./scripts/test-pipeline.sh services
    ./scripts/test-pipeline.sh api
    
    success "Manual deployment completed!"
}

phase_github_setup() {
    header "📝 Phase 5: GitHub Integration"
    
    log "Checking current git status..."
    git status
    
    if [ -n "$(git status --porcelain)" ]; then
        confirm "You have uncommitted changes. Commit and push them to GitHub?"
        
        log "Adding changes..."
        git add .
        
        echo -e "${CYAN}Enter commit message (or press Enter for default):${NC}"
        read -r commit_msg
        if [ -z "$commit_msg" ]; then
            commit_msg="🔄 Update BLC PostgreSQL HA - $(date +'%Y-%m-%d %H:%M:%S')"
        fi
        
        log "Committing changes..."
        git commit -m "$commit_msg"
    fi
    
    log "Pushing to GitHub..."
    git push slertora rustmaster
    
    success "GitHub integration completed!"
    
    info "🎯 GitHub Actions will now:"
    info "   1. Run tests and compilation"
    info "   2. Build Linux binaries"
    info "   3. Deploy to all servers automatically"
    info ""
    info "Monitor the workflow at: https://github.com/slertora/blcpgha/actions"
}

phase_verify_pipeline() {
    header "✅ Phase 6: Pipeline Verification"
    
    log "Waiting a moment for GitHub Actions to start..."
    sleep 10
    
    log "Running full pipeline test..."
    ./scripts/test-pipeline.sh full
    
    success "Pipeline verification completed!"
    
    info "🎉 Complete development workflow is now active!"
    info ""
    info "🔄 Your workflow:"
    info "   1. Make code changes"
    info "   2. Test locally: ./scripts/test-pipeline.sh compilation"
    info "   3. Commit and push: git push slertora rustmaster"
    info "   4. GitHub Actions automatically deploys"
    info "   5. Test deployment: ./scripts/test-pipeline.sh api"
    info ""
    info "🌐 Access your applications:"
    info "   - Master GUI: http://187.33.155.182:3000"
    info "   - Replica 1 GUI: http://187.33.144.18:3000"
    info "   - Replica 2 GUI: http://187.33.147.49:3000"
}

# Quick actions
action_quick_test() {
    header "⚡ Quick Test"
    ./scripts/test-pipeline.sh connectivity
    ./scripts/test-pipeline.sh services
    ./scripts/test-pipeline.sh api
}

action_quick_deploy() {
    header "⚡ Quick Deploy"
    confirm "This will build and deploy to all servers."
    cargo build --release
    ./scripts/deploy-manual.sh all update
    action_quick_test
}

action_quick_push() {
    header "⚡ Quick Push"
    if [ -n "$(git status --porcelain)" ]; then
        git add .
        echo -e "${CYAN}Enter commit message:${NC}"
        read -r commit_msg
        if [ -n "$commit_msg" ]; then
            git commit -m "$commit_msg"
            git push slertora rustmaster
            success "Pushed to GitHub! Check Actions for deployment status."
        else
            warning "Empty commit message, operation cancelled."
        fi
    else
        info "No changes to commit."
    fi
}

action_status() {
    header "📊 System Status"
    
    echo -e "${CYAN}Git Status:${NC}"
    git status --short
    echo ""
    
    echo -e "${CYAN}Last Commit:${NC}"
    git log -1 --oneline
    echo ""
    
    echo -e "${CYAN}Server Connectivity:${NC}"
    ./scripts/test-pipeline.sh connectivity
    echo ""
    
    echo -e "${CYAN}Services Status:${NC}"
    ./scripts/test-pipeline.sh services
    echo ""
    
    echo -e "${CYAN}API Status:${NC}"
    ./scripts/test-pipeline.sh api
}

show_usage() {
    cat << EOF
BLC PostgreSQL HA - Development Workflow Script

Usage: $0 [action]

Full Workflow (run in order):
  setup           - Verify prerequisites and setup
  test            - Run local tests
  prepare         - Prepare production servers
  deploy          - Test manual deployment
  github          - Setup GitHub integration
  verify          - Verify complete pipeline
  full            - Run all phases (setup → verify)

Quick Actions:
  quick-test      - Quick connectivity and API test
  quick-deploy    - Quick build and deploy
  quick-push      - Quick commit and push to GitHub
  status          - Show current system status

Examples:
  $0 full              # Complete workflow setup
  $0 quick-test        # Quick health check
  $0 quick-deploy      # Quick deployment
  $0 quick-push        # Quick commit and push
  $0 status            # Show current status

Typical Development Flow:
  1. $0 full           # One-time setup
  2. Make code changes
  3. $0 quick-test     # Test changes
  4. $0 quick-push     # Push to GitHub (auto-deploys)
  5. $0 status         # Verify deployment

EOF
}

main() {
    local action=${1:-"help"}
    
    case $action in
        "setup")
            phase_setup
            ;;
        "test")
            phase_test_local
            ;;
        "prepare")
            phase_prepare_servers
            ;;
        "deploy")
            phase_manual_deploy
            ;;
        "github")
            phase_github_setup
            ;;
        "verify")
            phase_verify_pipeline
            ;;
        "full")
            phase_setup
            phase_test_local
            phase_prepare_servers
            phase_manual_deploy
            phase_github_setup
            phase_verify_pipeline
            ;;
        "quick-test")
            action_quick_test
            ;;
        "quick-deploy")
            action_quick_deploy
            ;;
        "quick-push")
            action_quick_push
            ;;
        "status")
            action_status
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Show banner
echo -e "${MAGENTA}"
cat << 'EOF'
 ____  _     ____   ____   ____                   _   _    _    
| __ )| |   / ___| |  _ \ / ___|                 | | | |  / \   
|  _ \| |  | |     | |_) | |  _   _____ _____ ___| |_| | / _ \  
| |_) | |__| |___  |  __/| |_| | |_____|_____|___|  _  |/ ___ \ 
|____/|____|\____| |_|    \____|                 |_| |_/_/   \_\

PostgreSQL High Availability - Development Workflow
EOF
echo -e "${NC}"

# Run main function with all arguments
main "$@"
