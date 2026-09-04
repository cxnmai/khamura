use crate::{config::Config, theme::Rgb};
use gpui::{App, BorrowAppContext};
use serde::{Deserialize, Serialize};

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
    pub theme_color: Rgb,
    pub background_opacity: f32,
    pub save_error: Option<String>,
    pub dirty: bool,
}

impl gpui::Global for SessionSettings {}

impl SessionSettings {
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
            theme_color: config.theme_color,
            background_opacity: config.background_opacity,
            save_error: None,
            dirty: false,
        }
    }
}
