# Memory System CLI Commands

This document provides command-line examples for managing the agent memory system using standard tools.

## Quick Search Commands

### Search Memory Content
```bash
# Search for specific terms across all memories
rg "nutype" .claude/memories/ -A 2 -B 2

# Search in specific scope
rg "validation" .claude/memories/shared/ --type json

# Search by category
rg '"category": "decisions"' .claude/memories/ -A 10

# Case-insensitive search
rg -i "github actions" .claude/memories/
```

### Search by Tags
```bash
# Find memories with specific tags
rg '"tags":.*"tdd"' .claude/memories/ -A 5

# Multiple tag search
rg '"tags":.*("testing"|"validation")' .claude/memories/

# Complex tag combinations
rg '"tags":.*"types".*"nutype"' .claude/memories/
```

### Search by Agent
```bash
# All memories from specific agent
rg '"agent": "implementer"' .claude/memories/

# Implementer's private memories only
find .claude/memories/private/implementer/ -name "*.json" -exec cat {} \;

# Agent's recent memories (last 5 files by date)
find .claude/memories/private/implementer/ -name "*.json" -exec ls -t {} \; | head -5
```

## Index Queries

### Count Memories by Category
```bash
# Shared memory counts
cat .claude/memories/shared/index.json | jq '.categories | to_entries[] | "\(.key): \(.value | length)"'

# Private memory counts for all agents
cat .claude/memories/private/index.json | jq '.agents | to_entries[] | "\(.key): \(.value.total_memories)"'
```

### Popular Tags
```bash
# Most used shared tags
cat .claude/memories/shared/index.json | jq '.tags | to_entries | sort_by(.value) | reverse | .[0:10] | .[] | "\(.key): \(.value)"'

# Agent-specific tag usage
cat .claude/memories/private/index.json | jq '.agents.implementer.tags | to_entries | sort_by(.value) | reverse | .[] | "\(.key): \(.value)"'
```

### Recent Activity
```bash
# Recent shared memories
cat .claude/memories/shared/index.json | jq -r '.recent_memories[]'

# Get details of recent memories
cat .claude/memories/shared/index.json | jq -r '.recent_memories[]' | head -3 | while read id; do
  find .claude/memories/shared/ -name "*${id#*-}*.json" -exec jq -r '"\(.title) (\(.metadata.created_at))"' {} \;
done
```

## File Management Commands

### List Memory Files
```bash
# All memory files by date
find .claude/memories/ -name "*.json" -not -name "index.json" -exec ls -lt {} \; | head -20

# Memory files by agent
find .claude/memories/private/implementer/ -name "*.json" -exec basename {} \; | sort

# Memory files by category
find .claude/memories/shared/decisions/ -name "*.json" -exec basename {} \;
```

### Memory File Statistics
```bash
# Total memory count
find .claude/memories/ -name "*.json" -not -name "index.json" | wc -l

# Memory distribution by scope
echo "Shared: $(find .claude/memories/shared/ -name "*.json" -not -name "index.json" | wc -l)"
echo "Private: $(find .claude/memories/private/ -name "*.json" -not -name "index.json" | wc -l)"

# Size statistics
find .claude/memories/ -name "*.json" -not -name "index.json" -exec wc -c {} \; | awk '{sum+=$1} END {print "Total memory size:", sum, "bytes"}'
```

## Content Analysis Commands

### Extract Memory Titles
```bash
# All memory titles
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.title' {} \;

# Titles by priority
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r 'select(.metadata.priority == "high") | .title' {} \;

# Recent high-priority memories
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r 'select(.metadata.priority == "high") | "\(.metadata.created_at): \(.title)"' {} \; | sort -r | head -10
```

### Extract Key Information
```bash
# All unique tags
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.tags[]' {} \; | sort -u

# Story contexts
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.metadata.story_context' {} \; | sort -u

# Agents that have created memories
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.agent' {} \; | sort -u
```

