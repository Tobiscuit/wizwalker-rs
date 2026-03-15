#!/usr/bin/env bash
# check-jules-status.sh — Check the status of wizwalker-rs Jules sessions
#
# Usage: ./check-jules-status.sh

set -euo pipefail

if [[ -z "${JULES_API_KEY:-}" ]]; then
    echo "❌ JULES_API_KEY not set. Export it first:"
    echo "   export JULES_API_KEY=your-key-here"
    exit 1
fi

echo "🧙 Checking wizwalker-rs Jules session status..."
echo ""

RESPONSE=$(curl -s \
    -H "x-goog-api-key: $JULES_API_KEY" \
    "https://jules.googleapis.com/v1alpha/sessions?pageSize=50")

python3 -c "
import sys, json
d = json.load(sys.stdin)
all_sessions = sorted(d.get('sessions', []), key=lambda s: s.get('createTime', ''))

# Filter to only wizwalker-rs tasks
sessions = [s for s in all_sessions if
    s.get('title', '').startswith('Task') or
    'wizwalker-rs' in json.dumps(s.get('sourceContext', {}))
]

if not sessions:
    print('No wizwalker-rs sessions found yet.')
    sys.exit(0)

from collections import Counter
needs_feedback = []

for s in sessions:
    state = s.get('state', 'UNKNOWN')
    title = s.get('title', 'Untitled')
    url   = s.get('url', '')
    icons = {'COMPLETED': '✅', 'IN_PROGRESS': '⏳', 'FAILED': '❌', 'AWAITING_USER_FEEDBACK': '🙋'}
    icon  = icons.get(state, '❓')
    if state == 'AWAITING_USER_FEEDBACK':
        needs_feedback.append((title, url))
    print(f'[{icon}] {title} — {state}')

if needs_feedback:
    print()
    print('⚠️  NEEDS YOUR FEEDBACK (Jules is waiting):')
    for title, url in needs_feedback:
        print(f'   {title}')
        print(f'   → {url}')

counts = Counter(s.get('state', 'UNKNOWN') for s in sessions)
print()
print('═' * 39)
print(f'  📊 Total:              {len(sessions)}')
print(f'  ✅ Completed:          {counts.get(\"COMPLETED\", 0)}')
print(f'  ⏳ In Progress:        {counts.get(\"IN_PROGRESS\", 0)}')
print(f'  🙋 Awaiting Feedback:  {counts.get(\"AWAITING_USER_FEEDBACK\", 0)}')
print(f'  ❌ Failed:             {counts.get(\"FAILED\", 0)}')
print('═' * 39)
" <<< "$RESPONSE"
