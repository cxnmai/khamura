# Khamura

A Linux desktop camera app built with Rust and GPUI.

## Settings

Click the sliders button on the toolbar to open settings above it. Choose Fit or
Fill for the preview, pick a theme swatch, or drag the background-opacity slider
to adjust the letterboxing. Changes apply immediately while the camera keeps
running. The toolbar and settings panel remain opaque.

Appearance offers two rows of ten small swatches: dark themes above, light themes
below, inspired by the [Helix theme catalogue](https://github.com/helix-editor/helix/tree/master/runtime/themes).
Hover a swatch for its name. These select the background color, not a full editor
theme; a custom hex color can still be set in the config file.

Press Escape, click outside the panel, or click the sliders button again to close
it. Fit and color selections save immediately; opacity saves when the slider is
released (or the panel closes). All three settings are restored on restart.
Save failures appear in the panel with a retry action; changes remain visible
but are not persistent until saving succeeds.

## Configuration

Optionally create `~/.config/khamura/config.toml`:

```toml
photo_directory = "~/Pictures/khamura"
theme_color = "#000000"
background_opacity = 0.7
preview_fit = "contain"
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
- `preview_fit`: `"contain"` (Fit, the default) or `"cover"` (Fill).

The config path is relative to `HOME`; `XDG_CONFIG_HOME` is not used.
No configuration file is required at startup. Changing a setting creates it
and its parent directory if needed. Saves preserve existing comments and
`photo_directory`, and replace the file atomically rather than truncating it.

## Development

```sh
nix develop
cargo run
cargo test
```
