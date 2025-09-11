#!/usr/bin/env bash
# Local Jekyll Development Server
# Serves the website locally without the GitHub Pages baseurl

set -euo pipefail

# Change to the website directory
cd "$(dirname "$0")"

echo "🌐 Starting Jekyll local development server..."
echo "📁 Serving from: $(pwd)"
echo "🔗 Local URL: http://localhost:4000"
echo "📝 Configuration: _config.yml + _config.dev.yml"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Serve with both config files - the dev config overrides production settings
exec bundle exec jekyll serve \
  --config "_config.yml,_config.dev.yml" \
  --livereload \
  --incremental \
  --drafts \
  --future \
  --verbose
