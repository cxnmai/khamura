#!/usr/bin/env bash
# Exercise the shipped executable without a camera, compositor, or user's config.
set -euo pipefail
binary=$(realpath "${1:?usage: smoke-desktop.sh BINARY}")
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
export HOME="$scratch/home"
export XDG_DATA_HOME="$scratch/data"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_RUNTIME_DIR="$scratch/runtime"
export DISPLAY=:65535 WAYLAND_DISPLAY=nonexistent-khamura-ci-display
mkdir -p "$HOME/.config/khamura" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
printf 'invalid = [\n' > "$HOME/.config/khamura/config.toml"

# A path containing spaces also checks desktop Exec quoting.
mkdir "$scratch/bin with spaces"
cp "$binary" "$scratch/bin with spaces/khamura"
binary="$scratch/bin with spaces/khamura"
timeout 15 "$binary" --version | grep -E '^khamura [0-9]+\.[0-9]+\.[0-9]+'
timeout 15 "$binary" --help | grep -F -- '--install-desktop'
timeout 15 "$binary" --install-desktop
launcher="$XDG_DATA_HOME/applications/khamura.desktop"
icon="$XDG_DATA_HOME/icons/hicolor/scalable/apps/khamura.svg"
desktop-file-validate "$launcher"
grep -Fx "Exec=\"$binary\"" "$launcher"
grep -Fx 'Icon=khamura' "$launcher"
test "$(stat -c %a "$launcher")" = 644
test "$(stat -c %a "$icon")" = 644
cmp "$root/assets/khamura.svg" "$icon"
cp "$launcher" "$scratch/first.desktop"
timeout 15 "$binary" --install-desktop
cmp "$scratch/first.desktop" "$launcher"
cmp "$root/assets/khamura.svg" "$icon"
# Installing desktop assets must not parse or modify the camera configuration.
printf 'invalid = [\n' | cmp - "$HOME/.config/khamura/config.toml"
echo 'Headless desktop installation passed.'
