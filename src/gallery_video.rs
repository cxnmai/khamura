//! Extract one bounded preview without blocking on encoder output pipes.
use crate::runtime_tools::{self, Tool};
use std::{
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};

pub(super) fn thumbnail(path: &Path) -> Option<image::DynamicImage> {
    let directory = tempfile::tempdir().ok()?;
    let output = directory.path().join("thumbnail.png");
    let mut child = runtime_tools::command(Tool::Ffmpeg)
        .args([
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "1",
            "-filter_threads",
            "1",
            "-i",
        ])
        .arg(path)
        .args([
            "-map",
            "0:v:0",
            "-frames:v",
            "1",
            "-an",
            "-vf",
            "scale=240:240:force_original_aspect_ratio=decrease",
            "-threads",
            "1",
            "-y",
        ])
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return image::open(output).ok(),
            Ok(Some(_)) => return None,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_video_preview_at_bounded_size() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sample.mp4");
        let status = runtime_tools::command(Tool::Ffmpeg)
            .args([
                "-nostdin",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=640x360:r=1",
                "-frames:v",
                "1",
                "-c:v",
                "libx264",
                "-threads",
                "1",
                "-pix_fmt",
                "yuv420p",
                "-y",
            ])
            .arg(&path)
            .status()
            .expect("ffmpeg must be available for capture tests");
        assert!(status.success());
        let preview = thumbnail(&path).expect("valid MP4 should produce a thumbnail");
        assert_eq!((preview.width(), preview.height()), (240, 135));
    }
}
