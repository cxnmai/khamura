#!/usr/bin/env bash
set -euo pipefail
version=${1:?usage: publish-crate.sh VERSION}
if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo '::warning::GitHub binaries are published, but crates.io needs the CARGO_REGISTRY_TOKEN repository secret.'
  echo 'Add CARGO_REGISTRY_TOKEN and re-run this release to enable cargo binstall khamura.' >> "$GITHUB_STEP_SUMMARY"
  exit 0
fi

# Re-running a completed tag must not attempt an immutable version overwrite.
status=$(curl --silent --show-error --location --output "$RUNNER_TEMP/crate-version.json" \
  --write-out '%{http_code}' --user-agent khamura-release \
  "https://crates.io/api/v1/crates/khamura/$version")
case "$status" in
  200) echo "khamura $version is already published on crates.io." ;;
  404) cargo publish --locked --no-verify ;;
  *) echo "Could not check crates.io version (HTTP $status)" >&2; exit 1 ;;
esac
