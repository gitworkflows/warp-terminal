#!/bin/bash

# Warp Preview Monitor Shell Wrapper
# ==================================
# 
# This script provides easy access to Warp Preview monitoring functionality
# and manages the ~/.warp directory structure.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR_SCRIPT="${SCRIPT_DIR}/warp_monitor.py"
WARP_HOME="${HOME}/.warp"
MONITOR_DIR="${WARP_HOME}/monitor"
CONFIG_DIR="${WARP_HOME}/config"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
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

check_dependencies() {
    local missing_deps=()
    
    # Check for Python 3
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi
    
    # Check for required system tools
    for cmd in ps tail grep; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again."
        exit 1
    fi
}

check_warp_preview() {
    local preview_path="/Volumes/Warp/WarpPreview.app/Contents/MacOS/preview"
    
    if [ ! -f "$preview_path" ]; then
        log_warning "Warp Preview executable not found at: $preview_path"
        log_info "Please ensure Warp Preview is mounted and accessible."
        return 1
    fi
    
    # Check if Warp Preview is running
    if pgrep -f "preview" > /dev/null; then
        log_success "Warp Preview is currently running"
        return 0
    else
        log_warning "Warp Preview does not appear to be running"
        return 1
    fi
}

setup_directories() {
    log_info "Setting up ~/.warp directory structure..."
    
    python3 "$MONITOR_SCRIPT" --setup
    
    if [ $? -eq 0 ]; then
        log_success "Directory structure initialized"
    else
        log_error "Failed to initialize directory structure"
        exit 1
    fi
}

start_monitoring() {
    local verbose_flag=""
    if [ "${1:-}" = "--verbose" ] || [ "${1:-}" = "-v" ]; then
        verbose_flag="--verbose"
        shift
    fi
    
    log_info "Starting Warp Preview monitoring..."
    
    # Check if already running
    if [ -f "${MONITOR_DIR}/monitor.pid" ]; then
        local pid=$(cat "${MONITOR_DIR}/monitor.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_warning "Monitor is already running (PID: $pid)"
            log_info "Use 'stop' command to stop monitoring first"
            return 1
        else
            # Remove stale PID file
            rm -f "${MONITOR_DIR}/monitor.pid"
        fi
    fi
    
    # Start monitoring in background
    nohup python3 "$MONITOR_SCRIPT" $verbose_flag > "${MONITOR_DIR}/monitor_output.log" 2>&1 &
    local monitor_pid=$!
    
    # Save PID
    echo "$monitor_pid" > "${MONITOR_DIR}/monitor.pid"
    
    log_success "Monitor started (PID: $monitor_pid)"
    log_info "Logs: ${MONITOR_DIR}/monitor_output.log"
    log_info "Use 'status' command to check monitoring status"
}

stop_monitoring() {
    if [ -f "${MONITOR_DIR}/monitor.pid" ]; then
        local pid=$(cat "${MONITOR_DIR}/monitor.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping monitor (PID: $pid)..."
            kill -TERM "$pid"
            
            # Wait for graceful shutdown
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 1
                ((count++))
            done
            
            if kill -0 "$pid" 2>/dev/null; then
                log_warning "Monitor didn't stop gracefully, forcing..."
                kill -KILL "$pid"
            fi
            
            rm -f "${MONITOR_DIR}/monitor.pid"
            log_success "Monitor stopped"
        else
            log_warning "Monitor PID file exists but process is not running"
            rm -f "${MONITOR_DIR}/monitor.pid"
        fi
    else
        log_warning "Monitor is not running"
    fi
}

show_status() {
    echo "=== Warp Preview Monitor Status ==="
    echo
    
    # Check if monitoring is running
    if [ -f "${MONITOR_DIR}/monitor.pid" ]; then
        local pid=$(cat "${MONITOR_DIR}/monitor.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_success "Monitor is running (PID: $pid)"
        else
            log_error "Monitor PID file exists but process is not running"
        fi
    else
        log_warning "Monitor is not running"
    fi
    
    echo
    
    # Show Warp Preview processes
    echo "=== Warp Preview Processes ==="
    ps aux | grep -i "preview" | grep -v grep || log_info "No Warp Preview processes found"
    
    echo
    
    # Show recent stats if available
    if [ -f "${MONITOR_DIR}/stats.json" ]; then
        echo "=== Recent Statistics ==="
        python3 "$MONITOR_SCRIPT" --stats
    else
        log_info "No statistics available yet"
    fi
}

show_logs() {
    local lines="${1:-50}"
    
    if [ -f "${MONITOR_DIR}/monitor.log" ]; then
        log_info "Showing last $lines lines from monitor log:"
        echo
        tail -n "$lines" "${MONITOR_DIR}/monitor.log" | while IFS= read -r line; do
            # Pretty print JSON log entries
            if command -v jq &> /dev/null; then
                echo "$line" | jq -r '[.timestamp, .level, .component, .message] | @tsv' 2>/dev/null || echo "$line"
            else
                echo "$line"
            fi
        done
    else
        log_warning "No monitor log found at: ${MONITOR_DIR}/monitor.log"
    fi
}

show_errors() {
    local lines="${1:-20}"
    
    if [ -f "${MONITOR_DIR}/errors.json" ]; then
        log_info "Showing last $lines error entries:"
        echo
        tail -n "$lines" "${MONITOR_DIR}/errors.json" | while IFS= read -r line; do
            if command -v jq &> /dev/null; then
                echo "$line" | jq -r '[.timestamp, .level, .component, .message] | @tsv' 2>/dev/null || echo "$line"
            else
                echo "$line"
            fi
        done
    else
        log_info "No errors logged yet"
    fi
}

tail_preview_log() {
    local lines="${1:-100}"
    log_info "Showing last $lines lines from Warp Preview log:"
    echo
    python3 "$MONITOR_SCRIPT" --tail "$lines"
}

show_help() {
    cat << EOF
Warp Preview Monitor - Shell Wrapper

USAGE:
    $0 <command> [options]

COMMANDS:
    setup           Initialize ~/.warp directory structure
    start [-v]      Start monitoring (use -v for verbose output)
    stop            Stop monitoring
    status          Show monitoring status and statistics
    logs [N]        Show last N lines from monitor log (default: 50)
    errors [N]      Show last N error entries (default: 20)
    tail [N]        Show last N lines from Warp Preview log (default: 100)
    check           Check Warp Preview status and dependencies
    help            Show this help message

EXAMPLES:
    $0 setup                # Initialize directory structure
    $0 start --verbose      # Start monitoring with verbose output
    $0 status               # Check current status
    $0 logs 100             # Show last 100 monitor log lines
    $0 tail 50              # Show last 50 lines from Warp Preview log

FILES:
    ~/.warp/monitor/        Monitor output and statistics
    ~/.warp/config/         Configuration files
    ~/.warp/logs/           Archived logs
    ~/Library/Logs/warp_preview.log    Source log file

EOF
}

# Main script logic
main() {
    if [ $# -eq 0 ]; then
        show_help
        exit 1
    fi
    
    local command="$1"
    shift
    
    case "$command" in
        "setup")
            check_dependencies
            setup_directories
            ;;
        "start")
            check_dependencies
            setup_directories
            start_monitoring "$@"
            ;;
        "stop")
            stop_monitoring
            ;;
        "status")
            show_status
            ;;
        "logs")
            show_logs "$@"
            ;;
        "errors")
            show_errors "$@"
            ;;
        "tail")
            check_dependencies
            tail_preview_log "$@"
            ;;
        "check")
            check_dependencies
            check_warp_preview
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            log_error "Unknown command: $command"
            echo
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
