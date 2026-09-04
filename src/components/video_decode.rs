use super::process::{Process, has_audio};
use crate::runtime_tools::{self, Tool};
use async_channel::Sender;
use std::time::{Duration, Instant};
use std::{
    io::Read,
    os::fd::AsRawFd,
    path::PathBuf,
    process::Stdio,
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
                if worker.stopped() {
                    return;
                }
                let _ = tx.send_blocking(Event::Error(error));
            } else if !worker.stop.load(Ordering::Relaxed) {
                let _ = tx.send_blocking(Event::End);
            }
        });
        state
    }
    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
fn decode(path: PathBuf, state: &Playback, tx: &Sender<Event>) -> Result<(), String> {
    let mut video = Process(runtime_tools::command(Tool::Ffmpeg)
        .args(["-v", "error", "-nostdin", "-threads", "2", "-filter_threads", "1", "-i"]).arg(&path)
        .args(["-an", "-vf", "fps=24,scale=960:540:force_original_aspect_ratio=decrease,pad=960:540:(ow-iw)/2:(oh-ih)/2", "-pix_fmt", "bgra", "-threads", "1", "-f", "rawvideo", "pipe:1"])
        .stdin(Stdio::null()).stderr(Stdio::null()).stdout(Stdio::piped()).spawn().map_err(|e| format!("Could not play video: {e}"))?);
    let audio_present = has_audio(&path, state)?;
    let mut audio = if audio_present {
        Some(super::process::audio(&path, 0.)?)
    } else {
        None
    };
    let mut stdout = video.0.stdout.take().unwrap();
    // Best effort: avoid hundreds of tiny kernel pipe transfers per BGRA frame.
    // SAFETY: stdout owns this live pipe descriptor; failure preserves its default.
    unsafe { libc::fcntl(stdout.as_raw_fd(), libc::F_SETPIPE_SZ, 1024 * 1024) };
    let mut paused = false;
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
    let mut offset = 0;
    let mut frames = 0;
    let mut next_frame = Instant::now();
    let mut progress = Instant::now();
    while !state.stop.load(Ordering::Relaxed) && !tx.is_closed() {
        let next_pause = state.paused.load(Ordering::Relaxed);
        if next_pause != paused {
            video.pause(next_pause);
            if next_pause {
                audio.take();
            } else if audio_present {
                audio = Some(super::process::audio(&path, frames as f64 / 24.)?);
            }
            paused = next_pause;
            next_frame = Instant::now();
            progress = Instant::now();
        }
        if paused {
            std::thread::sleep(std::time::Duration::from_millis(30));
            continue;
        }
        if let Some(wait) = next_frame.checked_duration_since(Instant::now()) {
            // Sleep to the frame deadline, but still react to pause/close promptly.
            std::thread::sleep(wait.min(Duration::from_millis(20)));
            continue;
        }
        if progress.elapsed() > Duration::from_secs(10) {
            return Err("Video decoder stalled".into());
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
            Ok(count) => {
                offset += count;
                progress = Instant::now();
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Video playback failed: {e}")),
        }
        if offset == pixels.len() {
            next_frame += Duration::from_secs_f64(1. / 24.);
            frames += 1;
            // A hidden/busy UI must not copy megabytes merely to drop the frame.
            if !tx.is_full() {
                let _ = tx.try_send(Event::Frame(pixels.clone()));
            }
            offset = 0;
        }
    }
    if state.stop.load(Ordering::Relaxed) || tx.is_closed() {
        return Ok(());
    }
    if frames == 0 || !video.finish(state)?.success() {
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
