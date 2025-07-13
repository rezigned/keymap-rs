#!/usr/bin/env bash

set -euxo pipefail

TARGET_BRANCH="${1:-main}"

# An overview of how release-please works:
#
# It only runs against the target remote branch of the repository. Any changes made locally will
# not trigger a release.
#
# By default, only commits start with "feat:", "fix:", or "deps:" will trigger a new release.
release-please release-pr \
  --token "$(gh auth token)" \
  --repo-url rezigned/keymap-rs \
  --config-file .github/prerelease-please-config.json \
  --manifest-file .github/.release-please-manifest.json \
  --target-branch "${TARGET_BRANCH}" \
  --debug \
  --dry-run
