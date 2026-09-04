use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub(super) struct Encoder {
    pub child: Child,
    errors: Arc<Mutex<Vec<u8>>>,
}

impl Encoder {
    pub fn launch(
        path: &Path,
        width: u32,
        height: u32,
        fps: u32,
        mirror: bool,
        microphone: Option<&str>,
    ) -> Result<Self, String> {
        let microphone = microphone.map(super::microphone::resolve).transpose()?;
        let microphone = microphone.as_deref();
        let mut command = Command::new("ffmpeg");
        command.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "bgra",
            "-video_size",
        ]);
        command.arg(format!("{width}x{height}")).args([
            "-framerate",
            &fps.to_string(),
            "-i",
            "pipe:0",
        ]);
        if let Some(device) = microphone {
            command.args(["-thread_queue_size", "512", "-f", "pulse", "-i", device]);
        }
        command.args(["-map", "0:v:0", "-vf"]);
        // H.264 yuv420p requires even dimensions. Pad rather than discard camera pixels.
        command.arg(if mirror {
            "hflip,pad=ceil(iw/2)*2:ceil(ih/2)*2"
        } else {
            "pad=ceil(iw/2)*2:ceil(ih/2)*2"
        });
        command.args([
            "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p",
        ]);
        if microphone.is_some() {
            command.args([
                "-map",
                "1:a:0",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-af",
                "aresample=async=1",
                "-shortest",
            ]);
        }
        command
            .args(["-movflags", "+faststart", "-f", "mp4"])
            .arg(path);
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Cannot start FFmpeg (install ffmpeg with H.264 support): {e}"))?;
        let mut stderr = child.stderr.take().unwrap();
        let errors = Arc::new(Mutex::new(Vec::new()));
        let captured = errors.clone();
        std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            while let Ok(n) = stderr.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                let mut tail = captured.lock().unwrap();
                tail.extend_from_slice(&buffer[..n]);
                let excess = tail.len().saturating_sub(8192);
                tail.drain(..excess);
            }
        });
        Ok(Self { child, errors })
    }

    pub fn error(&self, context: &str) -> String {
        let bytes = self.errors.lock().unwrap();
        let details = String::from_utf8_lossy(&bytes);
        if details.trim().is_empty() {
            context.to_string()
        } else {
            format!("{context}: {}", details.trim())
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
