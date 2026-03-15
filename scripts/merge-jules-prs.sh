#!/usr/bin/env bash
# merge-jules-prs.sh — Merge all open Jules PRs in dependency order
#
# Usage: ./merge-jules-prs.sh [--dry-run]
#
# This script merges PRs in the right order so dependencies resolve correctly.
# Run with --dry-run first to see what it would do.

set -euo pipefail

REPO="Tobiscuit/wizwalker-rs"
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "🔍 DRY RUN — no PRs will be merged"
fi

echo "📦 Fetching open PRs for $REPO..."
echo ""

# Get all open PRs as JSON
PRS=$(gh pr list --repo "$REPO" --state open --json number,title,headRefName --limit 50)

if [[ "$PRS" == "[]" ]]; then
    echo "✅ No open PRs found."
    exit 0
fi

echo "Found PRs:"
echo "$PRS" | jq -r '.[] | "  #\(.number): \(.title)"'
echo ""

# Merge order: foundation first, then things that depend on them
# Phase 1 tasks are independent, so order doesn't matter much,
# but we do skeleton first for good measure.
MERGE_ORDER=(
    "Task 1"    # Skeleton + Errors + Constants — no dependencies
    "Task 2"    # Memory Reader — no dependencies
    "Task 4"    # Base Memory Object — used by Tasks 5-8
    "Task 3"    # Hook System — uses MemoryReader
    "Task 11"   # File Readers — no dependencies
    "Task 12"   # Combat + Utils — no dependencies
    "Task 10"   # Mouse + Hotkey — no dependencies
    "Task 9"    # Client — uses hooks + memory objects
    "Task 5"    # Memory Objects Batch 1
    "Task 6"    # Memory Objects Batch 2
    "Task 7"    # Memory Objects Batch 3
    "Task 8"    # Memory Objects Batch 4
    # Phase 2
    "Task 13"   # GUI
    "Task 14"   # Config + Hotkeys
    "Task 17"   # DeimosLang
    "Task 15"   # Combat + Questing
    "Task 16"   # Navigation + Misc
    "Task 18"   # Main integration
    # Phase 3
    "Task 19"   # Audit
)

merged=0
failed=0

for task_prefix in "${MERGE_ORDER[@]}"; do
    # Find PR matching this task
    PR_NUM=$(echo "$PRS" | jq -r ".[] | select(.title | startswith(\"$task_prefix\")) | .number" | head -1)
    
    if [[ -z "$PR_NUM" || "$PR_NUM" == "null" ]]; then
        continue
    fi

    PR_TITLE=$(echo "$PRS" | jq -r ".[] | select(.number == $PR_NUM) | .title")
    
    echo "🔀 Merging PR #$PR_NUM: $PR_TITLE"
    
    if $DRY_RUN; then
        echo "   (dry run — skipping)"
    else
        if gh pr merge "$PR_NUM" --repo "$REPO" --squash --auto 2>/dev/null; then
            echo "   ✅ Merged"
            ((merged++))
        else
            echo "   ⚠️  Could not merge (may have conflicts or failing checks)"
            ((failed++))
        fi
    fi
done

echo ""
echo "═══════════════════════════════════════"
echo "  ✅ Merged: $merged"
echo "  ⚠️  Failed: $failed"
echo "═══════════════════════════════════════"
