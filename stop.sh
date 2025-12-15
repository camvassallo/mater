#!/bin/bash

# Mater Development Environment Stop Script
# Stops: ScyllaDB, Backend, Frontend, Cloudflare Tunnel, and any hung processes

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PID_DIR="$SCRIPT_DIR/.pids"

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

# Kill process by PID file
kill_by_pid_file() {
    local pid_file=$1
    local name=$2

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping $name (PID: $pid)..."
            kill "$pid" 2>/dev/null
            sleep 1
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                log_warning "$name did not stop gracefully, force killing..."
                kill -9 "$pid" 2>/dev/null
            fi
            log_success "$name stopped"
        fi
        rm -f "$pid_file"
    fi
}

# Kill processes by port
kill_by_port() {
    local port=$1
    local name=$2

    local pids=$(lsof -ti :$port 2>/dev/null)
    if [ -n "$pids" ]; then
        log_info "Stopping processes on port $port ($name)..."
        for pid in $pids; do
            log_info "  Killing PID $pid..."
            kill "$pid" 2>/dev/null
        done
        sleep 1
        # Force kill any remaining
        pids=$(lsof -ti :$port 2>/dev/null)
        if [ -n "$pids" ]; then
            log_warning "Force killing remaining processes on port $port..."
            for pid in $pids; do
                kill -9 "$pid" 2>/dev/null
            done
        fi
        log_success "Port $port cleared"
    else
        log_info "No processes found on port $port ($name)"
    fi
}

# Kill processes by name pattern
kill_by_pattern() {
    local pattern=$1
    local name=$2
    local skip_self=${3:-false}

    local pids=$(pgrep -f "$pattern" 2>/dev/null)
    if [ -n "$pids" ]; then
        log_info "Stopping $name processes..."
        for pid in $pids; do
            # Skip the current script
            if [ "$skip_self" = true ] && [ "$pid" = "$$" ]; then
                continue
            fi
            log_info "  Killing PID $pid..."
            kill "$pid" 2>/dev/null
        done
        sleep 1
        # Force kill any remaining
        pids=$(pgrep -f "$pattern" 2>/dev/null)
        if [ -n "$pids" ]; then
            log_warning "Force killing remaining $name processes..."
            for pid in $pids; do
                if [ "$skip_self" = true ] && [ "$pid" = "$$" ]; then
                    continue
                fi
                kill -9 "$pid" 2>/dev/null
            done
        fi
        log_success "$name processes stopped"
    fi
}

# Stop Frontend
stop_frontend() {
    log_info "Stopping Frontend..."

    # Kill by PID file
    kill_by_pid_file "$PID_DIR/frontend.pid" "Frontend (from PID file)"

    # Kill by port
    kill_by_port 5173 "Frontend"

    # Kill any remaining vite/node processes for this project
    kill_by_pattern "vite.*mater" "Vite dev server"
    kill_by_pattern "node.*mater/frontend" "Node (frontend)"
}

# Stop Backend
stop_backend() {
    log_info "Stopping Backend..."

    # Kill by PID file
    kill_by_pid_file "$PID_DIR/backend.pid" "Backend (from PID file)"

    # Kill by port
    kill_by_port 8000 "Backend"

    # Kill any remaining cargo/mater processes
    kill_by_pattern "cargo.*run.*mater" "Cargo run"
    kill_by_pattern "target/debug/mater" "Mater binary (debug)"
    kill_by_pattern "target/release/mater" "Mater binary (release)"
}

# Stop ScyllaDB
stop_scylla() {
    log_info "Stopping ScyllaDB..."

    if ! docker info >/dev/null 2>&1; then
        log_warning "Docker is not running"
        return 0
    fi

    if docker ps --format '{{.Names}}' | grep -q '^scylla$'; then
        docker stop scylla
        log_success "ScyllaDB container stopped"
    else
        log_info "ScyllaDB container is not running"
    fi
}

# Stop Cloudflare Tunnel
stop_cloudflare() {
    log_info "Stopping Cloudflare Tunnel..."

    # Kill by PID file
    kill_by_pid_file "$PID_DIR/cloudflare.pid" "Cloudflare Tunnel (from PID file)"

    # Kill by pattern - kill tunnels started by this script
    kill_by_pattern "cloudflared tunnel run" "Cloudflare Tunnel"
}

