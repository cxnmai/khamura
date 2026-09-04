# Publishing releases

`.github/workflows/release.yml` publishes Linux x86-64 binaries when a `vVERSION`
tag is pushed. It does **not** run on pull requests or ordinary branch pushes.
The tag must match `Cargo.toml`, for example `v0.1.0`.

The workflow builds on Ubuntu 22.04, checks headless desktop installation, creates
the archive and checksum, and attaches them to a GitHub release. It then publishes
the crate if the `CARGO_REGISTRY_TOKEN` Actions secret is configured. Nothing
requires a camera or microphone on the build runner.

## One-time setup

1. Sign in to crates.io and create a publishing API token authorized for `khamura`.
   For the first publication, it must permit creating the new crate.
2. Add it as the repository's **Actions secret** `CARGO_REGISTRY_TOKEN`:
   <https://github.com/cxnmai/khamura/settings/secrets/actions>.
   Never commit the token or paste it into an issue or chat.
3. Ensure GitHub Actions is enabled and the version/name/license are correct.
   Crates.io versions are immutable; review before publishing.

Without the secret, the GitHub release still completes and the workflow prints a
warning. Add the secret and re-run the release workflow **from the same tag** to
publish the crate. Already-published crate versions are skipped on retries.

## Release

```sh
# Update Cargo.toml and Cargo.lock when changing the version, then commit.
git push origin main
git tag -a v0.1.0 -m 'Khamura 0.1.0'
git push origin v0.1.0
```

Monitor the **Publish release** workflow in GitHub Actions. It can also be
manually dispatched from a version tag for retries. GitHub release uploads are
idempotent; a retry replaces that tag's existing archive/checksum.

## Archive and compatibility

For `v0.1.0`, cargo-binstall metadata expects:

```text
khamura-x86_64-unknown-linux-gnu-v0.1.0.tar.gz
└── khamura-x86_64-unknown-linux-gnu-v0.1.0/
    ├── khamura
    └── LICENSE
```

The icon is embedded, so no source checkout is needed for desktop installation.
`scripts/release/package.sh` rejects binaries linked to Nix paths or missing
shared libraries. Release binaries require glibc 2.35+ and the runtime dependencies
listed in the README. Other architectures are not currently built by this workflow.
NixOS users need an appropriate compatibility environment or a Nix source build.

## Verify installation

After crates.io publication:

```sh
cargo binstall khamura --version 0.1.0 --strategies crate-meta-data
khamura --version
khamura --install-desktop
```

Before the first crate publication, the release archive can be installed using
this checkout's metadata:

```sh
cargo binstall --manifest-path . khamura --strategies crate-meta-data
```

See the [official cargo-binstall metadata documentation](https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md)
for archive discovery rules. No OS dependencies are installed by cargo-binstall.
