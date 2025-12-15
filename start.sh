#!/bin/bash

# Mater Development Environment Start Script
# Starts: ScyllaDB, Backend, Frontend, and optionally Cloudflare Tunnel

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# PID file locations
PID_DIR="$SCRIPT_DIR/.pids"
mkdir -p "$PID_DIR"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if a port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Start ScyllaDB Docker container
start_scylla() {
    log_info "Starting ScyllaDB..."

    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker first."
        return 1
    fi

    # Check if container exists
    if docker ps -a --format '{{.Names}}' | grep -q '^scylla$'; then
        # Container exists, check if running
        if docker ps --format '{{.Names}}' | grep -q '^scylla$'; then
            log_warning "ScyllaDB container is already running"
        else
            # Container exists but not running, start it
            docker start scylla
            log_success "ScyllaDB container started"
        fi
    else
        # Container doesn't exist, create and start it
        log_info "Creating new ScyllaDB container..."
        docker run -d \
            --name scylla \
            -p 9042:9042 \
            -v scylla-data:/var/lib/scylla \
            scylladb/scylla
        log_success "ScyllaDB container created and started"
    fi

    # Wait for ScyllaDB to be ready
    log_info "Waiting for ScyllaDB to be ready..."
    local retries=30
    while [ $retries -gt 0 ]; do
        if docker exec scylla cqlsh -e "DESCRIBE KEYSPACES" >/dev/null 2>&1; then
            log_success "ScyllaDB is ready"
            return 0
        fi
        retries=$((retries - 1))
        sleep 2
    done
    log_warning "ScyllaDB may still be starting up. Continuing anyway..."
}

# Start Backend (Rust)
start_backend() {
    log_info "Starting Backend..."

    if check_port 8000; then
        log_warning "Port 8000 is already in use. Backend may already be running."
        return 0
    fi

    cd "$SCRIPT_DIR/backend"

    # Build if needed
    if [ ! -f "target/debug/mater" ] && [ ! -f "target/release/mater" ]; then
        log_info "Building backend..."
        cargo build
    fi

    # Start backend in background
    cargo run > "$SCRIPT_DIR/.logs/backend.log" 2>&1 &
    local pid=$!
    echo $pid > "$PID_DIR/backend.pid"

    # Wait for backend to start
    local retries=30
    while [ $retries -gt 0 ]; do
        if check_port 8000; then
            log_success "Backend started (PID: $pid) on http://localhost:8000"
            return 0
        fi
        retries=$((retries - 1))
        sleep 1
    done

    log_error "Backend failed to start. Check logs at .logs/backend.log"
    return 1
}

# Start Frontend (React/Vite)
start_frontend() {
    log_info "Starting Frontend..."

    if check_port 5173; then
        log_warning "Port 5173 is already in use. Frontend may already be running."
        return 0
    fi

    cd "$SCRIPT_DIR/frontend"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing frontend dependencies..."
        npm install
    fi

    # Start frontend in background
    npm run dev > "$SCRIPT_DIR/.logs/frontend.log" 2>&1 &
    local pid=$!
    echo $pid > "$PID_DIR/frontend.pid"

    # Wait for frontend to start
    local retries=30
    while [ $retries -gt 0 ]; do
        if check_port 5173; then
            log_success "Frontend started (PID: $pid) on http://localhost:5173"
            return 0
        fi
        retries=$((retries - 1))
        sleep 1
    done

    log_error "Frontend failed to start. Check logs at .logs/frontend.log"
    return 1
}

# Start Cloudflare Tunnel
start_cloudflare() {
    log_info "Starting Cloudflare Tunnel..."

    # Check if cloudflared is installed
    if ! command -v cloudflared &> /dev/null; then
        log_warning "cloudflared is not installed. Skipping tunnel."
        log_info "Install with: brew install cloudflared"
        return 0
    fi

    # Check if named tunnel is already running
    if pgrep -f "cloudflared tunnel run" > /dev/null; then
        log_warning "Cloudflare Tunnel is already running"
        log_success "Tunnel URL: https://campom.org"
        return 0
    fi

    # Start named tunnel in background
    cloudflared tunnel run > "$SCRIPT_DIR/.logs/cloudflare.log" 2>&1 &
    local pid=$!
    echo $pid > "$PID_DIR/cloudflare.pid"

    log_info "Cloudflare Tunnel starting (PID: $pid)..."

    # Wait for tunnel to connect
    local retries=15
    while [ $retries -gt 0 ]; do
        if grep -q "Registered tunnel connection" "$SCRIPT_DIR/.logs/cloudflare.log" 2>/dev/null; then
            break
        fi
        retries=$((retries - 1))
        sleep 1
    done

    if grep -q "Registered tunnel connection" "$SCRIPT_DIR/.logs/cloudflare.log" 2>/dev/null; then
        log_success "Cloudflare Tunnel: https://campom.org"
    else
        log_warning "Cloudflare Tunnel may still be connecting..."
        log_info "Check .logs/cloudflare.log for status"
    fi
}

# Main function
main() {
    echo ""
    echo "=========================================="
    echo "  Mater Development Environment Startup"
    echo "=========================================="
    echo ""

    # Create logs directory
    mkdir -p "$SCRIPT_DIR/.logs"

    # Parse arguments
    local skip_scylla=false
    local skip_cloudflare=false
    local start_tunnel=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-scylla)
                skip_scylla=true
                shift
                ;;
            --no-cloudflare)
                skip_cloudflare=true
                shift
                ;;
            --with-tunnel)
                start_tunnel=true
                shift
                ;;
            -h|--help)
                echo "Usage: ./start.sh [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --no-scylla      Skip starting ScyllaDB"
                echo "  --with-tunnel    Start Cloudflare Tunnel for remote access"
                echo "  -h, --help       Show this help message"
                echo ""
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Start services
    if [ "$skip_scylla" = false ]; then
        start_scylla
    fi

    start_backend
    start_frontend

    if [ "$start_tunnel" = true ]; then
        start_cloudflare
    fi

    echo ""
    echo "=========================================="
    echo "  All Services Started!"
    echo "=========================================="
    echo ""
    log_info "Frontend:  http://localhost:5173"
    log_info "Backend:   http://localhost:8000"
    log_info "ScyllaDB:  localhost:9042"

    # Show Cloudflare tunnel URL if running
    if [ "$start_tunnel" = true ]; then
        echo ""
        log_success "Cloudflare Tunnel: https://campom.org"
    fi

    echo ""
    log_info "Logs are stored in: .logs/"
    log_info "To stop all services, run: ./stop.sh"
    echo ""
}

main "$@"
