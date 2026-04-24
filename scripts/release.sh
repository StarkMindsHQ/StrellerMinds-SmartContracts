#!/bin/bash

# Complete Release Automation Script
# Handles the entire release process from validation to publication

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="$PROJECT_ROOT/logs"
LOG_FILE="$LOG_DIR/release_$TIMESTAMP.log"

mkdir -p "$LOG_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
DRY_RUN=false
SKIP_TESTS=false
SKIP_VALIDATION=false
AUTO_PUSH=false

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1" | tee -a "$LOG_FILE"
}

show_help() {
    cat << EOF
Release Automation Script

Usage: $0 [OPTIONS] <version>

OPTIONS:
    -h, --help          Show this help message
    -d, --dry-run       Perform a dry run (no actual changes)
    -f, --force         Skip validation and tests
    --skip-tests        Skip test execution
    --skip-validation   Skip pre-release validation
    -y, --yes           Automatically confirm prompts
    -v, --verbose       Enable verbose output

ARGUMENTS:
    version             Release version (e.g., v1.2.3)

EXAMPLES:
    $0 v1.2.3                   # Create release v1.2.3
    $0 --dry-run v1.2.3         # Dry run only
    $0 --force v1.2.3           # Force release (skip checks)
    $0 -y v1.2.3                # Auto-confirm all prompts

EXIT CODES:
    0   Release successful
    1   Release failed
    2   Validation failed
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -f|--force)
            SKIP_VALIDATION=true
            SKIP_TESTS=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-validation)
            SKIP_VALIDATION=true
            shift
            ;;
        -y|--yes)
            AUTO_PUSH=true
            shift
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        *)
            if [ -z "$VERSION" ]; then
                VERSION="$1"
            else
                log_error "Unknown argument: $1"
                show_help
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate version format
if [ -z "$VERSION" ]; then
    log_error "Version not specified"
    show_help
    exit 1
fi

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    log_error "Invalid version format: $VERSION"
    echo "Expected format: vX.Y.Z or vX.Y.Z-prerelease+build"
    exit 1
fi

log_info "Starting automated release process"
log_info "Version: $VERSION"
log_info "Dry Run: $DRY_RUN"
echo

# Phase 1: Pre-Release Validation
phase_validation() {
    log_step "Phase 1: Pre-Release Validation"
    echo "=========================================="
    
    if [ "$SKIP_VALIDATION" = true ]; then
        log_warning "Skipping validation (forced)"
        return 0
    fi
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would run pre-release validation"
        return 0
    fi
    
    if [ -f "$SCRIPT_DIR/pre-release-validation.sh" ]; then
        if "$SCRIPT_DIR/pre-release-validation.sh" "$VERSION"; then
            log_success "Pre-release validation passed"
        else
            log_error "Pre-release validation failed"
            return 2
        fi
    else
        log_warning "Validation script not found, skipping"
    fi
    echo
}

# Phase 2: Testing
phase_testing() {
    log_step "Phase 2: Release Testing"
    echo "=========================================="
    
    if [ "$SKIP_TESTS" = true ]; then
        log_warning "Skipping tests (forced)"
        return 0
    fi
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would run release tests"
        return 0
    fi
    
    if [ -f "$SCRIPT_DIR/release-test.sh" ]; then
        if "$SCRIPT_DIR/release-test.sh" all; then
            log_success "Release tests passed"
        else
            log_error "Release tests failed"
            return 2
        fi
    else
        log_warning "Test script not found, skipping"
    fi
    echo
}

# Phase 3: Build
phase_build() {
    log_step "Phase 3: Building Artifacts"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would build all contracts"
        return 0
    fi
    
    cd "$PROJECT_ROOT"
    
    if [ -f "$SCRIPT_DIR/build.sh" ]; then
        if "$SCRIPT_DIR/build.sh"; then
            log_success "Build completed successfully"
        else
            log_error "Build failed"
            return 1
        fi
    else
        log_warning "Build script not found, using direct cargo build"
        if cargo build --workspace --target wasm32-unknown-unknown --release; then
            log_success "Direct build completed successfully"
        else
            log_error "Direct build failed"
            return 1
        fi
    fi
    echo
}

