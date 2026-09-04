use crate::runtime_tools::{self, Tool};
use async_channel::Sender;
use std::{
    io::Read,
    os::fd::AsRawFd,
    path::PathBuf,
    process::{Child, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub(super) const WIDTH: u32 = 960;
pub(super) const HEIGHT: u32 = 540;
pub(super) enum Event {
    Frame(Vec<u8>),
    End,
    Error(String),
}
pub(super) struct Playback {
    pub paused: AtomicBool,
    stop: AtomicBool,
}
impl Playback {
    pub fn start(path: PathBuf, tx: Sender<Event>) -> Arc<Self> {
        let state = Arc::new(Self {
            paused: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        });
        let worker = state.clone();
        std::thread::spawn(move || {
            if let Err(error) = decode(path, &worker, &tx) {
                let _ = tx.send_blocking(Event::Error(error));
            } else if !worker.stop.load(Ordering::Relaxed) {
                let _ = tx.send_blocking(Event::End);
            }
        });
        state
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
struct Process(Child);
impl Process {
    fn pause(&self, paused: bool) {
        unsafe {
            libc::kill(
                self.0.id() as i32,
                if paused { libc::SIGSTOP } else { libc::SIGCONT },
            );
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn decode(path: PathBuf, state: &Playback, tx: &Sender<Event>) -> Result<(), String> {
    let mut video = Process(runtime_tools::command(Tool::Ffmpeg)
        .args(["-v", "error", "-nostdin", "-re", "-i"]).arg(&path)
        .args(["-an", "-vf", "fps=24,scale=960:540:force_original_aspect_ratio=decrease,pad=960:540:(ow-iw)/2:(oh-ih)/2", "-pix_fmt", "bgra", "-f", "rawvideo", "pipe:1"])
        .stdin(Stdio::null()).stderr(Stdio::null()).stdout(Stdio::piped()).spawn().map_err(|e| format!("Could not play video: {e}"))?);
    let probe = runtime_tools::command(Tool::Ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
        ])
        .arg(&path)
        .output()
        .map_err(|e| format!("Could not inspect audio: {e}"))?;
    let mut audio = if probe.stdout.is_empty() {
        None
    } else {
        Some(Process(
            runtime_tools::command(Tool::Ffplay)
                .args(["-v", "error", "-vn", "-nodisp", "-autoexit", "-i"])
                .arg(&path)
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .map_err(|e| format!("Could not start audio playback: {e}"))?,
        ))
    };
    let mut stdout = video.0.stdout.take().unwrap();
    let mut paused = false;
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
    let mut offset = 0;
    let mut frames = 0;
    while !state.stop.load(Ordering::Relaxed) && !tx.is_closed() {
        let next_pause = state.paused.load(Ordering::Relaxed);
        if next_pause != paused {
            video.pause(next_pause);
            if let Some(audio) = &audio {
                audio.pause(next_pause);
            }
            paused = next_pause;
        }
        if paused {
            std::thread::sleep(std::time::Duration::from_millis(30));
            continue;
        }
        let mut poll = libc::pollfd {
            fd: stdout.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll, 1, 100) };
        if ready < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e.to_string());
        }
        if ready == 0 {
            continue;
        }
        match stdout.read(&mut pixels[offset..]) {
            Ok(0) => break,
            Ok(count) => offset += count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Video playback failed: {e}")),
        }
        if offset == pixels.len() {
            frames += 1;
            let _ = tx.try_send(Event::Frame(pixels.clone()));
            offset = 0;
        }
    }
    if state.stop.load(Ordering::Relaxed) || tx.is_closed() {
        return Ok(());
    }
    if frames == 0 || !video.0.wait().map_err(|e| e.to_string())?.success() {
        return Err("Could not decode this video".into());
    }
    if let Some(audio) = &mut audio {
        if let Ok(Some(status)) = audio.0.try_wait() {
            if !status.success() {
                return Err("Audio playback unavailable for this video".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "video_decode_tests.rs"]
mod tests;
