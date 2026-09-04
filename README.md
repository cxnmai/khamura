# Khamura

A minimal Linux camera app built with Rust and GPUI. Capture PNG photos and
H.264 MP4 videos with optional AAC audio, then browse them in the built-in gallery.
Includes camera and microphone selection, capture timers, mirroring, and themes.

## Dependencies

- A V4L2-compatible camera.
- `ffmpeg` with H.264/AAC encoding and PulseAudio input support.
- `ffprobe` and `ffplay` for gallery video playback.
- `pactl` for microphone discovery; PulseAudio or PipeWire's PulseAudio service
  for audio.
- A Vulkan-capable graphics driver. VAAPI encoding is optional; recording falls
  back to software when hardware encoding is unavailable.

## Installation

On x86-64 Linux with glibc 2.35+, install using
[cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall khamura
khamura --install-desktop
```

Desktop installation embeds the icon and registers the current binary in your
application menu, without opening the camera. It uses `$XDG_DATA_HOME` (default
`~/.local/share`); re-run it if you move the binary. System dependencies still
need to be installed separately. See [release packaging](docs/releases.md).

### Nix / NixOS

On x86-64 NixOS, install the prebuilt release with Nix-managed runtime tools,
the desktop launcher, and icon:

```sh
nix profile add github:cxnmai/khamura
```

For a declarative NixOS or Home Manager setup, add `github:cxnmai/khamura` as a
flake input and include `inputs.khamura.packages.${pkgs.stdenv.hostPlatform.system}.default`
in `environment.systemPackages` or `home.packages`. No `--install-desktop` step
is needed for Nix installs. Nix downloads the release binary and patches its
library paths; it does not compile Rust.

## Configuration

Settings save automatically to `~/.config/khamura/config.toml`. No config file is
required to start; missing settings use defaults. Restart after editing manually.

```toml
photo_directory = "~/Pictures/khamura"
theme_color = "#000000"
background_opacity = 0.7
preview_fit = "contain"
mirror = true

[capture]
mode = "photo"
timer_seconds = 0
aspect = "native"
grid = false
microphone_on = true

# Optional: omit to use default devices and native camera quality.
# camera_device = "/dev/video0"
# microphone_device = "alsa_input.example"
# quality = { width = 1920, height = 1080, fps = 30 }
```

| Setting | Values |
| --- | --- |
| `photo_directory` | Photo/video output folder; absolute path or `~/…`. |
| `theme_color` | Background color as `#RRGGBB`. |
| `background_opacity` | `0.0`–`1.0`; preview background and gallery transparency. |
| `preview_fit` | `"contain"` (Fit) or `"cover"` (Fill). |
| `mirror` | Mirror the preview and saved captures. |
| `capture.mode` | `"photo"` or `"video"`. |
| `capture.timer_seconds` | `0`, `3`, or `10`. |
| `capture.aspect` | `"native"`, `"four_three"`, `"sixteen_nine"`, or `"square"`. |
| `capture.grid` | Show composition guides, without saving them in captures. |
| `capture.microphone_on` | Include audio in recordings. |
| `capture.camera_device` | Camera device path. |
| `capture.microphone_device` | PulseAudio source name, not its display label. |
| `capture.quality` | Camera-supported `width`, `height`, and `fps`. |

The settings menu can change the config and output paths. Relocated config paths
are remembered in `~/.config/khamura/config-path`; the default location uses
`HOME`, not `XDG_CONFIG_HOME`.

Executable locations can be overridden with `KHAMURA_FFMPEG`, `KHAMURA_FFPROBE`,
`KHAMURA_FFPLAY`, and `KHAMURA_PACTL`. Set `KHAMURA_VIDEO_ENCODER=software` to
force software recording when troubleshooting graphics drivers.

## Development

Install Rust and Cargo. The Nix development shell supplies the native build
libraries and runtime tools:

```sh
nix develop
cargo run             # Debug build and run
cargo test            # Run tests
cargo build --release # Optimized binary: target/release/khamura
target/release/khamura --install-desktop # Optional local desktop integration
```

Use `cargo run --release` when evaluating camera or video performance.
Nix builds retain runtime tool paths so the binary can also launch outside the
shell. Without Nix, install the dependencies above plus `pkg-config`, libclang,
Linux headers, Fontconfig, FreeType, libxkbcommon, Vulkan, Wayland, and XCB
libraries and development headers. Cargo manages the Rust dependencies.