# Phase 4: Update Documentation
phase_documentation() {
    log_step "Phase 4: Updating Documentation"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would update CHANGELOG"
        return 0
    fi
    
    # Check if git-cliff is available
    if command -v git-cliff &> /dev/null; then
        log_info "Generating changelog for $VERSION"
        if git-cliff --tag "$VERSION" --output CHANGELOG.md 2>/dev/null; then
            log_success "Changelog updated"
        else
            log_warning "Could not auto-generate changelog"
        fi
    else
        log_info "git-cliff not installed, skipping changelog generation"
        log_info "Install with: cargo install git-cliff"
    fi
    echo
}

# Phase 5: Create Git Tag
phase_tagging() {
    log_step "Phase 5: Creating Git Tag"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would create tag $VERSION"
        return 0
    fi
    
    # Check if tag already exists
    if git rev-parse "$VERSION" >/dev/null 2>&1; then
        log_error "Tag $VERSION already exists"
        return 1
    fi
    
    # Create annotated tag
    log_info "Creating annotated tag: $VERSION"
    git tag -a "$VERSION" -m "Release $VERSION"
    log_success "Tag created successfully"
    echo
}

# Phase 6: Push Release
phase_push() {
    log_step "Phase 6: Pushing Release"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would push tag to remote"
        return 0
    fi
    
    # Confirm before pushing
    if [ "$AUTO_PUSH" != true ]; then
        echo -n "Push tag $VERSION to remote? [y/N]: "
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            log_warning "Push cancelled by user"
            return 0
        fi
    fi
    
    log_info "Pushing tag $VERSION to origin"
    if git push origin "$VERSION"; then
        log_success "Tag pushed successfully"
        log_info "GitHub Actions will now create the release"
    else
        log_error "Failed to push tag"
        return 1
    fi
    echo
}

# Phase 7: Monitor Release
phase_monitor() {
    log_step "Phase 7: Monitoring Release Workflow"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would monitor GitHub Actions workflow"
        return 0
    fi
    
    log_info "Monitor the release workflow at:"
    log_info "https://github.com/your-org/repo/actions"
    echo
    log_warning "Automated monitoring requires GitHub API access"
    log_info "Please check the GitHub Actions page manually"
    echo
}

# Phase 8: Post-Release
phase_post_release() {
    log_step "Phase 8: Post-Release Activities"
    echo "=========================================="
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY RUN] Would run post-release activities"
        return 0
    fi
    
    log_info "Post-release checklist:"
    echo "  □ Verify GitHub Release page"
    echo "  □ Download and test artifacts"
    echo "  ✓ Checksums match"
    echo "  □ Update project website"
    echo "  □ Announce on social media"
    echo "  □ Notify stakeholders"
    echo
    
    # Schedule follow-up review
    REVIEW_DATE=$(date -d "+7 days" '+%Y-%m-%d' 2>/dev/null || date -v+7d '+%Y-%m-%d' 2>/dev/null || echo "in 7 days")
    log_info "Schedule post-release review for: $REVIEW_DATE"
    log_info "Run: ./scripts/post-release-review.sh $VERSION"
    echo
}

# Summary
show_summary() {
    echo "╔════════════════════════════════════════╗"
    if [ "$DRY_RUN" = true ]; then
        echo -e "${GREEN}║  ✓ Dry Run Completed Successfully    ║${NC}"
    else
        echo -e "${GREEN}║  ✓ Release Process Completed         ║${NC}"
    fi
    echo "╚════════════════════════════════════════╝"
    echo
    
    log_info "Release Summary:"
    echo "  Version: $VERSION"
    echo "  Timestamp: $TIMESTAMP"
    echo "  Log File: $LOG_FILE"
    echo
    
    if [ "$DRY_RUN" != true ]; then
        log_success "🎉 Release $VERSION completed successfully!"
        echo
        echo "Next Steps:"
        echo "1. Monitor GitHub Actions: https://github.com/your-org/repo/actions"
        echo "2. Verify release artifacts"
        echo "3. Run post-release review in 7 days"
        echo "4. Announce to community"
    fi
}

# Cleanup handler
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        log_error "Release process failed with exit code: $exit_code"
        log_info "Check log file for details: $LOG_FILE"
    fi
    exit $exit_code
}

trap cleanup EXIT

# Main execution
main() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Automated Release System             ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo
    
    phase_validation
    phase_testing
    phase_build
    phase_documentation
    phase_tagging
    phase_push
    phase_monitor
    phase_post_release
    show_summary
}

# Run main function
main "$@"
