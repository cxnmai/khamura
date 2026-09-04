//! Resolve the chosen source explicitly: Pulse may silently fall back for unknown names.
use serde_json::Value;
use std::io::{Read, Seek};
use std::process::Stdio;
use std::time::{Duration, Instant};

pub(super) fn resolve(requested: &str) -> Result<String, String> {
    let name = if requested == "default" {
        String::from_utf8(query(&["get-default-source"])?)
            .map_err(|e| format!("Invalid default microphone name: {e}"))?
            .trim()
            .to_string()
    } else {
        requested.to_string()
    };
    let sources = query(&["--format=json", "list", "sources"])?;
    validate(&sources, &name)?;
    Ok(name)
}

fn validate(json: &[u8], requested: &str) -> Result<(), String> {
    let sources: Vec<Value> =
        serde_json::from_slice(json).map_err(|e| format!("Cannot read microphone list: {e}"))?;
    let usable = sources.iter().any(|source| {
        source["name"].as_str() == Some(requested)
            && !requested.ends_with(".monitor")
            && match &source["monitor_of_sink"] {
                Value::Null => true,
                Value::String(value) => value == "n/a",
                Value::Number(value) => value.as_u64() == Some(u64::from(u32::MAX)),
                _ => false,
            }
    });
    if usable {
        Ok(())
    } else {
        Err(format!(
            "Microphone '{requested}' is unavailable or is a system-audio monitor. Select a microphone in Settings or turn Mic off."
        ))
    }
}

/// Capture output through a file so a full stdout pipe cannot deadlock the timeout.
fn query(args: &[&str]) -> Result<Vec<u8>, String> {
    let mut output = tempfile::tempfile().map_err(|e| format!("Cannot query microphone: {e}"))?;
    let mut child = crate::runtime_tools::command(crate::runtime_tools::Tool::Pactl)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output.try_clone().map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| format!("Cannot query microphone (install pactl): {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => {
                return Err(
                    "Cannot query microphones: PulseAudio/PipeWire audio server is unavailable"
                        .into(),
                );
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(e) => format!("Cannot query microphone: {e}"),
                    _ => "Microphone discovery timed out".into(),
                });
            }
        }
    }
    output.rewind().map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    output
        .take(1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCES: &[u8] = br#"[
        {"name":"usb-mic","monitor_of_sink":4294967295},
        {"name":"analog-mic","monitor_of_sink":"n/a"},
        {"name":"sink.monitor","monitor_of_sink":0},
        {"name":"renamed-monitor","monitor_of_sink":3}
    ]"#;

    #[test]
    fn accepts_exact_microphones_but_never_falls_back_or_records_monitors() {
        assert!(validate(SOURCES, "usb-mic").is_ok());
        assert!(validate(SOURCES, "analog-mic").is_ok());
        assert!(validate(SOURCES, "missing-mic").is_err());
        assert!(validate(SOURCES, "sink.monitor").is_err());
        assert!(validate(SOURCES, "renamed-monitor").is_err());
        assert!(validate(b"[]", "default").is_err());
    }
}
