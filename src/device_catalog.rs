//! Device discovery runs on the background executor, never on the render thread.
use nokhwa::utils::{ApiBackend, CameraIndex};
use serde::Deserialize;
use std::process::Command;

#[derive(Clone)]
pub struct Device {
    pub id: String,
    pub label: String,
}

#[derive(Default)]
pub struct DeviceCatalog {
    pub cameras: Vec<Device>,
    pub microphones: Vec<Device>,
    pub errors: Vec<String>,
}

impl DeviceCatalog {
    pub fn discover() -> Self {
        let mut result = Self::default();
        match nokhwa::query(ApiBackend::Video4Linux) {
            Ok(cameras) => result.cameras = cameras.into_iter().map(|camera| Device {
                id: match camera.index() {
                    CameraIndex::Index(index) => format!("/dev/video{index}"),
                    CameraIndex::String(path) => path.clone(),
                },
                label: camera.human_name(),
            }).collect(),
            Err(error) => result.errors.push(format!("Camera discovery: {error}")),
        }
        match microphones() {
            Ok(devices) => result.microphones = devices,
            Err(error) => result.errors.push(format!("Microphone discovery: {error}")),
        }
        result
    }
}

#[derive(Deserialize)]
struct Source {
    name: String,
    description: Option<String>,
    monitor_of_sink: Option<serde_json::Value>,
}

fn microphones() -> Result<Vec<Device>, String> {
    let output = Command::new("pactl")
        .args(["--format=json", "list", "sources"])
        .output().map_err(|error| format!("could not run pactl: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let sources: Vec<Source> = serde_json::from_slice(&output.stdout)
        .map_err(|error| error.to_string())?;
    Ok(sources.into_iter().filter(|source| {
        !source.name.ends_with(".monitor") && match &source.monitor_of_sink {
            None | Some(serde_json::Value::Null) => true,
            Some(serde_json::Value::String(value)) => value == "n/a",
            Some(serde_json::Value::Number(value)) => value.as_u64() == Some(u32::MAX as u64),
            _ => false,
        }
    }).map(|source| Device {
        label: source.description.unwrap_or_else(|| source.name.clone()),
        id: source.name,
    }).collect())
}
