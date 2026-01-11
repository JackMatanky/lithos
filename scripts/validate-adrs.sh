#!/bin/bash

# ADR Validation Script
# Checks for template compliance and section completeness in docs/adr/*.md

ADR_DIR="docs/adr"
EXIT_CODE=0

echo "🔍 Validating ADRs in $ADR_DIR..."

for adr in "$ADR_DIR"/[0-9][0-9][0-9][0-9]-*.md; do
    [ -e "$adr" ] || continue

    filename=$(basename "$adr")
    echo "  📄 Checking $filename..."

    # Check for required sections
    for section in "Status" "Context" "Decision" "Consequences"; do
        if ! grep -q "^#*.*$section" "$adr"; then
            echo "    ❌ Missing section: $section"
            EXIT_CODE=1
        fi
    done

    # Check for Status value (Proposed, Accepted, Superseded, Deprecated, Rejected)
    if ! grep -qiE "Status: (Proposed|Accepted|Superseded|Deprecated|Rejected)" "$adr"; then
        # Check if it's in a header instead
        if ! grep -qiE "^#+ Status" "$adr" || ! grep -qiE "(Proposed|Accepted|Superseded|Deprecated|Rejected)" "$adr"; then
            echo "    ⚠️ Status might be missing or invalid"
            # Not a hard failure yet but recommended
        fi
    fi
done

# Also check for files that don't match the naming convention
for f in "$ADR_DIR"/*.md; do
    filename=$(basename "$f")
    if [[ "$filename" == "adr-process.md" || "$filename" == "template.md" || "$filename" == "index.md" ]]; then
        continue
    fi
    if [[ ! "$filename" =~ ^[0-9]{4}-.*\.md$ ]]; then
        echo "    ❌ Invalid filename: $filename (Expected NNNN-name.md)"
        EXIT_CODE=1
    fi
done

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ All ADRs passed validation."
else
    echo "❌ Some ADRs failed validation."
fi

exit $EXIT_CODE
