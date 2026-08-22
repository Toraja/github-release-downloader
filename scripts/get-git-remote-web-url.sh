#!/usr/bin/env bash
set -euo pipefail

# Get the current origin URL
REMOTE_URL=$(git remote get-url origin)

if [[ "$REMOTE_URL" == https://* ]]; then
  echo "${REMOTE_URL%.git}"
  exit 0
elif [[ "$REMOTE_URL" == git@* ]]; then
  # SSH format: git@github.com:user/repo.git
  # Convert to HTTPS: https://github.com/user/repo

  # Remove the 'git@' prefix
  tmp="${REMOTE_URL#git@}"

  # Split host and path
  host="${tmp%%:*}"
  path="${tmp#*:}"

  echo "https://${host}/${path%.git}"
  exit 0
else
  echo "Unknown remote URL protocol: $REMOTE_URL"
  exit 1
fi
