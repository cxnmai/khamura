A minimal Linux camera app with PNG photos, H.264 MP4 recording, inline gallery
playback, themes, and a built-in desktop launcher installer.

## Linux x86-64

The archive targets `x86_64-unknown-linux-gnu`, built on Ubuntu 22.04
(glibc 2.35 or newer). It contains the executable and license; the desktop icon
is embedded in the executable. SHA-256 checksums are provided alongside it.

Install with cargo-binstall:

```sh
cargo binstall khamura
khamura --install-desktop
```

Alternatively, extract the archive to a permanent location and run
`./khamura --install-desktop` there. The launcher points to that executable.

Install runtime dependencies separately: FFmpeg (`ffmpeg`, `ffprobe`, `ffplay`),
`pactl`, PulseAudio or PipeWire's PulseAudio service, and Vulkan/Wayland/X11,
Fontconfig, FreeType, and xkbcommon runtime libraries. A V4L2 camera is required.
Hardware encoding is optional; software encoding is available as a fallback.

This is not a standalone AppImage or Nix package. NixOS requires a compatible
loader/library environment (such as a configured nix-ld setup), or a source build
using the project's Nix development shell.
