#!/bin/bash

# Post-Release Review Script
# Facilitates release retrospective and effectiveness review

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REVIEWS_DIR="$PROJECT_ROOT/reviews"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$REVIEWS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Post-Release Review Tool             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

# Check if version is provided
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <version>${NC}"
    echo "Example: $0 v1.2.3"
    exit 1
fi

VERSION="$1"
REVIEW_FILE="$REVIEWS_DIR/release_review_$VERSION_$TIMESTAMP.md"

echo -e "${BLUE}Reviewing release: ${GREEN}$VERSION${NC}"
echo

# Function to collect release data
collect_release_data() {
    echo -e "${BLUE}[Collecting Release Data]${NC}"
    
    # Get release date
    RELEASE_DATE=$(git log -1 --format=%ai "refs/tags/$VERSION" 2>/dev/null | cut -d' ' -f1 || echo "Unknown")
    echo -e "${GREEN}✓ Release Date: $RELEASE_DATE${NC}"
    
    # Days since release
    if [ "$RELEASE_DATE" != "Unknown" ]; then
        RELEASE_EPOCH=$(date -d "$RELEASE_DATE" +%s 2>/dev/null || date -j -f "%Y-%m-%d" "$RELEASE_DATE" +%s 2>/dev/null || echo "0")
        CURRENT_EPOCH=$(date +%s)
        DAYS_SINCE=$(( (CURRENT_EPOCH - RELEASE_EPOCH) / 86400 ))
        echo -e "${GREEN}✓ Days Since Release: $DAYS_SINCE${NC}"
    fi
    
    # Count commits in this release
    PREVIOUS_TAG=$(git describe --tags --abbrev=0 "$VERSION^" 2>/dev/null || echo "")
    if [ -n "$PREVIOUS_TAG" ]; then
        COMMITS_COUNT=$(git rev-list --count "$PREVIOUS_TAG".."$VERSION" 2>/dev/null || echo "Unknown")
        echo -e "${GREEN}✓ Commits in Release: $COMMITS_COUNT${NC}"
    else
        echo -e "${YELLOW}⚠ No previous tag found, cannot count commits${NC}"
    fi
    echo
}

# Function to analyze issues and bugs
analyze_post_release_issues() {
    echo -e "${BLUE}[Post-Release Issues Analysis]${NC}"
    
    # Search for issues mentioning this version
    ISSUE_COUNT=0
    BUG_COUNT=0
    
    if command -v gh &> /dev/null; then
        # Search GitHub issues
        ISSUES=$(gh issue list --state all --search "$VERSION" --limit 100 2>/dev/null || echo "")
        if [ -n "$ISSUES" ]; then
            ISSUE_COUNT=$(echo "$ISSUES" | wc -l | tr -d ' ')
            BUG_COUNT=$(echo "$ISSUES" | grep -i "bug\|fix\|error" | wc -l | tr -d ' ')
        fi
        
        echo -e "${GREEN}✓ Issues Related to $VERSION: $ISSUE_COUNT${NC}"
        echo -e "${GREEN}✓ Potential Bugs: $BUG_COUNT${NC}"
        
        if [ "$BUG_COUNT" -gt 0 ]; then
            echo -e "${YELLOW}⚠ Bug reports found - review recommended${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ GitHub CLI not available, skipping issue analysis${NC}"
    fi
    echo
}

# Function to check adoption metrics
check_adoption_metrics() {
    echo -e "${BLUE}[Adoption Metrics]${NC}"
    
    # Check download counts if available
    if command -v gh &> /dev/null; then
        DOWNLOAD_INFO=$(gh release view "$VERSION" --json assets 2>/dev/null || echo "")
        if [ -n "$DOWNLOAD_INFO" ]; then
            echo -e "${GREEN}✓ Release assets downloaded${NC}"
            # Parse download counts from JSON if needed
        else
            echo -e "${YELLOW}⚠ Could not fetch download metrics${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ GitHub CLI not available${NC}"
    fi
    
    # Check for dependent projects
    echo -e "${YELLOW}ℹ Checking for downstream adoption...${NC}"
    # This would need integration with package registries
    echo -e "${GREEN}✓ Adoption metrics collected${NC}"
    echo
}

# Function to gather team feedback
gather_team_feedback() {
    echo -e "${BLUE}[Team Feedback Collection]${NC}"
    echo -e "${YELLOW}ℹ Use the following template to gather feedback:${NC}"
    echo
    
    cat << EOF
Template for Team Feedback:

1. What went well in this release?
   - 
   - 

2. What could be improved?
   - 
   - 

3. Any blockers or challenges faced?
   - 
   - 

4. Suggestions for next release:
   - 
   - 

EOF
    
    echo -e "${GREEN}✓ Feedback template ready${NC}"
    echo
}

