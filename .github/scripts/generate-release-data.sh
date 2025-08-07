#!/bin/bash
# Generate release data YAML file for Jekyll

VERSION="$1"
TAG_NAME="$2"
RELEASE_NAME="$3"
RELEASE_DATE="$4"
RELEASE_STATUS="$5"
RELEASE_PHASE="$6"
RELEASE_URL="$7"
RELEASE_BODY="$8"

cat > website/_data/release.yml << EOF
version: "${VERSION}"
tag: "${TAG_NAME}"
name: "${RELEASE_NAME}"
date: "${RELEASE_DATE}"
status: "${RELEASE_STATUS}"
phase: "${RELEASE_PHASE}"
url: "${RELEASE_URL}"
description: |
  ${RELEASE_BODY}
EOF
