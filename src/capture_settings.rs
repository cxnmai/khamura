use crate::config::Config;
use gpui::{App, BorrowAppContext};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CameraMode {
    #[default]
    Photo,
    Video,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhotoAspect {
    #[default]
    Native,
    FourThree,
    SixteenNine,
    Square,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VideoQuality {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CapturePreferences {
    pub mode: CameraMode,
    pub timer_seconds: u64,
    pub aspect: PhotoAspect,
    pub grid: bool,
    pub microphone_on: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<VideoQuality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone_device: Option<String>,
}

impl Default for CapturePreferences {
    fn default() -> Self {
        Self {
            mode: CameraMode::Photo,
            timer_seconds: 0,
            aspect: PhotoAspect::Native,
            grid: false,
            microphone_on: true,
            quality: None,
            camera_device: None,
            microphone_device: None,
        }
    }
}

impl CapturePreferences {
    pub fn validate(&self) -> Result<(), String> {
        if ![0, 3, 10].contains(&self.timer_seconds) {
            return Err("capture.timer_seconds must be 0, 3, or 10".into());
        }
        if self
            .quality
            .is_some_and(|q| q.width == 0 || q.height == 0 || q.fps == 0)
        {
            return Err("capture.quality dimensions and fps must be positive".into());
        }
        if [&self.camera_device, &self.microphone_device]
            .iter()
            .any(|device| device.as_ref().is_some_and(|value| value.trim().is_empty()))
        {
            return Err("capture device names must not be empty; omit to use the default".into());
        }
        Ok(())
    }
}

pub struct CaptureSettings {
    pub mode: CameraMode,
    pub timer_seconds: u64,
    pub aspect: PhotoAspect,
    pub grid: bool,
    pub microphone_on: bool,
    pub quality: Option<VideoQuality>,
    pub qualities: Vec<VideoQuality>,
    pub camera_device: Option<String>,
    pub microphone_device: Option<String>,
    pub busy: bool,
    pub recording: bool,
    pub save_error: Option<String>,
}
impl gpui::Global for CaptureSettings {}

impl From<&Config> for CaptureSettings {
    fn from(config: &Config) -> Self {
        let p = &config.capture;
        Self {
            mode: p.mode,
            timer_seconds: p.timer_seconds,
            aspect: p.aspect,
            grid: p.grid,
            microphone_on: p.microphone_on,
            quality: p.quality,
            qualities: Vec::new(),
            camera_device: p.camera_device.clone(),
            microphone_device: p.microphone_device.clone(),
            busy: false,
            recording: false,
            save_error: None,
        }
    }
}

impl CaptureSettings {
    pub fn change(cx: &mut App, update: impl FnOnce(&mut Self)) {
        if cx.global::<Self>().busy {
            return;
        }
        cx.update_global::<Self, _>(|settings, _| update(settings));
        Self::save(cx);
    }

    pub fn save(cx: &mut App) {
        let s = cx.global::<Self>();
        let preferences = CapturePreferences {
            mode: s.mode,
            timer_seconds: s.timer_seconds,
            aspect: s.aspect,
            grid: s.grid,
            microphone_on: s.microphone_on,
            quality: s.quality,
            camera_device: s.camera_device.clone(),
            microphone_device: s.microphone_device.clone(),
        };
        let result = cx.global::<Config>().save_capture(&preferences);
        if result.is_ok() {
            cx.update_global::<Config, _>(|config, _| config.capture = preferences);
        }
        cx.update_global::<Self, _>(|settings, _| settings.save_error = result.err());
    }
}
