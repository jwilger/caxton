#!/usr/bin/env bash
# Minimal Jekyll Development Server
# Fallback if the main script has plugin issues

set -euo pipefail

cd "$(dirname "$0")"

echo "🌐 Starting minimal Jekyll server..."
echo "🔗 Local URL: http://localhost:4000"
echo ""

# Minimal Jekyll serve with just the dev config override
exec bundle exec jekyll serve \
  --config "_config.yml,_config.dev.yml" \
  --host 127.0.0.1 \
  --port 4000
