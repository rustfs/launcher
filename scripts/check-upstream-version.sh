#!/bin/bash

# Check the upstream rustfs/rustfs version.
# Used to test the version-sync logic locally.

set -e

UPSTREAM_REPO="rustfs/rustfs"
VERSION_API="https://version.rustfs.com/latest.json"

echo "Checking upstream repository version..."
echo "Repository: ${UPSTREAM_REPO}"
echo ""

echo "Fetching the latest upstream version..."
RAW_RESPONSE="$(curl -fsSL "$VERSION_API")"
UPSTREAM_VERSION="$(printf '%s' "$RAW_RESPONSE" | jq -r '.tag // .version // empty')"

if [ -z "$UPSTREAM_VERSION" ] || [ "$UPSTREAM_VERSION" = "null" ]; then
    echo "Failed to fetch the upstream version"
    echo "Raw response:"
    echo "$RAW_RESPONSE"
    echo "Check:"
    echo "  1. Network connectivity"
    echo "  2. Whether latest.json is reachable"
    echo "  3. Whether jq is installed"
    exit 1
fi

echo "Latest upstream version: $UPSTREAM_VERSION"
echo ""

# Read the current repository version.
echo "Checking the current repository version..."
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "Current version: $CURRENT_VERSION"
echo ""

# Compare versions.
echo "Comparing versions..."
UPSTREAM_NORMALIZED="${UPSTREAM_VERSION#v}"
CURRENT_NORMALIZED="${CURRENT_VERSION#v}"

if [ "$UPSTREAM_NORMALIZED" != "$CURRENT_NORMALIZED" ]; then
    echo "New version found."
    echo "  Upstream: $UPSTREAM_VERSION"
    echo "  Current: $CURRENT_VERSION"
    echo ""
    echo "Suggested actions:"
    echo "  1. Create a tag with 'git tag $UPSTREAM_VERSION'"
    echo "  2. Push it with 'git push origin $UPSTREAM_VERSION'"
    echo "  3. Or wait for the sync workflow (daily at 02:00 UTC; it only creates the tag, desktop installers are built manually)"
    exit 0
else
    echo "Already on the latest version"
    echo "No update needed"
    exit 0
fi
