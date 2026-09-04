use super::*;

#[test]
fn preferences_roundtrip_and_preserve_manual_settings() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".config/khamura/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# My camera\nphoto_directory = '~/Photos' # Keep this\ntheme_color = '#000000' # Toolbar\n",
    )
    .unwrap();
    let config = Config::parse(&fs::read_to_string(&path).unwrap(), home.path()).unwrap();
    config
        .save_preferences(CameraFit::Cover, Rgb::new(46, 52, 64), 0.35, false)
        .unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("# My camera"));
    assert!(text.contains("photo_directory = '~/Photos' # Keep this"));
    assert!(text.contains("# Toolbar"));
    let reloaded = Config::parse(&text, home.path()).unwrap();
    assert_eq!(reloaded.preview_fit, CameraFit::Cover);
    assert_eq!(reloaded.theme_color, Rgb::new(46, 52, 64));
    assert_eq!(reloaded.background_opacity, 0.35);
    assert!(!reloaded.mirror);
    assert_eq!(reloaded.photo_directory, home.path().join("Photos"));
    // Simulate an external edit after startup.
    fs::write(&path, text.replace("~/Photos", "~/NewPhotos")).unwrap();
    config
        .save_preferences(CameraFit::Contain, Rgb::new(255, 255, 255), 1.0, true)
        .unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("~/NewPhotos"));
    assert!(Config::parse(&saved, home.path()).unwrap().mirror);
}

#[test]
fn first_save_creates_directory_and_invalid_updates_leave_file_untouched() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".config/khamura/config.toml");
    let config = Config::parse("", home.path()).unwrap();
    assert_eq!(config.preview_fit, CameraFit::Contain);
    config
        .save_preferences(CameraFit::Cover, Rgb::new(0, 0, 0), 0.7, true)
        .unwrap();
    let original = fs::read_to_string(&path).unwrap();
    for opacity in [f32::NAN, -0.1, 1.1] {
        assert!(
            config
                .save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), opacity, true)
                .is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }
    for invalid in [
        "broken = [",
        "preview_fit = 'stretch'",
        "background_opacity = 2.0",
    ] {
        fs::write(&path, invalid).unwrap();
        assert!(
            config
                .save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), 0.5, true)
                .is_err()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), invalid);
    }
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn directory_creation_failure_does_not_replace_obstacle() {
    let home = tempfile::tempdir().unwrap();
    let obstacle = home.path().join(".config");
    fs::write(&obstacle, "not a directory").unwrap();
    let config = Config::parse("", home.path()).unwrap();
    assert!(
        config
            .save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), 0.5, true)
            .is_err()
    );
    assert_eq!(fs::read_to_string(obstacle).unwrap(), "not a directory");
}

#[test]
fn capture_preferences_roundtrip_without_clobbering_appearance() {
    use crate::capture_settings::{CameraMode, CapturePreferences, PhotoAspect, VideoQuality};
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".config/khamura/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "theme_color = '#123456' # Appearance\n[capture]\ngrid = false # Guides\n").unwrap();
    let config = Config::parse("", home.path()).unwrap();
    let preferences = CapturePreferences {
        mode: CameraMode::Video, timer_seconds: 3, aspect: PhotoAspect::Square,
        grid: true, microphone_on: false,
        quality: Some(VideoQuality { width: 1920, height: 1080, fps: 30 }),
        camera_device: Some("/dev/video2".into()), microphone_device: Some("alsa_input.test".into()),
    };
    config.save_capture(&preferences).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("# Appearance"));
    assert!(text.contains("# Guides"));
    let reloaded = Config::parse(&text, home.path()).unwrap();
    assert_eq!(reloaded.theme_color, Rgb::new(0x12, 0x34, 0x56));
    assert_eq!(reloaded.capture.quality, preferences.quality);
    assert_eq!(reloaded.capture.camera_device, preferences.camera_device);
    assert!(!reloaded.capture.microphone_on);
    config.save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), 0.5, false).unwrap();
    let reloaded = Config::parse(&fs::read_to_string(&path).unwrap(), home.path()).unwrap();
    assert_eq!(reloaded.capture.mode, CameraMode::Video);
    assert_eq!(reloaded.capture.aspect, PhotoAspect::Square);
    config.save_capture(&CapturePreferences::default()).unwrap();
    let reloaded = Config::parse(&fs::read_to_string(&path).unwrap(), home.path()).unwrap();
    assert!(reloaded.capture.quality.is_none());
    assert!(reloaded.capture.camera_device.is_none());
    assert_eq!(reloaded.theme_color, Rgb::new(1, 2, 3));
}

#[test]
fn capture_rejects_invalid_settings_and_accepts_legacy_config() {
    let home = Path::new("/home/test");
    assert!(Config::parse("", home).unwrap().capture.microphone_on);
    for settings in [
        "timer_seconds = 5", "unknown = true", "camera_device = ''",
        "quality = {width = 1920, height = 0, fps = 30}",
        "quality = {width = 1920, height = 1080, fps = 0}",
    ] {
        assert!(Config::parse(&format!("[capture]\n{settings}"), home).is_err(), "{settings}");
    }
}
