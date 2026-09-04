//! A stable pointer keeps relocated configuration discoverable on next launch.
use crate::{config::Config, config_store};
use std::{fs, io::Write, path::{Path, PathBuf}};

fn marker(home: &Path) -> PathBuf {
    home.join(".config/khamura/config-path")
}

pub(crate) fn resolve(home: &Path) -> Result<(PathBuf, bool), String> {
    let marker = marker(home);
    match fs::read_to_string(&marker) {
        Ok(text) => {
            let path = PathBuf::from(text);
            validate(&path)?;
            if !path.is_file() {
                return Err(format!("{}: configuration file does not exist", path.display()));
            }
            Ok((path, true))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok((home.join(".config/khamura/config.toml"), false))
        }
        Err(error) => Err(format!("{}: {error}", marker.display())),
    }
}

pub(crate) fn validate(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.to_str().is_none() {
        return Err("path must be absolute and valid UTF-8".into());
    }
    if path.components().any(|part| matches!(part, std::path::Component::ParentDir)) {
        return Err("path must not contain .. components".into());
    }
    Ok(())
}

pub(crate) fn relocate(config: &Config, home: &Path, target: &Path) -> Result<(), String> {
    validate(target)?;
    if target == config.path() {
        return Ok(());
    }
    if target == marker(home) {
        return Err("configuration cannot replace the location marker".into());
    }
    let text = match fs::read_to_string(config.path()) {
        Ok(text) => {
            Config::parse(&text, home)?;
            text
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut document = toml_edit::DocumentMut::new();
            document["photo_directory"] = toml_edit::value(
                config.photo_directory.to_str().ok_or("output path must be valid UTF-8")?);
            document["preview_fit"] = toml_edit::value(match config.preview_fit {
                crate::session_settings::CameraFit::Contain => "contain",
                crate::session_settings::CameraFit::Cover => "cover",
            });
            let color = config.theme_color;
            document["theme_color"] = toml_edit::value(
                format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b));
            document["background_opacity"] = toml_edit::value(config.background_opacity as f64);
            document.to_string()
        }
        Err(error) => return Err(error.to_string()),
    };
    let parent = target.parent().ok_or("configuration path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    if let Ok(metadata) = fs::metadata(config.path()) {
        pending.as_file().set_permissions(metadata.permissions()).map_err(|error| error.to_string())?;
    }
    pending.write_all(text.as_bytes()).map_err(|error| error.to_string())?;
    pending.as_file().sync_all().map_err(|error| error.to_string())?;
    pending.persist_noclobber(target).map_err(|error| error.to_string())?;
    if let Err(error) = config_store::write_atomic(&marker(home), target.to_str().unwrap()) {
        // The old file and pointer remain authoritative if switching fails.
        let _ = fs::remove_file(target);
        return Err(error);
    }
    // Keeping an old copy is safer than undoing a successful switch if removal fails.
    let _ = fs::remove_file(config.path());
    Ok(())
}