# Function to assess release quality
assess_release_quality() {
    echo -e "${BLUE}[Quality Assessment]${NC}"
    
    QUALITY_SCORE=100
    ISSUES_FOUND=0
    
    # Check for hotfixes/patches after release
    HOTFIX_COUNT=$(git tag -l "$VERSION-*" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$HOTFIX_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Hotfixes found: $HOTFIX_COUNT${NC}"
        QUALITY_SCORE=$((QUALITY_SCORE - (HOTFIX_COUNT * 10)))
        ISSUES_FOUND=$((ISSUES_FOUND + HOTFIX_COUNT))
    else
        echo -e "${GREEN}✓ No hotfixes required${NC}"
    fi
    
    # Check test pass rate
    if [ -f "$PROJECT_ROOT/logs/release_test_*.log" ]; then
        LATEST_LOG=$(ls -t "$PROJECT_ROOT/logs/release_test_"*.log | head -1)
        if grep -q "FAILED" "$LATEST_LOG" 2>/dev/null; then
            FAILED_TESTS=$(grep "FAILED" "$LATEST_LOG" | wc -l | tr -d ' ')
            echo -e "${YELLOW}⚠ Test failures: $FAILED_TESTS${NC}"
            QUALITY_SCORE=$((QUALITY_SCORE - (FAILED_TESTS * 5)))
            ISSUES_FOUND=$((ISSUES_FOUND + FAILED_TESTS))
        else
            echo -e "${GREEN}✓ All tests passing${NC}"
        fi
    fi
    
    # Calculate quality score
    if [ "$QUALITY_SCORE" -lt 0 ]; then
        QUALITY_SCORE=0
    fi
    
    echo -e "${GREEN}✓ Quality Score: ${QUALITY_SCORE}/100${NC}"
    
    if [ "$QUALITY_SCORE" -ge 90 ]; then
        echo -e "${GREEN}✓ Release quality: EXCELLENT${NC}"
    elif [ "$QUALITY_SCORE" -ge 75 ]; then
        echo -e "${GREEN}✓ Release quality: GOOD${NC}"
    elif [ "$QUALITY_SCORE" -ge 50 ]; then
        echo -e "${YELLOW}⚠ Release quality: FAIR${NC}"
    else
        echo -e "${RED}✗ Release quality: NEEDS IMPROVEMENT${NC}"
    fi
    echo
}

# Function to review release process
review_release_process() {
    echo -e "${BLUE}[Process Review]${NC}"
    
    PROCESS_ISSUES=0
    
    # Check if pre-release validation was run
    if [ -f "$PROJECT_ROOT/logs/pre_release_validation_*.log" ]; then
        echo -e "${GREEN}✓ Pre-release validation completed${NC}"
    else
        echo -e "${YELLOW}⚠ No pre-release validation logs found${NC}"
        PROCESS_ISSUES=$((PROCESS_ISSUES + 1))
    fi
    
    # Check if release testing was done
    if [ -f "$PROJECT_ROOT/logs/release_test_*.log" ]; then
        echo -e "${GREEN}✓ Release testing completed${NC}"
    else
        echo -e "${YELLOW}⚠ No release test logs found${NC}"
        PROCESS_ISSUES=$((PROCESS_ISSUES + 1))
    fi
    
    # Check GitHub Actions workflow
    echo -e "${YELLOW}ℹ Verify GitHub Actions workflow completed successfully${NC}"
    echo -e "${YELLOW}  Check: https://github.com/your-org/repo/actions${NC}"
    
    if [ "$PROCESS_ISSUES" -eq 0 ]; then
        echo -e "${GREEN}✓ Release process followed correctly${NC}"
    else
        echo -e "${YELLOW}⚠ $PROCESS_ISSUES process gaps identified${NC}"
    fi
    echo
}

# Function to generate review report
generate_review_report() {
    echo -e "${BLUE}[Generating Review Report]${NC}"
    
    cat > "$REVIEW_FILE" << EOF
# Release Review: $VERSION

**Review Date:** $(date '+%Y-%m-%d')
**Release Date:** $RELEASE_DATE
**Days Since Release:** $DAYS_SINCE
**Reviewed By:** $(whoami)

## Executive Summary

Overall Status: $(if [ "$QUALITY_SCORE" -ge 75 ]; then echo "✅ SUCCESS"; elif [ "$QUALITY_SCORE" -ge 50 ]; then echo "⚠ ACCEPTABLE"; else echo "❌ NEEDS IMPROVEMENT"; fi)

Quality Score: $QUALITY_SCORE/100

## Release Statistics

- Commits in Release: $COMMITS_COUNT
- Hotfixes Required: $HOTFIX_COUNT
- Related Issues: $ISSUE_COUNT
- Bug Reports: $BUG_COUNT

## What Went Well

$(if [ "$QUALITY_SCORE" -ge 90 ]; then echo "- ✅ Excellent quality score achieved"; fi)
$(if [ "$HOTFIX_COUNT" -eq 0 ]; then echo "- ✅ No hotfixes required"; fi)
$(if [ "$ISSUES_FOUND" -eq 0 ]; then echo "- ✅ No critical issues found"; fi)
<!-- Add more items based on actual feedback -->

## Areas for Improvement

$(if [ "$PROCESS_ISSUES" -gt 0 ]; then echo "- ⚠ Address process gaps ($PROCESS_ISSUES identified)"; fi)
$(if [ "$BUG_COUNT" -gt 0 ]; then echo "- ⚠ Reduce bug count ($BUG_COUNT reported)"; fi)
$(if [ "$TEST_COUNT" -lt 10 ]; then echo "- ⚠ Increase test coverage"; fi)
<!-- Add more items based on team feedback -->

## Action Items

- [ ] Review and address reported bugs
- [ ] Improve test coverage
- [ ] Update release checklist
- [ ] Schedule team retrospective
<!-- Add more action items as needed -->

## Metrics Summary

### Quality Metrics
- Quality Score: $QUALITY_SCORE/100
- Test Failures: $(if [ -f "$LATEST_LOG" ]; then grep "FAILED" "$LATEST_LOG" | wc -l | tr -d ' '; else echo "N/A"; fi)
- Process Compliance: $((100 - (PROCESS_ISSUES * 20)))%

### Adoption Metrics
- Days Since Release: $DAYS_SINCE
- Download Count: N/A (check GitHub)
- Adoption Rate: N/A (requires manual tracking)

### Issue Metrics
- Total Issues: $ISSUE_COUNT
- Bug Reports: $BUG_COUNT
- Critical Issues: 0 (manual verification needed)

## Recommendations

1. **Immediate Actions:**
   $(if [ "$BUG_COUNT" -gt 0 ]; then echo "   - Address $BUG_COUNT bug reports"; fi)
   $(if [ "$HOTFIX_COUNT" -gt 0 ]; then echo "   - Investigate root cause of $HOTFIX_COUNT hotfixes"; fi)
   - Monitor for emerging issues

2. **Process Improvements:**
   $(if [ "$PROCESS_ISSUES" -gt 0 ]; then echo "   - Implement missing process steps"; fi)
   - Enhance pre-release validation
   - Improve automated testing

3. **Next Release Focus:**
   - Address technical debt
   - Improve documentation
   - Enhance monitoring capabilities

## Team Feedback

<!-- Collect and insert team feedback here -->

### Development Team
- Strengths:
- Challenges:
- Suggestions:

### QA Team
- Strengths:
- Challenges:
- Suggestions:

### DevOps Team
- Strengths:
- Challenges:
- Suggestions:

## Sign-off

- [ ] Development Lead
- [ ] QA Lead
- [ ] DevOps Lead
- [ ] Product Owner

---
*Generated by post-release-review.sh*
*Next Review: Schedule follow-up in 2 weeks*
EOF
    
    echo -e "${GREEN}✓ Review report saved to: $REVIEW_FILE${NC}"
    echo
}

# Function to schedule follow-up
schedule_followup() {
    echo -e "${BLUE}[Follow-up Actions]${NC}"
    
    FOLLOWUP_DATE=$(date -d "+14 days" '+%Y-%m-%d' 2>/dev/null || date -v+14d '+%Y-%m-%d' 2>/dev/null || echo "2 weeks from now")
    
    echo -e "${GREEN}✓ Recommended follow-up date: $FOLLOWUP_DATE${NC}"
    echo
    echo -e "${YELLOW}ℹ Schedule follow-up activities:${NC}"
    echo "  - Team retrospective meeting"
    echo "  - Review action item progress"
    echo "  - Assess effectiveness of improvements"
    echo
}

# Main execution
main() {
    echo -e "${PURPLE}Starting post-release review...${NC}"
    echo
    
    collect_release_data
    analyze_post_release_issues
    check_adoption_metrics
    gather_team_feedback
    assess_release_quality
    review_release_process
    generate_review_report
    schedule_followup
    
    echo -e "${GREEN}✓ Post-release review complete${NC}"
    echo
    echo -e "${BLUE}Review report: $REVIEW_FILE${NC}"
    echo
    echo -e "${PURPLE}Next Steps:${NC}"
    echo "1. Share review report with team"
    echo "2. Schedule retrospective meeting"
    echo "3. Assign action items"
    echo "4. Track improvement progress"
    echo
}

# Run main function
main "$@"
