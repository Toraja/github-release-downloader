#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $(basename "$0") <level> [changelog] [repo-url]

Prepare the CHANGELOG for a release by inserting a versioned section and
updating the reference-style comparison links.

Arguments:
  level       Version bump level passed to cargo-release (e.g. patch, minor, major)
  changelog   Path to the changelog file (default: CHANGELOG.md)
  repo-url    Repository base URL used for comparison links
              (default: https://github.com/Toraja/github-release-downloader)

Options:
  -h, --help  Show this help message and exit
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

LEVEL=$1
CHANGELOG="${2:-CHANGELOG.md}"
REPO_URL="${3:-https://github.com/Toraja/github-release-downloader}"

if [[ -z "$LEVEL" ]]; then
  echo "ERROR: <level> is required" >&2
  usage >&2
  exit 1
fi

# Determine next version via cargo-release
VERSION=$(cargo release version "$LEVEL" 2>&1 | grep "^ *Upgrading" | awk '{print $NF}')
if [[ -z "$VERSION" ]]; then
  echo "ERROR: could not determine next version from cargo release" >&2
  exit 1
fi

DATE=$(date +%Y-%m-%d)

echo "Preparing release: v${VERSION} (${DATE})"

# --- Insert version header below ## [Unreleased] ---
# Replace the first occurrence of "## [Unreleased]" with itself followed by
# a blank line and the new version header + blank line.
sed -i "s|^## \[Unreleased\]$|## [Unreleased]\n\n## [${VERSION}] - ${DATE}|" "$CHANGELOG"

# --- Update / add reference-style links in the footnote ---

# Find the previous latest version by looking at the current [Unreleased] link
PREV_VERSION=$(grep -E '^\[Unreleased\]:' "$CHANGELOG" | sed 's|.*compare/v\(.*\)\.\.\.HEAD|\1|')
if [[ -z "$PREV_VERSION" ]]; then
  echo "ERROR: could not determine previous version from [Unreleased] link" >&2
  exit 1
fi

# Update [Unreleased] to compare new version..HEAD
sed -i "s|^\[Unreleased\]:.*|\[Unreleased\]: ${REPO_URL}/compare/v${VERSION}...HEAD|" "$CHANGELOG"

# Insert the new version link directly after the updated [Unreleased] line
NEW_VERSION_LINK="[${VERSION}]: ${REPO_URL}/compare/v${PREV_VERSION}...v${VERSION}"
sed -i "/^\[Unreleased\]:/a ${NEW_VERSION_LINK}" "$CHANGELOG"

echo "Done. Review ${CHANGELOG} before committing."
