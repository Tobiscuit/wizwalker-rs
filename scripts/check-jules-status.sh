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

# Fetch all sessions (paginate if needed)
RESPONSE=$(curl -s \
    -H "x-goog-api-key: $JULES_API_KEY" \
    "https://jules.googleapis.com/v1alpha/sessions?pageSize=50")

# Parse and display — using real API state names confirmed from live call
echo "$RESPONSE" | python3 -c "
import sys, json
d = json.load(sys.stdin)
sessions = sorted(d.get('sessions', []), key=lambda s: s.get('createTime', ''))

needs_feedback = []
for s in sessions:
    state = s.get('state', 'UNKNOWN')
    title = s.get('title', 'Untitled')
    url = s.get('url', '')

    if state == 'COMPLETED':
        icon = '✅'
    elif state == 'IN_PROGRESS':
        icon = '⏳'
    elif state == 'FAILED':
        icon = '❌'
    elif state == 'AWAITING_USER_FEEDBACK':
        icon = '🙋'
        needs_feedback.append((title, url))
    else:
        icon = '❓'

    print(f'[{icon}] {title} — {state}')

if needs_feedback:
    print()
    print('⚠️  NEEDS YOUR FEEDBACK (Jules is waiting):')
    for title, url in needs_feedback:
        print(f'   {title}')
        print(f'   → {url}')
"

echo ""

# Summary counts using python3 (more reliable than jq for this)
python3 -c "
import sys, json
d = json.load(sys.stdin)
sessions = d.get('sessions', [])
from collections import Counter
counts = Counter(s.get('state', 'UNKNOWN') for s in sessions)
print('═' * 39)
print(f\"  📊 Total:              {len(sessions)}\")
print(f\"  ✅ Completed:          {counts.get('COMPLETED', 0)}\")
print(f\"  ⏳ In Progress:        {counts.get('IN_PROGRESS', 0)}\")
print(f\"  🙋 Awaiting Feedback:  {counts.get('AWAITING_USER_FEEDBACK', 0)}\")
print(f\"  ❌ Failed:             {counts.get('FAILED', 0)}\")
print('═' * 39)
" <<< "$RESPONSE"
