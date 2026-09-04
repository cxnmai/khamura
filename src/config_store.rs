//! Comment-preserving preference updates with atomic replacement.
use crate::{config::Config, session_settings::CameraFit, theme::Rgb};
use std::{fs, io::Write, path::Path};
use toml_edit::{DocumentMut, Value};

pub(crate) fn save(
    path: &Path,
    home: &Path,
    fit: CameraFit,
    color: Rgb,
    opacity: f32,
) -> Result<(), String> {
    if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
        return Err("background_opacity must be between 0.0 and 1.0".into());
    }
    // Re-read each time so manual edits since startup survive.
    let original = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.to_string()),
    };
    Config::parse(&original, home)?;
    let mut document = original
        .parse::<DocumentMut>()
        .map_err(|error| error.to_string())?;
    set(&mut document, "preview_fit", Value::from(match fit {
        CameraFit::Contain => "contain",
        CameraFit::Cover => "cover",
    }));
    set(&mut document, "theme_color", Value::from(format!(
        "#{:02x}{:02x}{:02x}", color.r, color.g, color.b
    )));
    set(&mut document, "background_opacity", Value::from(opacity as f64));
    let updated = document.to_string();
    Config::parse(&updated, home)?;
    let parent = path.parent().ok_or("configuration path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut pending = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| error.to_string())?;
    if let Ok(metadata) = fs::metadata(path) {
        pending.as_file().set_permissions(metadata.permissions())
            .map_err(|error| error.to_string())?;
    }
    pending.write_all(updated.as_bytes()).map_err(|error| error.to_string())?;
    pending.as_file().sync_all().map_err(|error| error.to_string())?;
    pending.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn set(document: &mut DocumentMut, key: &str, mut value: Value) {
    if let Some(previous) = document.get(key).and_then(|item| item.as_value()) {
        *value.decor_mut() = previous.decor().clone();
    }
    document[key] = toml_edit::Item::Value(value);
}

#[cfg(test)]
#[path = "config_store_tests.rs"]
mod tests;
