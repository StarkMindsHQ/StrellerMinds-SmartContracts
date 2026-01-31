#!/bin/bash

# Soroban Localnet Startup Script
# This script starts a local Soroban network in a Docker container for E2E testing

set -e

# Configuration
CONTAINER_NAME="soroban-localnet"
NETWORK_NAME="local"
HOST_PORT="8000"
CONTAINER_PORT="8000"
RPC_URL="http://localhost:${HOST_PORT}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    log_info "Docker is running"
}

# Function to check if Soroban CLI is installed
check_soroban_cli() {
    if ! command -v soroban &> /dev/null; then
        log_error "Soroban CLI is not installed. Please install it and try again."
        echo "Install with: cargo install --locked soroban-cli"
        exit 1
    fi
    
    local version=$(soroban version 2>/dev/null || echo "unknown")
    log_info "Soroban CLI version: $version"
}

# Function to stop existing container if running
stop_existing_container() {
    if docker ps -q -f name=${CONTAINER_NAME} | grep -q .; then
        log_warning "Stopping existing ${CONTAINER_NAME} container..."
        soroban container stop ${NETWORK_NAME} || docker stop ${CONTAINER_NAME} || true
        sleep 2
    fi
    
    # Remove any stopped container with the same name
    if docker ps -aq -f name=${CONTAINER_NAME} | grep -q .; then
        log_info "Removing existing ${CONTAINER_NAME} container..."
        docker rm ${CONTAINER_NAME} || true
    fi
}

# Function to start the localnet container
start_localnet() {
    log_info "Starting Soroban localnet container..."
    
    # Start the container using Soroban CLI
    soroban container start ${NETWORK_NAME} \
        --name ${CONTAINER_NAME} \
        --ports-mapping ${HOST_PORT}:${CONTAINER_PORT} \
        -d
    
    # Wait for the container to be ready
    log_info "Waiting for localnet to be ready..."
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "${RPC_URL}/health" > /dev/null 2>&1; then
            break
        fi
        
        attempt=$((attempt + 1))
        echo -n "."
        sleep 2
    done
    echo
    
    if [ $attempt -eq $max_attempts ]; then
        log_error "Localnet failed to start within timeout"
        log_info "Container logs:"
        soroban container logs ${NETWORK_NAME} || docker logs ${CONTAINER_NAME}
        exit 1
    fi
    
    log_success "Soroban localnet is running!"
    log_info "RPC URL: ${RPC_URL}"
    log_info "Container name: ${CONTAINER_NAME}"
}

# Function to configure local network in Soroban CLI
configure_network() {
    log_info "Configuring Soroban CLI network settings..."
    
    # Add or update the local network configuration
    soroban network add local \
        --rpc-url "${RPC_URL}" \
        --network-passphrase "Standalone Network ; February 2017" || true
    
    log_success "Network configured successfully"
}

# Function to create test accounts
setup_test_accounts() {
    log_info "Setting up test accounts..."
    
    # Create admin account
    if ! soroban keys ls | grep -q "^admin$"; then
        soroban keys generate admin --network local
        log_success "Created admin account"
    else
        log_info "Admin account already exists"
    fi
    
    # Create test user accounts
    local users=("alice" "bob" "charlie")
    for user in "${users[@]}"; do
        if ! soroban keys ls | grep -q "^${user}$"; then
            soroban keys generate ${user} --network local
            log_success "Created ${user} account"
        else
            log_info "${user} account already exists"
        fi
    done
    
    # Fund the accounts using friendbot
    log_info "Funding accounts..."
    local accounts=($(soroban keys ls))
    for account in "${accounts[@]}"; do
        local address=$(soroban keys address ${account})
        curl -X POST "${RPC_URL}/friendbot?addr=${address}" > /dev/null 2>&1 || true
    done
    
    log_success "Test accounts created and funded"
}

# Function to show network status
show_status() {
    log_info "Localnet Status:"
    echo "  RPC URL: ${RPC_URL}"
    echo "  Container: ${CONTAINER_NAME}"
    echo "  Network: ${NETWORK_NAME}"
    
    if docker ps -q -f name=${CONTAINER_NAME} | grep -q .; then
        echo -e "  Status: ${GREEN}Running${NC}"
    else
        echo -e "  Status: ${RED}Stopped${NC}"
    fi
    
    echo
    log_info "Available accounts:"
    if command -v soroban &> /dev/null; then
        local accounts=($(soroban keys ls 2>/dev/null || echo ""))
        for account in "${accounts[@]}"; do
            if [ -n "$account" ]; then
                local address=$(soroban keys address ${account} 2>/dev/null || echo "unknown")
                echo "  ${account}: ${address}"
            fi
        done
    fi
}

# Main execution
main() {
    log_info "Starting Soroban Localnet for E2E Testing"
    echo "========================================"
    
    # Parse command line arguments
    case "${1:-start}" in
        start)
            check_docker
            check_soroban_cli
            stop_existing_container
            start_localnet
            configure_network
            setup_test_accounts
            show_status
            log_success "Localnet is ready for E2E testing!"
            echo
            log_info "To stop the localnet, run: $0 stop"
            ;;
        stop)
            log_info "Stopping Soroban localnet..."
            soroban container stop ${NETWORK_NAME} || docker stop ${CONTAINER_NAME} || true
            docker rm ${CONTAINER_NAME} || true
            log_success "Localnet stopped"
            ;;
        status)
            show_status
            ;;
        restart)
            $0 stop
            sleep 3
            $0 start
            ;;
        logs)
            log_info "Showing localnet logs:"
            soroban container logs ${NETWORK_NAME} || docker logs ${CONTAINER_NAME}
            ;;
        *)
            echo "Usage: $0 {start|stop|restart|status|logs}"
            echo
            echo "Commands:"
            echo "  start   - Start the Soroban localnet (default)"
            echo "  stop    - Stop the localnet"
            echo "  restart - Restart the localnet"
            echo "  status  - Show localnet status"
            echo "  logs    - Show container logs"
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"