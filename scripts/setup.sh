#!/bin/bash
# Developer Bootstrap Script for StrellerMinds Smart Contracts
# This script sets up all prerequisites for Soroban development

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Pinned versions
SOROBAN_CLI_VERSION="21.5.0"
STELLAR_CLI_VERSION="21.5.0"

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# Check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Get version of a command
get_version() {
    local cmd="$1"
    if command_exists "$cmd"; then
        $cmd --version 2>&1 | head -n 1
    else
        echo "Not installed"
    fi
}

# Main setup function
main() {
    print_header "StrellerMinds Smart Contracts - Developer Bootstrap"
    
    echo "This script will set up your development environment for Soroban smart contracts."
    echo "It will install and verify the following:"
    echo "  • Rust toolchain with wasm32-unknown-unknown target"
    echo "  • Soroban CLI (v${SOROBAN_CLI_VERSION})"
    echo "  • Stellar CLI (v${STELLAR_CLI_VERSION})"
    echo "  • Optional: Binaryen (wasm-opt)"
    echo ""
    
    # Step 1: Check Rust installation
    print_header "Step 1: Checking Rust Installation"
    
    if ! command_exists rustc; then
        print_error "Rust is not installed."
        print_info "Please install Rust from: https://www.rust-lang.org/tools/install"
        print_info "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    print_success "Rust is installed: $(rustc --version)"
    print_success "Cargo is installed: $(cargo --version)"
    
    # Step 2: Install wasm32-unknown-unknown target
    print_header "Step 2: Installing Rust WASM Target"
    
    if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
        print_success "wasm32-unknown-unknown target is already installed"
    else
        print_info "Installing wasm32-unknown-unknown target..."
        if rustup target add wasm32-unknown-unknown; then
            print_success "Successfully installed wasm32-unknown-unknown target"
        else
            print_error "Failed to install wasm32-unknown-unknown target"
            exit 1
        fi
    fi
    
    # Step 3: Install Soroban CLI
    print_header "Step 3: Installing Soroban CLI"
    
    if command_exists soroban; then
        current_version=$(soroban --version 2>&1 | grep -oP 'soroban \K[0-9.]+' || echo "unknown")
        print_info "Soroban CLI is already installed: v${current_version}"
        
        if [ "$current_version" != "$SOROBAN_CLI_VERSION" ]; then
            print_warning "Installed version (v${current_version}) differs from pinned version (v${SOROBAN_CLI_VERSION})"
            read -p "Do you want to install the pinned version v${SOROBAN_CLI_VERSION}? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                print_info "Installing Soroban CLI v${SOROBAN_CLI_VERSION}..."
                if cargo install --locked soroban-cli --version ${SOROBAN_CLI_VERSION}; then
                    print_success "Successfully installed Soroban CLI v${SOROBAN_CLI_VERSION}"
                else
                    print_error "Failed to install Soroban CLI"
                    exit 1
                fi
            else
                print_warning "Continuing with current version v${current_version}"
            fi
        else
            print_success "Soroban CLI v${SOROBAN_CLI_VERSION} is already installed"
        fi
    else
        print_info "Installing Soroban CLI v${SOROBAN_CLI_VERSION}..."
        if cargo install --locked soroban-cli --version ${SOROBAN_CLI_VERSION}; then
            print_success "Successfully installed Soroban CLI v${SOROBAN_CLI_VERSION}"
        else
            print_error "Failed to install Soroban CLI"
            exit 1
        fi
    fi
    
    # Step 4: Install Stellar CLI
    print_header "Step 4: Installing Stellar CLI"
    
    if command_exists stellar; then
        current_version=$(stellar --version 2>&1 | grep -oP 'stellar \K[0-9.]+' || echo "unknown")
        print_info "Stellar CLI is already installed: v${current_version}"
        
        if [ "$current_version" != "$STELLAR_CLI_VERSION" ]; then
            print_warning "Installed version (v${current_version}) differs from pinned version (v${STELLAR_CLI_VERSION})"
            read -p "Do you want to install the pinned version v${STELLAR_CLI_VERSION}? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                print_info "Installing Stellar CLI v${STELLAR_CLI_VERSION}..."
                if cargo install --locked stellar-cli --version ${STELLAR_CLI_VERSION}; then
                    print_success "Successfully installed Stellar CLI v${STELLAR_CLI_VERSION}"
                else
                    print_error "Failed to install Stellar CLI"
                    exit 1
                fi
            else
                print_warning "Continuing with current version v${current_version}"
            fi
        else
            print_success "Stellar CLI v${STELLAR_CLI_VERSION} is already installed"
        fi
    else
        print_info "Installing Stellar CLI v${STELLAR_CLI_VERSION}..."
        if cargo install --locked stellar-cli --version ${STELLAR_CLI_VERSION}; then
            print_success "Successfully installed Stellar CLI v${STELLAR_CLI_VERSION}"
        else
            print_error "Failed to install Stellar CLI"
            exit 1
        fi
    fi
    
    # Step 5: Verify installations
    print_header "Step 5: Verifying Installations"
    
    print_info "Verifying soroban command..."
    if command_exists soroban; then
        print_success "soroban: $(soroban --version)"
    else
        print_error "soroban command not found in PATH"
        print_warning "You may need to add ~/.cargo/bin to your PATH"
        exit 1
    fi
    
    print_info "Verifying stellar command..."
    if command_exists stellar; then
        print_success "stellar: $(stellar --version)"
    else
        print_error "stellar command not found in PATH"
        print_warning "You may need to add ~/.cargo/bin to your PATH"
        exit 1
    fi
    
    # Step 6: Optional - Install Binaryen (wasm-opt)
    print_header "Step 6: Optional - Binaryen (wasm-opt)"
    
    if command_exists wasm-opt; then
        print_success "wasm-opt is already installed: $(wasm-opt --version 2>&1 | head -n 1)"
    else
        print_warning "wasm-opt is not installed (optional but recommended for WASM optimization)"
        echo ""
        echo "To install Binaryen (wasm-opt), choose your platform:"
        echo "  • macOS:   brew install binaryen"
        echo "  • Ubuntu:  sudo apt-get install binaryen"
        echo "  • Arch:    sudo pacman -S binaryen"
        echo "  • Manual:  https://github.com/WebAssembly/binaryen/releases"
        echo ""
        read -p "Do you want to attempt automatic installation? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                if command_exists brew; then
                    print_info "Installing binaryen via Homebrew..."
                    brew install binaryen && print_success "Successfully installed binaryen"
                else
                    print_warning "Homebrew not found. Please install manually."
                fi
            elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
                if command_exists apt-get; then
                    print_info "Installing binaryen via apt-get..."
                    sudo apt-get update && sudo apt-get install -y binaryen && print_success "Successfully installed binaryen"
                elif command_exists pacman; then
                    print_info "Installing binaryen via pacman..."
                    sudo pacman -S --noconfirm binaryen && print_success "Successfully installed binaryen"
                else
                    print_warning "Package manager not recognized. Please install manually."
                fi
            else
                print_warning "OS not recognized. Please install manually."
            fi
        fi
    fi
    
    # Step 7: Additional checks
    print_header "Step 7: Additional Checks"
    
    # Check for Node.js (needed for E2E tests)
    if command_exists node; then
        node_version=$(node --version)
        print_success "Node.js is installed: ${node_version}"
        
        # Check if version is >= 18
        node_major=$(echo "$node_version" | grep -oP 'v\K[0-9]+')
        if [ "$node_major" -lt 18 ]; then
            print_warning "Node.js version should be >= 18 for E2E tests (current: ${node_version})"
        fi
    else
        print_warning "Node.js is not installed (required for E2E tests)"
        print_info "Install from: https://nodejs.org/"
    fi
    
    # Check for Docker (optional for local testnet)
    if command_exists docker; then
        print_success "Docker is installed: $(docker --version)"
    else
        print_warning "Docker is not installed (optional, needed for local Stellar testnet)"
        print_info "Install from: https://docs.docker.com/get-docker/"
    fi
    
    # Final summary
    print_header "Setup Complete!"
    
    echo "Your development environment is ready for Soroban smart contract development!"
    echo ""
    echo "Installed components:"
    echo "  ✓ Rust: $(rustc --version)"
    echo "  ✓ Cargo: $(cargo --version)"
    echo "  ✓ wasm32-unknown-unknown target"
    echo "  ✓ Soroban CLI: $(soroban --version)"
    echo "  ✓ Stellar CLI: $(stellar --version)"
    if command_exists wasm-opt; then
        echo "  ✓ wasm-opt: $(wasm-opt --version 2>&1 | head -n 1)"
    fi
    echo ""
    echo "Next steps:"
    echo "  1. Build contracts:     ./scripts/build.sh"
    echo "  2. Run tests:           cargo test"
    echo "  3. Run E2E tests:       ./scripts/run-e2e-tests.sh"
    echo "  4. Deploy to testnet:   ./scripts/deploy_testnet.sh"
    echo ""
    print_success "Happy coding! 🚀"
}

# Run main function
main "$@"
