#!/usr/bin/env bash

for file in /workspaces/caxton/website/_adr/*.md; do
  filename=$(basename "$file" .md)

  # Check if front matter already exists
  if head -1 "$file" | grep -q "^---"; then
    echo "Front matter already exists in $(basename "$file")"
    continue
  fi

  # Extract ADR number and title from first line
  title_line=$(head -1 "$file")
  adr_num=$(echo "$title_line" | grep -oE 'ADR-[0-9]+' | sed 's/ADR-//')
  title=$(echo "$title_line" | sed 's/^# //' | sed 's/^ADR-[0-9]*: //')

  # Extract status from the file
  status=$(grep -A1 "^## Status" "$file" | tail -1 | tr '[:upper:]' '[:lower:]')

  # Create temp file with front matter
  {
    echo "---"
    echo "layout: adr"
    echo "title: \"ADR-${adr_num}: ${title}\""
    echo "status: ${status}"
    echo "date: 2025-08-08"
    echo "---"
    echo ""
    cat "$file"
  } > "${file}.tmp"

  mv "${file}.tmp" "$file"
  echo "Added front matter to $(basename "$file")"
done
