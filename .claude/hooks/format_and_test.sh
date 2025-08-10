#!/usr/bin/env bash
set -euo pipefail

# Run Rust-specific formatting and linting
if [ -f Cargo.toml ]; then
    echo "Running Rust formatting and tests..."
    cargo fmt
    cargo clippy -- -D warnings
    cargo nextest run --nocapture
elif [ -f .git/hooks/pre-commit ]; then
    ./.git/hooks/pre-commit
fi
