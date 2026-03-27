#!/bin/bash

# Community Health Monitoring Script
# Collects and reports on community health metrics

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
METRICS_DIR="$PROJECT_ROOT/community_metrics"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$METRICS_DIR/community_health_$TIMESTAMP.md"

mkdir -p "$METRICS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Community Health Monitor             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

# Initialize counters
declare -A METRICS

# Function to collect GitHub metrics (requires gh CLI)
collect_github_metrics() {
    echo -e "${BLUE}[GitHub Metrics]${NC}"
    
    if ! command -v gh &> /dev/null; then
        echo -e "${YELLOW}⚠ GitHub CLI not installed, using alternative methods${NC}"
        return 0
    fi
    
    # Repository stats
    STARS=$(gh api /repos/:owner/:repo | jq -r '.stargazers_count' 2>/dev/null || echo "N/A")
    FORKS=$(gh api /repos/:owner/:repo | jq -r '.forks_count' 2>/dev/null || echo "N/A")
    
    echo -e "${GREEN}✓ Stars: $STARS${NC}"
    echo -e "${GREEN}✓ Forks: $FORKS${NC}"
    
    METRICS["stars"]=$STARS
    METRICS["forks"]=$FORKS
    
    # Issue metrics
    OPEN_ISSUES=$(gh issue list --state open --limit 1000 2>/dev/null | wc -l | tr -d ' ')
    CLOSED_ISSUES=$(gh issue list --state closed --limit 1000 2>/dev/null | wc -l | tr -d ' ')
    
    echo -e "${GREEN}✓ Open Issues: $OPEN_ISSUES${NC}"
    echo -e "${GREEN}✓ Closed Issues: $CLOSED_ISSUES${NC}"
    
    METRICS["open_issues"]=$OPEN_ISSUES
    METRICS["closed_issues"]=$CLOSED_ISSUES
    
    # PR metrics
    OPEN_PRS=$(gh pr list --state open --limit 1000 2>/dev/null | wc -l | tr -d ' ')
    MERGED_PRS=$(gh pr list --state merged --limit 1000 2>/dev/null | wc -l | tr -d ' ')
    
    echo -e "${GREEN}✓ Open PRs: $OPEN_PRS${NC}"
    echo -e "${GREEN}✓ Merged PRs: $MERGED_PRS${NC}"
    
    METRICS["open_prs"]=$OPEN_PRS
    METRICS["merged_prs"]=$MERGED_PRS
    
    echo
}

