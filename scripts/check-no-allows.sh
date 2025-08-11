#!/bin/bash
# Pre-commit hook to prevent new clippy allow attributes
# Part of Story 053 code quality enforcement

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Checking for new clippy allow attributes..."

# Check for any new allow attributes in staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=AM | grep '\.rs$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "✅ No Rust files in staging area"
    exit 0
fi

# Look for clippy allows in staged changes
FOUND_ALLOWS=false

for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        # Check for both function-level and crate-level allows (exact patterns only)
        FUNCTION_ALLOWS=$(git diff --cached "$file" | grep "^+" | grep -E "^\+[[:space:]]*#\[allow\(clippy::" || true)
        CRATE_ALLOWS=$(git diff --cached "$file" | grep "^+" | grep -E "^\+[[:space:]]*#!\[allow\(clippy::" || true)

        if [ -n "$FUNCTION_ALLOWS" ] || [ -n "$CRATE_ALLOWS" ]; then
            if [ "$FOUND_ALLOWS" = false ]; then
                echo -e "${RED}❌ POLICY VIOLATION: New clippy allow attributes detected${NC}"
                echo ""
                echo -e "${YELLOW}📋 Code Quality Policy:${NC}"
                echo "  • Clippy allow attributes are prohibited without explicit team approval"
                echo "  • Fix the underlying issue instead of suppressing warnings"
                echo "  • For rare exceptions, create a GitHub issue and get team approval"
                echo ""
                echo -e "${YELLOW}🚫 Found new allow attributes in:${NC}"
                FOUND_ALLOWS=true
            fi

            echo -e "${RED}📄 $file:${NC}"
            if [ -n "$FUNCTION_ALLOWS" ]; then
                echo "$FUNCTION_ALLOWS" | sed 's/^+/  /'
            fi
            if [ -n "$CRATE_ALLOWS" ]; then
                echo "$CRATE_ALLOWS" | sed 's/^+/  /'
            fi
            echo ""
        fi
    fi
done

if [ "$FOUND_ALLOWS" = true ]; then
    echo -e "${YELLOW}💡 Emergency Override:${NC}"
    echo "  If this is a genuine emergency, use: git commit --no-verify"
    echo "  ⚠️  You MUST notify the team immediately if you use --no-verify"
    echo ""
    exit 1
fi

echo "✅ No new clippy allow attributes found"
exit 0