# Kill all hung processes related to Mater
# Argument: $1 = "keep_scylla" to skip port 9042
kill_hung_processes() {
    local keep_scylla=${1:-false}
    log_info "Checking for hung processes..."

    # Common hung process patterns
    local patterns=(
        "cargo.*mater"
        "rustc.*mater"
        "node.*mater"
        "npm.*mater"
        "vite.*5173"
        "cloudflared.*5173"
    )

    local found_any=false
    for pattern in "${patterns[@]}"; do
        local pids=$(pgrep -f "$pattern" 2>/dev/null)
        if [ -n "$pids" ]; then
            found_any=true
            log_warning "Found hung processes matching '$pattern'"
            for pid in $pids; do
                # Get process info for display
                local proc_info=$(ps -p "$pid" -o pid,comm,args 2>/dev/null | tail -1)
                log_info "  Killing: $proc_info"
                kill -9 "$pid" 2>/dev/null
            done
        fi
    done

    # Also check ports (skip 9042 if keeping scylla)
    local ports=(5173 8000)
    if [ "$keep_scylla" = false ]; then
        ports+=(9042)
    fi
    for port in "${ports[@]}"; do
        local pids=$(lsof -ti :$port 2>/dev/null)
        if [ -n "$pids" ]; then
            found_any=true
            log_warning "Found processes still using port $port"
            for pid in $pids; do
                local proc_info=$(ps -p "$pid" -o pid,comm,args 2>/dev/null | tail -1)
                log_info "  Killing: $proc_info"
                kill -9 "$pid" 2>/dev/null
            done
        fi
    done

    if [ "$found_any" = false ]; then
        log_success "No hung processes found"
    else
        log_success "Hung processes cleaned up"
    fi
}

# Main function
main() {
    echo ""
    echo "=========================================="
    echo "  Mater Development Environment Shutdown"
    echo "=========================================="
    echo ""

    # Parse arguments
    local stop_scylla_flag=true
    local force_kill=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --keep-scylla)
                stop_scylla_flag=false
                shift
                ;;
            --force)
                force_kill=true
                shift
                ;;
            -h|--help)
                echo "Usage: ./stop.sh [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --keep-scylla    Keep ScyllaDB running"
                echo "  --force          Force kill all processes immediately"
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

    # Determine if we should keep scylla for hung process cleanup
    local keep_scylla_for_cleanup=false
    if [ "$stop_scylla_flag" = false ]; then
        keep_scylla_for_cleanup=true
    fi

    # If force flag, skip graceful shutdown
    if [ "$force_kill" = true ]; then
        log_warning "Force mode enabled - killing all processes immediately"
        kill_hung_processes "$keep_scylla_for_cleanup"
        if [ "$stop_scylla_flag" = true ]; then
            stop_scylla
        fi
    else
        # Graceful shutdown
        stop_cloudflare
        stop_frontend
        stop_backend

        if [ "$stop_scylla_flag" = true ]; then
            stop_scylla
        fi

        # Clean up any remaining hung processes
        kill_hung_processes "$keep_scylla_for_cleanup"
    fi

    # Clean up PID files
    rm -f "$PID_DIR"/*.pid 2>/dev/null

    echo ""
    echo "=========================================="
    echo "  All Services Stopped!"
    echo "=========================================="
    echo ""

    # Show status
    log_info "Port status:"
    for port in 5173 8000; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            log_warning "  Port $port: STILL IN USE"
        else
            log_success "  Port $port: Free"
        fi
    done
    # Show ScyllaDB port status with context
    if lsof -Pi :9042 -sTCP:LISTEN -t >/dev/null 2>&1; then
        if [ "$stop_scylla_flag" = false ]; then
            log_success "  Port 9042: In use (ScyllaDB kept running)"
        else
            log_warning "  Port 9042: STILL IN USE"
        fi
    else
        log_success "  Port 9042: Free"
    fi
    echo ""
}

main "$@"
