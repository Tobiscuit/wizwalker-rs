#!/usr/bin/env bash
# check-jules-status.sh — Check the status of all Jules sessions
#
# Usage: ./check-jules-status.sh
#
# Shows the current state of all Jules tasks (pending/working/completed/failed).

set -euo pipefail

if [[ -z "${JULES_API_KEY:-}" ]]; then
    echo "❌ JULES_API_KEY not set. Export it first:"
    echo "   export JULES_API_KEY=your-key-here"
    exit 1
fi

echo "🧙 Checking Jules session status..."
echo ""

# Fetch sessions via the Jules REST API
RESPONSE=$(curl -s \
    -H "x-goog-api-key: $JULES_API_KEY" \
    "https://jules.googleapis.com/v1alpha/sessions?pageSize=20")

# Parse and display
echo "$RESPONSE" | jq -r '
    .sessions // [] | sort_by(.createTime) | .[] |
    "[\(.state // "UNKNOWN" | 
        if . == "COMPLETED" then "✅"
        elif . == "WORKING" then "⏳"
        elif . == "FAILED" then "❌"
        elif . == "PENDING" then "🕐"
        else "❓"
        end
    )] \(.title // "Untitled") — \(.state // "unknown")"
'

echo ""

# Summary counts
TOTAL=$(echo "$RESPONSE" | jq '.sessions // [] | length')
COMPLETED=$(echo "$RESPONSE" | jq '[.sessions // [] | .[] | select(.state == "COMPLETED")] | length')
WORKING=$(echo "$RESPONSE" | jq '[.sessions // [] | .[] | select(.state == "WORKING")] | length')
FAILED=$(echo "$RESPONSE" | jq '[.sessions // [] | .[] | select(.state == "FAILED")] | length')
PENDING=$(echo "$RESPONSE" | jq '[.sessions // [] | .[] | select(.state == "PENDING")] | length')

echo "═══════════════════════════════════════"
echo "  📊 Total:     $TOTAL"
echo "  ✅ Completed: $COMPLETED"
echo "  ⏳ Working:   $WORKING"
echo "  🕐 Pending:   $PENDING"
echo "  ❌ Failed:    $FAILED"
echo "═══════════════════════════════════════"
