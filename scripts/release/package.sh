#!/usr/bin/env bash
set -euo pipefail

RUNNER_TEMP=${RUNNER_TEMP:-${TMPDIR:-/tmp}}

target=${1:?usage: package.sh TARGET VERSION}
version=${2:?usage: package.sh TARGET VERSION}
[[ "$target" == x86_64-unknown-linux-gnu ]] || { echo 'Unsupported release target' >&2; exit 1; }
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] || { echo 'Invalid version' >&2; exit 1; }
binary="target/$target/release/khamura"
[[ -x "$binary" ]]

# Do not accidentally distribute Nix-linked binaries or unresolved shared libraries.
links=$(ldd "$binary")
if grep -Eq 'not found|/nix/store/' <<<"$links"; then
  printf 'Nonportable release dependencies:\n%s\n' "$links" >&2
  exit 1
fi
readelf -l "$binary" > "$RUNNER_TEMP/khamura-elf.txt"
if grep -q '/nix/store/' "$RUNNER_TEMP/khamura-elf.txt"; then
  echo 'Release executable uses a Nix dynamic linker' >&2
  exit 1
fi

name="khamura-$target-v$version"
stage=$(mktemp -d)
trap 'find "$stage" -type f -delete; find "$stage" -depth -type d -empty -delete' EXIT
mkdir -p "$stage/$name" dist
install -m755 "$binary" "$stage/$name/khamura"
install -m644 LICENSE "$stage/$name/LICENSE"
epoch=$(git log -1 --format=%ct)
tar --sort=name --mtime="@$epoch" --owner=0 --group=0 --numeric-owner \
  -C "$stage" -czf "dist/$name.tar.gz" "$name"
(cd dist && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
# Verify the archive itself, not just the original build output.
tar -tzf "dist/$name.tar.gz"
