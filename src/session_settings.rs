use crate::{config::Config, theme::Rgb};
use gpui::{App, BorrowAppContext};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CameraFit {
    #[default]
    Contain,
    Cover,
}

/// Live UI preferences, persisted after discrete changes or a completed drag.
pub struct SessionSettings {
    pub fit: CameraFit,
    pub mirror: bool,
    pub theme_color: Rgb,
    pub background_opacity: f32,
    pub save_error: Option<String>,
    pub dirty: bool,
}

impl gpui::Global for SessionSettings {}

impl SessionSettings {
    pub fn relocate_config(cx: &mut App, path: &Path) -> Result<(), String> {
        Self::save_before_path_change(cx)?;
        cx.update_global::<Config, _>(|config, _| config.relocate(path))
    }

    pub fn set_output_path(cx: &mut App, path: &Path) -> Result<(), String> {
        Self::save_before_path_change(cx)?;
        cx.update_global::<Config, _>(|config, _| config.set_output_path(path))
    }

    fn save_before_path_change(cx: &mut App) -> Result<(), String> {
        Self::save(cx);
        match &cx.global::<Self>().save_error {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    pub fn change(cx: &mut App, update: impl FnOnce(&mut Self)) {
        cx.update_global::<Self, _>(|settings, _| {
            update(settings);
            settings.dirty = true;
        });
        Self::save(cx);
    }

    pub fn save(cx: &mut App) {
        let settings = cx.global::<Self>();
        if !settings.dirty {
            return;
        }
        let result = cx.global::<Config>().save_preferences(
            settings.fit,
            settings.theme_color,
            settings.background_opacity,
            settings.mirror,
        );
        cx.update_global::<Self, _>(|settings, _| {
            settings.save_error = result.err();
            settings.dirty = settings.save_error.is_some();
        });
    }
}

impl From<&Config> for SessionSettings {
    fn from(config: &Config) -> Self {
        Self {
            fit: config.preview_fit,
            mirror: config.mirror,
            theme_color: config.theme_color,
            background_opacity: config.background_opacity,
            save_error: None,
            dirty: false,
        }
    }
}
