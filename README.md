# Khamura

A Linux desktop camera app built with Rust and GPUI.

## Capture

Click an inactive photo/video icon to select that mode, then click the selected
icon to capture a photo or start/stop recording. **Space** also captures or
starts/stops recording. Photo capture is momentary, not a toggle.

Photos save as **PNG**; videos save as **MP4 (H.264)**, with **AAC** audio when the
microphone is enabled. Both use the output directory, which is created when
needed. Mirror applies to both the live preview and saved photos/videos.

The toolbar changes with the selected mode:
- **Photo:** countdown timer (Off / 3s / 10s), aspect ratio (Native / 4:3 / 16:9 /
  Square), and a rule-of-thirds grid. The selected aspect ratio crops the saved
  photo as well as its preview. Explicit photo ratios are fully visible even if
  preview Fill is selected; Fill remains a preview-only crop for Native/video.
- **Video:** microphone On/Off, camera-supported resolution/frame-rate presets,
  and the same grid. Grid lines are guides only; they never appear in saved media.

A recording timer appears below the top edge of the preview. Capture options and
Mirror are locked during capture/recording. Closing the window waits for video
finalization. Capture/save feedback and error popups report results; there is
no gallery yet. **Escape** cancels a countdown or dismisses settings/an error.

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
released (or the panel closes). Settings are restored on restart.
The **Mirror** toggle flips the camera image horizontally and saves
immediately. Mirroring is on by default and works in both Fit and Fill modes;
saved photos and videos are mirrored too. The toolbar and settings panel are
never mirrored.
Save failures appear in the panel with a retry action; changes remain visible
but are not persistent until saving succeeds.

Camera and microphone dropdowns select capture devices; **System default**
uses the default device. **Refresh devices** updates the list after connecting
hardware. These selections persist across restarts; unavailable saved devices
remain visible in the list.

The path controls show the current locations and open native file dialogs:
- **Config path:** click the boxed path to choose a new settings-file location. Existing files
  are not overwritten. The app remembers the new location across restarts.
- **Output path:** click the boxed path to choose the photo/video output directory.
  This updates `photo_directory`.

Cancelling a dialog leaves the paths unchanged. Path-change failures appear
inline in the settings panel.

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

## Development

Video recording requires `ffmpeg` with H.264/AAC encoding and PulseAudio input;
microphone discovery requires `pactl`. Audio works with PulseAudio or PipeWire's
PulseAudio compatibility service. The Nix development environment supplies
`ffmpeg-full` and `pulseaudio`; run inside it, or install equivalent binaries
on your system and make them available on `PATH`.


```sh
nix develop
cargo run
cargo test
```
