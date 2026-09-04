use super::*;

#[test]
fn output_updates_preserve_comments_and_survive_reload() {
    let home = tempfile::tempdir().unwrap();
    let mut config = Config::load_from_home(home.path()).unwrap();
    let path = config.path().to_path_buf();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "# Camera\nphoto_directory = '~/Old' # Output\n").unwrap();
    let output = home.path().join("New Photos");
    config.set_output_path(&output).unwrap();
    assert_eq!(config.photo_directory, output);
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains("# Camera"));
    assert!(text.contains("# Output"));
    assert_eq!(
        Config::load_from_home(home.path()).unwrap().photo_directory,
        output
    );
    assert!(config.set_output_path(Path::new("relative")).is_err());
    let obstacle = home.path().join("file");
    fs::write(&obstacle, "keep").unwrap();
    assert!(config.set_output_path(&obstacle).is_err());
    assert_eq!(config.photo_directory, output);
}

#[test]
fn relocation_preserves_latest_preferences_and_restart_location() {
    let home = tempfile::tempdir().unwrap();
    let mut config = Config::load_from_home(home.path()).unwrap();
    config
        .save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), 0.5, false)
        .unwrap();
    let original_path = config.path().to_path_buf();
    let original = format!(
        "# Keep this\n{}",
        fs::read_to_string(&original_path).unwrap()
    );
    fs::write(&original_path, &original).unwrap();
    let target = home.path().join("custom/camera.toml");
    config.relocate(&target).unwrap();
    assert!(!original_path.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
    let reloaded = Config::load_from_home(home.path()).unwrap();
    assert_eq!(reloaded.path(), target);
    assert_eq!(reloaded.preview_fit, CameraFit::Cover);
    assert!(!reloaded.mirror);
    assert_eq!(reloaded.theme_color, Rgb::new(1, 2, 3));
    config.relocate(&target).unwrap();
}

#[test]
fn relocation_failure_preserves_original_and_in_memory_path() {
    let home = tempfile::tempdir().unwrap();
    let mut config = Config::load_from_home(home.path()).unwrap();
    config
        .save_preferences(CameraFit::Cover, Rgb::new(1, 2, 3), 0.5, false)
        .unwrap();
    let old = config.path().to_path_buf();
    let original = fs::read_to_string(&old).unwrap();
    let target = home.path().join("custom.toml");
    fs::write(&target, "do not overwrite").unwrap();
    assert!(config.relocate(&target).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), "do not overwrite");
    fs::remove_file(&target).unwrap();
    // Force the atomic location-marker replacement to fail.
    fs::create_dir(home.path().join(".config/khamura/config-path")).unwrap();
    assert!(config.relocate(&target).is_err());
    assert!(!target.exists());
    assert_eq!(config.path(), old);
    assert_eq!(fs::read_to_string(old).unwrap(), original);
}

#[test]
fn missing_config_materializes_and_bad_markers_do_not_use_defaults() {
    let home = tempfile::tempdir().unwrap();
    let mut config = Config::load_from_home(home.path()).unwrap();
    let target = home.path().join("camera.toml");
    config.relocate(&target).unwrap();
    let reloaded = Config::load_from_home(home.path()).unwrap();
    assert_eq!(reloaded.photo_directory, config.photo_directory);
    assert!(reloaded.mirror);
    assert!(
        fs::read_to_string(&target)
            .unwrap()
            .contains("mirror = true")
    );
    assert!(
        fs::read_to_string(&target)
            .unwrap()
            .contains("photo_directory")
    );
    fs::remove_file(&target).unwrap();
    assert!(Config::load_from_home(home.path()).is_err());
    fs::write(home.path().join(".config/khamura/config-path"), "relative").unwrap();
    assert!(Config::load_from_home(home.path()).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_marker_aliases_and_symlink_sources() {
    use std::os::unix::fs::symlink;
    let home = tempfile::tempdir().unwrap();
    let mut config = Config::load_from_home(home.path()).unwrap();
    let directory = home.path().join(".config/khamura");
    fs::create_dir_all(&directory).unwrap();
    let alias = home.path().join("alias");
    symlink(&directory, &alias).unwrap();
    assert!(config.relocate(&alias.join("config-path")).is_err());
    assert!(!directory.join("config-path").exists());
    let external = home.path().join("external.toml");
    fs::write(&external, "# managed externally").unwrap();
    symlink(&external, config.path()).unwrap();
    assert!(config.relocate(&home.path().join("new.toml")).is_err());
    assert_eq!(
        fs::read_to_string(external).unwrap(),
        "# managed externally"
    );
    symlink(home.path().join("missing"), directory.join("config-path")).unwrap();
    assert!(Config::load_from_home(home.path()).is_err());
}
