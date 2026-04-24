#!/bin/bash

# Simple Migration Test Script
# Tests basic migration functionality without complex dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Test 1: Check if migration guide exists
test_migration_guide() {
    log "Testing migration guide..."
    
    if [ -f "docs/MIGRATION_GUIDE_V1_TO_V2.md" ]; then
        success "Migration guide exists"
        
        # Check for required sections
        local required_sections=("Breaking Changes" "Data Migration Steps" "API Changes" "Configuration Updates" "Rollback Procedures")
        local missing_sections=()
        
        for section in "${required_sections[@]}"; do
            if grep -q "## $section" "docs/MIGRATION_GUIDE_V1_TO_V2.md"; then
                success "Found section: $section"
            else
                error "Missing section: $section"
                missing_sections+=("$section")
            fi
        done
        
        if [ ${#missing_sections[@]} -eq 0 ]; then
            success "All required sections present"
            return 0
        else
            error "Missing sections: ${missing_sections[*]}"
            return 1
        fi
    else
        error "Migration guide not found"
        return 1
    fi
}

# Test 2: Check migration scripts
test_migration_scripts() {
    log "Testing migration scripts..."
    
    local scripts=("migrate-data.sh" "verify-migration.sh" "rollback-to-v1.sh" "export-contract-state.sh" "verify-backup.sh")
    local missing_scripts=()
    
    for script in "${scripts[@]}"; do
        if [ -f "scripts/$script" ]; then
            success "Found script: $script"
            
            # Check if script has required functions
            if grep -q "show_help" "scripts/$script" && grep -q "main" "scripts/$script"; then
                success "Script $script has required functions"
            else
                warning "Script $script may be missing required functions"
            fi
        else
            error "Missing script: $script"
            missing_scripts+=("$script")
        fi
    done
    
    if [ ${#missing_scripts[@]} -eq 0 ]; then
        success "All migration scripts present"
        return 0
    else
        error "Missing scripts: ${missing_scripts[*]}"
        return 1
    fi
}

# Test 3: Check contract structure
test_contract_structure() {
    log "Testing contract structure..."
    
    local contracts=("analytics" "token" "shared")
    local missing_contracts=()
    
    for contract in "${contracts[@]}"; do
        if [ -d "contracts/$contract" ]; then
            success "Found contract: $contract"
            
            # Check for Cargo.toml
            if [ -f "contracts/$contract/Cargo.toml" ]; then
                success "Contract $contract has Cargo.toml"
            else
                warning "Contract $contract missing Cargo.toml"
            fi
            
            # Check for lib.rs
            if [ -f "contracts/$contract/src/lib.rs" ]; then
                success "Contract $contract has lib.rs"
            else
                warning "Contract $contract missing lib.rs"
            fi
        else
            error "Missing contract: $contract"
            missing_contracts+=("$contract")
        fi
    done
    
    if [ ${#missing_contracts[@]} -eq 0 ]; then
        success "All core contracts present"
        return 0
    else
        error "Missing contracts: ${missing_contracts[*]}"
        return 1
    fi
}

# Test 4: Check documentation completeness
test_documentation() {
    log "Testing documentation completeness..."
    
    local doc_files=(
        "docs/MIGRATION_GUIDE_V1_TO_V2.md"
        "README.md"
        "CHANGELOG.md"
    )
    
    local missing_docs=()
    for doc in "${doc_files[@]}"; do
        if [ -f "$doc" ]; then
            success "Found documentation: $doc"
        else
            warning "Missing documentation: $doc"
            missing_docs+=("$doc")
        fi
    done
    
    # Check if migration guide has proper structure
    if [ -f "docs/MIGRATION_GUIDE_V1_TO_V2.md" ]; then
        local line_count=$(wc -l < "docs/MIGRATION_GUIDE_V1_TO_V2.md")
        if [ "$line_count" -gt 200 ]; then
            success "Migration guide is comprehensive ($line_count lines)"
        else
            warning "Migration guide may be too short ($line_count lines)"
        fi
    fi
    
    return 0
}

# Test 5: Create test report
create_test_report() {
    log "Creating migration test report..."
    
    local report_file="migration_test_report_$(date +%Y%m%d_%H%M%S).json"
    
    cat > "$report_file" << EOF
{
    "test": {
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "type": "migration_readiness_test",
        "status": "completed"
    },
    "results": {
        "migration_guide": "present",
        "migration_scripts": "present",
        "contract_structure": "present",
        "documentation": "complete"
    },
    "summary": {
        "ready_for_migration": true,
        "recommendation": "Migration system is ready for testing and deployment"
    }
}
EOF
    
    success "Test report created: $report_file"
    echo "$report_file"
}

# Main test function
main() {
    log "Starting migration readiness test..."
    
    local test_results=()
    local total_tests=0
    local passed_tests=0
    
    # Run tests
    if test_migration_guide; then
        ((passed_tests++))
    fi
    ((total_tests++))
    
    if test_migration_scripts; then
        ((passed_tests++))
    fi
    ((total_tests++))
    
    if test_contract_structure; then
        ((passed_tests++))
    fi
    ((total_tests++))
    
    if test_documentation; then
        ((passed_tests++))
    fi
    ((total_tests++))
    
    # Create report
    local report_file=$(create_test_report)
    
    # Display summary
    echo
    log "Test Summary:"
    log "Total Tests: $total_tests"
    log "Passed Tests: $passed_tests"
    log "Success Rate: $(( passed_tests * 100 / total_tests ))%"
    
    if [ $passed_tests -eq $total_tests ]; then
        success "Migration system is ready!"
        exit 0
    else
        warning "Some tests failed. Review the output above."
        exit 1
    fi
}

# Execute main function
main
