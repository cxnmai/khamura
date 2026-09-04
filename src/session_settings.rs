use crate::{config::Config, theme::Rgb};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraFit {
    #[default]
    Contain,
    Cover,
}

/// Editable UI preferences. These never write back to the startup config.
pub struct SessionSettings {
    pub fit: CameraFit,
    pub theme_color: Rgb,
    pub background_opacity: f32,
}

impl gpui::Global for SessionSettings {}

impl From<&Config> for SessionSettings {
    fn from(config: &Config) -> Self {
        Self {
            fit: CameraFit::default(),
            theme_color: config.theme_color,
            background_opacity: config.background_opacity,
        }
    }
}
