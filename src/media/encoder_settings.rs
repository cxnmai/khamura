use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type Cache = HashMap<(u32, u32), Option<PathBuf>>;

pub(super) fn hardware(width: u32, height: u32) -> Option<PathBuf> {
    if std::env::var("KHAMURA_VIDEO_ENCODER").as_deref() == Ok("software") {
        return None;
    }
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Mutex::default).lock().unwrap();
    if let Some(device) = cache.get(&(width, height)) {
        return device.clone();
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut devices: Vec<_> = std::fs::read_dir("/dev/dri")
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("renderD"))
        .map(|entry| entry.path())
        .collect();
    devices.sort();
    let device = devices
        .into_iter()
        .find(|device| Instant::now() < deadline && probe(device, width, height, deadline));
    if cache.len() >= 16 {
        cache.clear();
    }
    cache.insert((width, height), device.clone());
    device
}

pub(super) fn input(command: &mut Command, device: Option<&std::path::Path>) {
    // Auto thread counts can allocate hundreds of MB and saturate all CPU cores.
    command.args(["-filter_threads", "2"]);
    if let Some(device) = device {
        command.arg("-vaapi_device").arg(device);
    }
}

pub(super) fn output(command: &mut Command, hardware: bool, mirror: bool) {
    // Preserve every camera pixel; H.264's 4:2:0 format requires even dimensions.
    let mut filters = if mirror { "hflip," } else { "" }.to_string();
    filters.push_str("pad=ceil(iw/2)*2:ceil(ih/2)*2");
    if hardware {
        filters.push_str(",format=nv12,hwupload");
    }
    command.args(["-vf", &filters]);
    if hardware {
        // QP is not equivalent to CRF: QP20 was measured against CRF20 before use.
        command.args(["-c:v", "h264_vaapi", "-rc_mode", "CQP", "-qp", "20"]);
    } else {
        command.args([
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-pix_fmt",
            "yuv420p",
            "-threads:v",
            "4",
        ]);
    }
}

fn probe(device: &std::path::Path, width: u32, height: u32, deadline: Instant) -> bool {
    let mut command = crate::runtime_tools::command(crate::runtime_tools::Tool::Ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    input(&mut command, Some(device));
    command.args([
        "-f",
        "lavfi",
        "-i",
        &format!("nullsrc=size={width}x{height}:rate=30,format=bgra"),
    ]);
    output(&mut command, true, true);
    command.args(["-frames:v", "3", "-f", "null", "-"]);
    let Ok(mut child) = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}
