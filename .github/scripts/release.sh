#!/usr/bin/env bash

set -euxo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_BRANCH="${1:-test-release-please}"

# Note that release-please will run against the remote branch, tags and releases on Github.
# It won't run against the local branch.
#
# By default, release-please will run against the `main` branch.
# We can change the target branch via `--target-branch` option.
release-please release-pr \
  --token "$(gh auth token)" \
  --repo-url rezigned/keymap-rs \
  --config-file .github/prerelease-please-config.json \
  --manifest-file .github/.release-please-manifest.json \
  --target-branch "${TARGET_BRANCH}" \
  --debug \
  --dry-run
