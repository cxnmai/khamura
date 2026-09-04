# Khamura

A Linux desktop camera app built with Rust and GPUI.

## Configuration

Optionally create `~/.config/khamura/config.toml`:

```toml
photo_directory = "~/Pictures/khamura"
theme_color = "#000000"
background_opacity = 0.7
```

All settings are optional; the example shows the defaults. Settings are loaded
at startup, so restart the app after editing. Invalid settings are reported on
stderr and prevent startup.

- `photo_directory`: an absolute path or a path starting with `~/`. Other shell
  expansions (such as `$HOME`) are not supported. Defaults to
  `~/Pictures/khamura`. This configures the destination for future photo saving;
  photo capture is not implemented yet. Loading config does not create it.
- `theme_color`: RGB hex color (`#RRGGBB`), used for the toolbar and preview
  letterbox background.
- `background_opacity`: number from `0.0` (transparent) to `1.0` (opaque), applied
  to the preview letterbox background. The toolbar stays opaque for readability;
  the camera image is unaffected.

The config path is relative to `HOME`; `XDG_CONFIG_HOME` is not used.
No configuration file is required or automatically generated.

## Development

```sh
nix develop
cargo run
cargo test
```
