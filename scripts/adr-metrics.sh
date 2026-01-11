#!/bin/bash

# ADR Metrics Script
# Generates a summary of ADR counts and quality metrics

ADR_DIR="docs/adr"
TOTAL_ADRS=0
ACCEPTED=0
PROPOSED=0
SUPERSEDED=0
REJECTED=0
DEPRECATED=0
COMPLETE=0

echo "📊 ADR Quality Metrics for Lithos..."
echo "-----------------------------------"

for adr in "$ADR_DIR"/[0-9][0-9][0-9][0-9]-*.md; do
    [ -e "$adr" ] || continue
    ((TOTAL_ADRS++))

    status=$(grep -oE "\*   \*\*Status\*\*:[[:space:]]*[A-Za-z]+" "$adr" | awk -F': ' '{print $2}')

    case "$status" in
    Accepted) ((ACCEPTED++)) ;;
    Proposed) ((PROPOSED++)) ;;
    Superseded) ((SUPERSEDED++)) ;;
    Rejected) ((REJECTED++)) ;;
    Deprecated) ((DEPRECATED++)) ;;
    esac

    # Check for completeness (all 6 headers present)
    missing=0
    for section in "Context" "Decision" "Alternatives Considered" "Technical Validation" "Consequences" "Status Tracking"; do
        if ! grep -q "^## $section" "$adr"; then
            ((missing++))
        fi
    done

    if [ $missing -eq 0 ]; then
        ((COMPLETE++))
    fi
done

echo "Total ADRs: $TOTAL_ADRS"
echo "  Accepted:   $ACCEPTED"
echo "  Proposed:   $PROPOSED"
echo "  Other:      $((SUPERSEDED + REJECTED + DEPRECATED))"
echo ""
echo "Quality Gate Status:"

if [ $TOTAL_ADRS -eq 0 ]; then
    COMPLETENESS=0
else
    COMPLETENESS=$(((COMPLETE * 100) / TOTAL_ADRS))
fi

echo "  Completeness: $COMPLETENESS% ($COMPLETE/$TOTAL_ADRS files meet template standards)"

if [ $COMPLETENESS -lt 100 ]; then
    echo "  ⚠️ Action Required: Some ADRs are missing required sections."
else
    echo "  ✅ All ADRs meet the gold standard."
fi
echo "-----------------------------------"
