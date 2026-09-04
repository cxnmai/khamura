# Binary releases

Khamura's Cargo metadata points `cargo-binstall` at GitHub release assets.
This configures discovery; it does not build, upload, or publish anything.
The regular installation flow requires a published `khamura` crate with this
metadata and a matching release asset for the user's Linux target:

```sh
cargo binstall khamura
khamura --install-desktop
```

The second command installs the current executable's desktop launcher and icon
for the current user. Run it again after moving the installed executable.

## Archive contract

For version `0.1.0` on `x86_64-unknown-linux-gnu`, use release tag `v0.1.0`
and asset name `khamura-x86_64-unknown-linux-gnu-v0.1.0.tar.gz` containing:

```text
khamura-x86_64-unknown-linux-gnu-v0.1.0/
  khamura                 # Executable, with its executable permission preserved
  LICENSE
```

The SVG is embedded in the executable, so desktop installation does not need
files from the source checkout. Repeat this naming/layout for every tested target.
The URL and binary-path templates follow the
[official cargo-binstall metadata format](https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md).

## Building a release

Build in a conventional Linux environment with the native development packages
listed in the README. A binary built in the Nix development shell is **not a
portable Linux release**: it can reference the Nix dynamic linker and libraries.
Use an appropriate older glibc baseline for the distributions you intend to
support, and test the result on those distributions before advertising support.

For a native x86-64 Linux build, from the repository root:

```sh
version=0.1.0 # Must match Cargo.toml
target=x86_64-unknown-linux-gnu
name="khamura-$target-v$version"
cargo build --locked --release --target "$target"
stage=$(mktemp -d)
mkdir "$stage/$name"
install -m755 "target/$target/release/khamura" "$stage/$name/khamura"
install -m644 LICENSE "$stage/$name/LICENSE"
tar -C "$stage" -czf "$name.tar.gz" "$name"
```

Inspect the binary's dependencies with `ldd`, then test capture, gallery playback,
and desktop installation on a clean target system. Users still need the README's
runtime dependencies: FFmpeg tools, `pactl`, audio services, graphics drivers,
and the native shared libraries linked by the executable. Cargo-binstall does
not install system packages.

## Publishing checklist

1. Update the crate version and review `cargo package --list`. Confirm the icon,
   source files, and existing license are included.
2. Run tests and `cargo publish --dry-run` in the release build environment.
3. Build and test the archives. Publish a GitHub release at the matching `v…`
   tag and upload its target archives. Make the corresponding source available
   and review the existing license's distribution requirements.
4. Publish the crate to crates.io once the assets are available. Publishing is
   permanent; verify the package name and account ownership first.
5. Test `cargo binstall khamura --version 0.1.0 --strategies crate-meta-data`
   on a clean machine, then `khamura --install-desktop`. Restricting strategies
   here verifies the release asset instead of silently testing a source fallback.

Before publishing the crate, uploaded assets can be checked against the local
manifest with `cargo binstall --manifest-path . khamura --strategies crate-meta-data`.
No releases or crates are published automatically by this repository.