## Validation Commands

### Check Memory Integrity
```bash
# Validate JSON syntax
find .claude/memories/ -name "*.json" -exec sh -c 'jq empty "$1" 2>/dev/null || echo "Invalid JSON: $1"' _ {} \;

# Check required fields
find .claude/memories/ -name "*.json" -not -name "index.json" -exec sh -c 'jq -e ".id and .scope and .agent and .category and .title and .content" "$1" >/dev/null || echo "Missing required fields: $1"' _ {} \;

# Verify scope consistency
find .claude/memories/shared/ -name "*.json" -not -name "index.json" -exec sh -c 'jq -e ".scope == \"shared\"" "$1" >/dev/null || echo "Scope mismatch in shared: $1"' _ {} \;
find .claude/memories/private/ -name "*.json" -not -name "index.json" -exec sh -c 'jq -e ".scope == \"private\"" "$1" >/dev/null || echo "Scope mismatch in private: $1"' _ {} \;
```

### Check Index Consistency
```bash
# Verify shared index matches files
SHARED_FILES=$(find .claude/memories/shared/ -name "*.json" -not -name "index.json" | wc -l)
INDEX_COUNT=$(cat .claude/memories/shared/index.json | jq '.total_memories')
echo "Shared files: $SHARED_FILES, Index count: $INDEX_COUNT"

# Check private index totals
cat .claude/memories/private/index.json | jq -r '.agents | to_entries[] | "\(.key): \(.value.total_memories)"' | while read agent count; do
  actual=$(find ".claude/memories/private/$agent/" -name "*.json" 2>/dev/null | wc -l)
  echo "$agent - Index: $count, Actual: $actual"
done
```

## Cleanup Commands

### Remove Old Low-Priority Memories
```bash
# Find old low-priority memories (older than 90 days)
find .claude/memories/ -name "*.json" -not -name "index.json" -mtime +90 -exec jq -r 'select(.metadata.priority == "low") | .id' {} \;

# List expired memories
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r 'select(.metadata.expires_after != null) | select((.metadata.expires_after | strptime("%Y-%m-%dT%H:%M:%SZ") | mktime) < now) | "\(.id) (expired: \(.metadata.expires_after))"' {} \;
```

### Duplicate Detection
```bash
# Find potential duplicates by title similarity
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.title' {} \; | sort | uniq -c | sort -nr | head -10

# Find memories with identical content
find .claude/memories/ -name "*.json" -not -name "index.json" -exec jq -r '.content' {} \; | sort | uniq -c | awk '$1 > 1' | head -5
```

## Export and Backup Commands

### Export Memory Data
```bash
# Export all memories to single JSON
echo '{"memories": [' > memory-export.json
find .claude/memories/ -name "*.json" -not -name "index.json" -exec cat {} \; | jq -s '.' | jq '.[0:-1] | .[]' | paste -sd ',' >> memory-export.json
echo ']}' >> memory-export.json

# Export by agent
mkdir -p memory-exports
for agent in researcher planner implementer type-architect test-hardener expert pr-manager; do
  find ".claude/memories/private/$agent/" -name "*.json" -exec cat {} \; | jq -s '.' > "memory-exports/$agent-memories.json" 2>/dev/null
done

# Export shared memories by category
for cat in decisions learnings context general; do
  find ".claude/memories/shared/$cat/" -name "*.json" -exec cat {} \; | jq -s '.' > "memory-exports/shared-$cat.json" 2>/dev/null
done
```

### Create Memory Backup
```bash
# Full backup
tar -czf "memory-backup-$(date +%Y%m%d).tar.gz" .claude/memories/

# Incremental backup (files changed in last 7 days)
find .claude/memories/ -mtime -7 -type f | tar -czf "memory-incremental-$(date +%Y%m%d).tar.gz" -T -
```

These commands provide comprehensive management capabilities for the memory system using standard command-line tools.
