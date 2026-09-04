# Khamura

A Linux desktop camera app built with Rust and GPUI.

## Configuration

Optionally create `~/.config/khamura/config.toml`:

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
# Omit these to use the default devices and native camera quality:
# camera_device = "/dev/video0"
# microphone_device = "alsa_input.example"
# quality = { width = 1920, height = 1080, fps = 30 }
```

All settings are optional; the example shows the defaults. Settings are loaded
at startup, so restart the app after editing. Invalid settings are reported on
stderr and prevent startup.

- `photo_directory`: an absolute path or a path starting with `~/`. Other shell
  expansions (such as `$HOME`) are not supported. Defaults to
  `~/Pictures/khamura`. Both photos and videos save here. Loading config does not
  create it; saving media does.
- `theme_color`: RGB hex color (`#RRGGBB`), used for the toolbar and preview
  letterbox background.
- `background_opacity`: number from `0.0` (transparent) to `1.0` (opaque), applied
  to the preview letterbox background. The toolbar stays opaque for readability;
  the camera image is unaffected.
- `preview_fit`: `"contain"` (Fit, the default) or `"cover"` (Fill).
- `mirror`: boolean, defaults to `true`. Set to `false` for an unmirrored live
  preview and saved media, or change it immediately with the settings toggle.

The optional `[capture]` table persists toolbar and device preferences:
- `mode`: `"photo"` or `"video"`.
- `timer_seconds`: `0`, `3`, or `10`.
- `aspect`: `"native"`, `"four_three"`, `"sixteen_nine"`, or `"square"`.
- `grid`: show composition guides (boolean).
- `microphone_on`: include microphone audio in video (boolean, default `true`).
- `camera_device`: camera device path; omit for the default camera.
- `microphone_device`: PulseAudio source name, not its display label; omit for
  the default source. The settings dropdown writes the correct name.
- `quality`: a table with positive `width`, `height`, and `fps` values. Omit for
  native quality; use the toolbar to select a format supported by the camera.

The default config path is relative to `HOME`; `XDG_CONFIG_HOME` is not used.
After relocating the config, `~/.config/khamura/config-path` records its absolute
location so the app can find it on restart. Keep that locator file in place;
removing it makes the app use the default config location again.
No configuration file is required at startup. Changing a setting creates it
and its parent directory if needed. Saves preserve existing comments and
`photo_directory`, and replace the file atomically rather than truncating it.
