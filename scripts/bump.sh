#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <patch|minor|major>"
  exit 1
}

[[ $# -ne 1 ]] && usage

KIND="$1"
[[ "$KIND" != "patch" && "$KIND" != "minor" && "$KIND" != "major" ]] && usage

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Read current version from package.json
CURRENT=$(sed -n 's/.*"version": "\([^"]*\)".*/\1/p' "$ROOT/package.json" | head -1)
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$KIND" in
  major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
  minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
  patch) PATCH=$((PATCH + 1)) ;;
esac

NEW="${MAJOR}.${MINOR}.${PATCH}"
TAG="v${NEW}"

echo "Bumping version: ${CURRENT} → ${NEW}"

# Update package.json
sed -i '' "s/\"version\": \"${CURRENT}\"/\"version\": \"${NEW}\"/" "$ROOT/package.json"

# Update src-tauri/tauri.conf.json
sed -i '' "s/\"version\": \"${CURRENT}\"/\"version\": \"${NEW}\"/" "$ROOT/src-tauri/tauri.conf.json"

# Update src-tauri/Cargo.toml (only the package version line)
sed -i '' "s/^version = \"${CURRENT}\"/version = \"${NEW}\"/" "$ROOT/src-tauri/Cargo.toml"

# Commit and tag
git -C "$ROOT" add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git -C "$ROOT" commit -m "release: ${TAG}"
git -C "$ROOT" tag "$TAG"

echo ""
echo "Done! Created commit and tag ${TAG}"
echo "Next step: git push origin main --tags"