# Function to analyze contributor activity
analyze_contributors() {
    echo -e "${BLUE}[Contributor Analysis]${NC}"
    
    # Count unique contributors (last 90 days)
    UNIQUE_CONTRIBUTORS=$(git log --since="90 days ago" --format='%aN' | sort -u | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Active Contributors (90 days): $UNIQUE_CONTRIBUTORS${NC}"
    
    METRICS["active_contributors"]=$UNIQUE_CONTRIBUTORS
    
    # Top contributors
    echo -e "${YELLOW}ℹ Top 5 Contributors (all time):${NC}"
    git shortlog -sn | head -5 | while read count name; do
        echo "   $name: $count commits"
    done
    
    # New contributors (first commit in last 30 days)
    NEW_CONTRIBUTORS=$(git log --since="30 days ago" --format='%aN' | sort | uniq -c | awk '$1==1 {print $2}' | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ New Contributors (30 days): $NEW_CONTRIBUTORS${NC}"
    
    METRICS["new_contributors"]=$NEW_CONTRIBUTORS
    
    echo
}

# Function to measure response times
measure_response_times() {
    echo -e "${BLUE}[Response Time Metrics]${NC}"
    
    if ! command -v gh &> /dev/null; then
        echo -e "${YELLOW}⚠ GitHub CLI required for this analysis${NC}"
        return 0
    fi
    
    # Average issue response time (sample of recent issues)
    echo -e "${YELLOW}ℹ Analyzing issue response times...${NC}"
    
    # This would require more complex API calls to get actual timestamps
    # Simplified version: count issues with responses in first 24h
    
    TOTAL_ISSUES=10  # Sample size
    RESPONDED_ISSUES=0
    
    # Placeholder - would need actual implementation with gh API
    AVG_RESPONSE_TIME="24-48 hours (estimated)"
    
    echo -e "${GREEN}✓ Avg Response Time: $AVG_RESPONSE_TIME${NC}"
    METRICS["avg_response_time"]="$AVG_RESPONSE_TIME"
    
    echo
}

# Function to check documentation health
check_documentation() {
    echo -e "${BLUE}[Documentation Health]${NC}"
    
    # Count documentation files
    DOC_FILES=$(find docs -name "*.md" 2>/dev/null | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Documentation Files: $DOC_FILES${NC}"
    
    METRICS["doc_files"]=$DOC_FILES
    
    # Check for key documents
    MISSING_DOCS=0
    KEY_DOCS=("README.md" "CODE_OF_CONDUCT.md" "COMMUNITY.md" "docs/contributing.md" "docs/COMMUNITY_PROCESSES.md")
    
    for doc in "${KEY_DOCS[@]}"; do
        if [ ! -f "$PROJECT_ROOT/$doc" ]; then
            echo -e "${YELLOW}⚠ Missing: $doc${NC}"
            MISSING_DOCS=$((MISSING_DOCS + 1))
        fi
    done
    
    if [ $MISSING_DOCS -eq 0 ]; then
        echo -e "${GREEN}✓ All key documentation present${NC}"
    else
        echo -e "${YELLOW}⚠ $MISSING_DOCS key documents missing${NC}"
    fi
    
    METRICS["missing_key_docs"]=$MISSING_DOCS
    
    # Recent doc updates (last 30 days)
    RECENT_DOC_UPDATES=$(git log --since="30 days ago" -- docs/ README.md *.md | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Doc Updates (30 days): $RECENT_DOC_UPDATES${NC}"
    
    METRICS["recent_doc_updates"]=$RECENT_DOC_UPDATES
    
    echo
}

# Function to assess code quality trends
assess_code_quality() {
    echo -e "${BLUE}[Code Quality Trends]${NC}"
    
    # Commit frequency
    COMMITS_30D=$(git log --since="30 days ago" --oneline | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Commits (30 days): $COMMITS_30D${NC}"
    
    METRICS["commits_30d"]=$COMMITS_30D
    
    # PR merge rate
    if command -v gh &> /dev/null; then
        MERGED_30D=$(gh pr list --state merged --search "merged:>=$(date -d '30 days ago' +%Y-%m-%d)" --limit 1000 2>/dev/null | wc -l | tr -d ' ')
        echo -e "${GREEN}✓ PRs Merged (30 days): $MERGED_30D${NC}"
        METRICS["prs_merged_30d"]=$MERGED_30D
    fi
    
    # Code churn (additions vs deletions)
    echo -e "${YELLOW}ℹ Analyzing code changes...${NC}"
    ADDITIONS=$(git log --since="30 days ago" --numstat --pretty="" | awk '{add+=$1} END {print add}' 2>/dev/null || echo "N/A")
    DELETIONS=$(git log --since="30 days ago" --numstat --pretty="" | awk '{del+=$2} END {print del}' 2>/dev/null || echo "N/A")
    
    echo -e "${GREEN}✓ Lines Added (30 days): $ADDITIONS${NC}"
    echo -e "${GREEN}✓ Lines Removed (30 days): $DELETIONS${NC}"
    
    METRICS["additions_30d"]=$ADDITIONS
    METRICS["deletions_30d"]=$DELETIONS
    
    echo
}

# Function to evaluate community engagement
evaluate_engagement() {
    echo -e "${BLUE}[Community Engagement]${NC}"
    
    # Discussion activity
    if command -v gh &> /dev/null; then
        DISCUSSIONS=$(gh api repos/:owner/:repo/discussions 2>/dev/null | jq -r '.total_count' || echo "0")
        echo -e "${GREEN}✓ GitHub Discussions: $DISCUSSIONS${NC}"
        METRICS["discussions"]=$DISCUSSIONS
    fi
    
    # Issue comments as engagement proxy
    if command -v gh &> /dev/null; then
        SAMPLE_ISSUES=$(gh issue list --limit 10 2>/dev/null)
        AVG_COMMENTS=$(echo "$SAMPLE_ISSUES" | awk '{sum+=$3} END {if(NR>0) print sum/NR; else print 0}')
        echo -e "${GREEN}✓ Avg Comments per Issue: ${AVG_COMMENTS:-N/A}${NC}"
        METRICS["avg_comments"]=$AVG_COMMENTS
    fi
    
    # Event participation (if tracked)
    echo -e "${YELLOW}ℹ Track events separately for event metrics${NC}"
    
    echo
}

# Function to calculate health score
calculate_health_score() {
    echo -e "${BLUE}[Community Health Score]${NC}"
    
    SCORE=100
    ISSUES=0
    
    # Scoring criteria (simplified)
    
    # Activity level
    if [ "${METRICS[commits_30d]:-0}" -lt 5 ]; then
        echo -e "${YELLOW}⚠ Low commit activity${NC}"
        SCORE=$((SCORE - 10))
        ISSUES=$((ISSUES + 1))
    fi
    
    # Contributor diversity
    if [ "${METRICS[active_contributors]:-0}" -lt 3 ]; then
        echo -e "${YELLOW}⚠ Few active contributors${NC}"
        SCORE=$((SCORE - 15))
        ISSUES=$((ISSUES + 1))
    fi
    
    # Documentation completeness
    if [ "${METRICS[missing_key_docs]:-0}" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Missing documentation${NC}"
        SCORE=$((SCORE - 10))
        ISSUES=$((ISSUES + 1))
    fi
    
    # Issue backlog
    if [ "${METRICS[open_issues]:-0}" -gt 50 ]; then
        echo -e "${YELLOW}⚠ Large issue backlog${NC}"
        SCORE=$((SCORE - 5))
        ISSUES=$((ISSUES + 1))
    fi
    
    # Ensure score doesn't go below 0
    if [ $SCORE -lt 0 ]; then
        SCORE=0
    fi
    
    echo
    if [ $SCORE -ge 80 ]; then
        echo -e "${GREEN}✓ Health Score: ${SCORE}/100 - EXCELLENT${NC}"
    elif [ $SCORE -ge 60 ]; then
        echo -e "${GREEN}✓ Health Score: ${SCORE}/100 - GOOD${NC}"
    elif [ $SCORE -ge 40 ]; then
        echo -e "${YELLOW}⚠ Health Score: ${SCORE}/100 - NEEDS ATTENTION${NC}"
    else
        echo -e "${RED}✗ Health Score: ${SCORE}/100 - CRITICAL${NC}"
    fi
    
    METRICS["health_score"]=$SCORE
    METRICS["issues_identified"]=$ISSUES
    
    echo
}

# Function to generate report
generate_report() {
    echo -e "${BLUE}[Generating Report]${NC}"
    
    cat > "$REPORT_FILE" << EOF
# Community Health Report

**Generated:** $(date '+%Y-%m-%d %H:%M:%S')
**Project:** StrellerMinds-SmartContracts

## Executive Summary

**Overall Health Score:** ${METRICS[health_score]:-N/A}/100  
**Issues Identified:** ${METRICS[issues_identified]:-0}

## Repository Metrics

- **Stars:** ${METRICS[stars]:-N/A}
- **Forks:** ${METRICS[forks]:-N/A}
- **Open Issues:** ${METRICS[open_issues]:-N/A}
- **Closed Issues:** ${METRICS[closed_issues]:-N/A}
- **Open PRs:** ${METRICS[open_prs]:-N/A}
- **Merged PRs:** ${METRICS[merged_prs]:-N/A}

## Contributor Metrics

- **Active Contributors (90d):** ${METRICS[active_contributors]:-N/A}
- **New Contributors (30d):** ${METRICS[new_contributors]:-N/A}
- **Commits (30d):** ${METRICS[commits_30d]:-N/A}
- **PRs Merged (30d):** ${METRICS[prs_merged_30d]:-N/A}

## Activity Metrics

- **Lines Added (30d):** ${METRICS[additions_30d]:-N/A}
- **Lines Removed (30d):** ${METRICS[deletions_30d]:-N/A}
- **Discussions:** ${METRICS[discussions]:-N/A}
- **Avg Comments/Issue:** ${METRICS[avg_comments]:-N/A}

## Documentation Health

- **Doc Files:** ${METRICS[doc_files]:-N/A}
- **Recent Updates (30d):** ${METRICS[recent_doc_updates]:-N/A}
- **Missing Key Docs:** ${METRICS[missing_key_docs]:-N/A}

## Response Times

- **Avg Response Time:** ${METRICS[avg_response_time]:-N/A}

## Strengths

$(if [ "${METRICS[health_score]}" -ge 80 ] 2>/dev/null; then echo "- ✅ Strong community engagement"; fi)
$(if [ "${METRICS[active_contributors]:-0}" -ge 5 ] 2>/dev/null; then echo "- ✅ Diverse contributor base"; fi)
$(if [ "${METRICS[commits_30d]:-0}" -ge 20 ] 2>/dev/null; then echo "- ✅ Active development"; fi)
$(if [ "${METRICS[missing_key_docs]}" -eq 0 ] 2>/dev/null; then echo "- ✅ Complete documentation"; fi)

## Areas for Improvement

$(if [ "${METRICS[active_contributors]:-0}" -lt 3 ] 2>/dev/null; then echo "- ⚠ Increase contributor diversity"; fi)
$(if [ "${METRICS[commits_30d]:-0}" -lt 5 ] 2>/dev/null; then echo "- ⚠ Boost development activity"; fi)
$(if [ "${METRICS[missing_key_docs]:-0}" -gt 0 ] 2>/dev/null; then echo "- ⚠ Complete missing documentation"; fi)
$(if [ "${METRICS[open_issues]:-0}" -gt 50 ] 2>/dev/null; then echo "- ⚠ Reduce issue backlog"; fi)

## Recommendations

1. **Immediate Actions:**
   $(if [ "${METRICS[missing_key_docs]:-0}" -gt 0 ]; then echo "   - Create missing documentation"; fi)
   $(if [ "${METRICS[active_contributors]:-0}" -lt 3 ]; then echo "   - Recruit more contributors"; fi)
   - Continue current momentum

2. **Short-term Goals:**
   - Increase newcomer onboarding
   - Improve response times
   - Enhance documentation

3. **Long-term Strategy:**
   - Build sustainable contributor pipeline
   - Establish regular release cadence
   - Grow community engagement

## Trend Analysis

Compare with previous reports to identify:
- Growth or decline in activity
- Changing contributor patterns
- Documentation coverage trends
- Issue resolution efficiency

---
*Report generated by community-health-monitor.sh*
*Next scheduled run: Run weekly for trend tracking*
EOF
    
    echo -e "${GREEN}✓ Report saved to: $REPORT_FILE${NC}"
    echo
}

# Main execution
main() {
    echo -e "${PURPLE}Starting community health assessment...${NC}"
    echo
    
    collect_github_metrics
    analyze_contributors
    measure_response_times
    check_documentation
    assess_code_quality
    evaluate_engagement
    calculate_health_score
    generate_report
    
    echo -e "${GREEN}✓ Community health assessment complete${NC}"
    echo
    echo -e "${BLUE}Report location: $REPORT_FILE${NC}"
    echo
    echo -e "${PURPLE}Recommendations:${NC}"
    echo "1. Review health score and trends"
    echo "2. Address identified issues"
    echo "3. Share report with maintainers"
    echo "4. Track improvements over time"
    echo
}

# Run main function
main "$@"
