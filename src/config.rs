use crate::theme::{CAMERA_LETTERBOX_COLOR, CAMERA_LETTERBOX_OPACITY, Rgb};
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Config {
    pub photo_directory: PathBuf,
    pub theme_color: Rgb,
    pub background_opacity: f32,
}

impl gpui::Global for Config {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    photo_directory: Option<String>,
    theme_color: Option<String>,
    background_opacity: Option<f32>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let home = env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or("HOME is not set")?;
        let path = home.join(".config/khamura/config.toml");
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        Self::parse(&text, &home).map_err(|error| format!("{}: {error}", path.display()))
    }

    fn parse(text: &str, home: &Path) -> Result<Self, String> {
        let raw: FileConfig = toml::from_str(text).map_err(|error| error.to_string())?;
        let photo_directory = match raw.photo_directory.as_deref() {
            None => home.join("Pictures/khamura"),
            Some("~") => home.to_path_buf(),
            Some(path) if path.starts_with("~/") => home.join(&path[2..]),
            Some(path) if Path::new(path).is_absolute() => PathBuf::from(path),
            Some(_) => {
                return Err("photo_directory must be an absolute path or start with ~/".into());
            }
        };
        let theme_color = match raw.theme_color {
            None => CAMERA_LETTERBOX_COLOR,
            Some(color) => parse_color(&color)?,
        };
        let background_opacity = raw.background_opacity.unwrap_or(CAMERA_LETTERBOX_OPACITY);
        if !background_opacity.is_finite() || !(0.0..=1.0).contains(&background_opacity) {
            return Err("background_opacity must be between 0.0 and 1.0".into());
        }
        Ok(Self {
            photo_directory,
            theme_color,
            background_opacity,
        })
    }
}

fn parse_color(color: &str) -> Result<Rgb, String> {
    let hex = color
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("theme_color must be a hex RGB color such as #034d70")?;
    let value = u32::from_str_radix(hex, 16).map_err(|error| error.to_string())?;
    Ok(Rgb::new(
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_partial_overrides() {
        let home = Path::new("/home/test");
        let config = Config::parse("", home).unwrap();
        assert_eq!(config.photo_directory, home.join("Pictures/khamura"));
        assert_eq!(config.theme_color, CAMERA_LETTERBOX_COLOR);
        assert_eq!(config.background_opacity, CAMERA_LETTERBOX_OPACITY);
        let config = Config::parse(
            "photo_directory = '~/Photos'\ntheme_color = '#034d70'\nbackground_opacity = 0.25",
            home,
        )
        .unwrap();
        assert_eq!(config.photo_directory, home.join("Photos"));
        assert_eq!(config.theme_color, Rgb::new(3, 77, 112));
        assert_eq!(config.background_opacity, 0.25);
        assert_eq!(
            Config::parse("photo_directory = '/tmp/photos'", home)
                .unwrap()
                .photo_directory,
            Path::new("/tmp/photos")
        );
    }

    #[test]
    fn rejects_invalid_settings() {
        for text in [
            "theme_color = 'red'",
            "theme_color = '#１２３'",
            "background_opacity = 1.1",
            "background_opacity = nan",
            "photo_directory = ''",
            "photo_directory = 'relative'",
            "unknown = true",
            "not toml",
        ] {
            assert!(
                Config::parse(text, Path::new("/home/test")).is_err(),
                "{text}"
            );
        }
    }
}
