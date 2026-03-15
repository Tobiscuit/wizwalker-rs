#!/usr/bin/env bash
# reply-to-jules.sh — Send a reply to a Jules session that's awaiting feedback
#
# Usage:
#   ./reply-to-jules.sh <session-id> "Your reply message"
#   ./reply-to-jules.sh auto "Default proceed message"   (replies to ALL waiting sessions)
#
# Examples:
#   ./reply-to-jules.sh 841505713017083886 "Yes, proceed with that approach."
#   ./reply-to-jules.sh auto

set -euo pipefail

if [[ -z "${JULES_API_KEY:-}" ]]; then
    echo "❌ JULES_API_KEY not set. Export it first:"
    echo "   export JULES_API_KEY=your-key-here"
    exit 1
fi

SESSION_ID="${1:-}"
MESSAGE="${2:-}"

DEFAULT_PROCEED="Yes, your approach is correct. Please proceed, run cargo check to fix any compile errors, then submit the PR."

if [[ "$SESSION_ID" == "auto" ]]; then
    echo "🔍 Finding all sessions awaiting feedback..."

    RESPONSE=$(curl -s \
        -H "x-goog-api-key: $JULES_API_KEY" \
        "https://jules.googleapis.com/v1alpha/sessions?pageSize=50")

    WAITING=$(echo "$RESPONSE" | python3 -c "
import sys, json
d = json.load(sys.stdin)
for s in d.get('sessions', []):
    if s.get('state') == 'AWAITING_USER_FEEDBACK':
        print(s['id'] + '|||' + s.get('title', 'Untitled'))
")

    if [[ -z "$WAITING" ]]; then
        echo "✅ No sessions currently awaiting feedback."
        exit 0
    fi

    while IFS='|||' read -r sid stitle; do
        echo "🙋 Replying to: $stitle (ID: $sid)"
        REPLY="${MESSAGE:-$DEFAULT_PROCEED}"
        HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
            -H "x-goog-api-key: $JULES_API_KEY" \
            -H "Content-Type: application/json" \
            -d "{\"prompt\": \"$REPLY\"}" \
            "https://jules.googleapis.com/v1alpha/sessions/${sid}:sendMessage")
        if [[ "$HTTP" == "200" ]]; then
            echo "   ✅ Replied — Jules will continue"
        else
            echo "   ❌ Failed (HTTP $HTTP)"
        fi
    done <<< "$WAITING"

elif [[ -n "$SESSION_ID" ]]; then
    REPLY="${MESSAGE:-$DEFAULT_PROCEED}"
    echo "📨 Sending reply to session $SESSION_ID..."
    echo "   Message: \"$REPLY\""

    HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
        -H "x-goog-api-key: $JULES_API_KEY" \
        -H "Content-Type: application/json" \
        -d "{\"prompt\": \"$REPLY\"}" \
        "https://jules.googleapis.com/v1alpha/sessions/${SESSION_ID}:sendMessage")

    if [[ "$HTTP" == "200" ]]; then
        echo "✅ Replied — Jules will continue"
    else
        echo "❌ Failed (HTTP $HTTP)"
    fi

else
    echo "Usage:"
    echo "  ./reply-to-jules.sh <session-id> \"Your message\""
    echo "  ./reply-to-jules.sh auto                        (reply to ALL waiting sessions)"
    echo "  ./reply-to-jules.sh auto \"Custom message\"       (reply with custom message)"
fi
