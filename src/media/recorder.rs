use super::encoder::Encoder;
use image::RgbaImage;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Recorder {
    frame: Arc<Mutex<Arc<RgbaImage>>>,
    stopped: Arc<Mutex<Option<Instant>>>,
}

impl Recorder {
    pub fn start(
        output: &Path,
        initial: Arc<RgbaImage>,
        mirror: bool,
        microphone: Option<String>,
        fps: u32,
    ) -> Result<(Self, async_channel::Receiver<Result<PathBuf, String>>), String> {
        if initial.width() == 0 || initial.height() == 0 || fps == 0 || fps > 120 {
            return Err("Invalid recording dimensions or frame rate".into());
        }
        let (temporary, destination) = super::destination(output, "mp4")?;
        let encoder = Encoder::launch(
            temporary.path(),
            initial.width(),
            initial.height(),
            fps,
            mirror,
            microphone.as_deref(),
        )?;
        let recorder = Self {
            frame: Arc::new(Mutex::new(initial)),
            stopped: Arc::new(Mutex::new(None)),
        };
        let frame = recorder.frame.clone();
        let stopped = recorder.stopped.clone();
        let (sender, receiver) = async_channel::bounded(1);
        std::thread::spawn(move || {
            let result = record(encoder, frame, stopped, fps)
                .and_then(|()| super::publish(temporary, destination));
            let _ = sender.send_blocking(result);
        });
        Ok((recorder, receiver))
    }

    pub fn update_frame(&self, frame: Arc<RgbaImage>) {
        if let Ok(mut latest) = self.frame.lock() {
            if latest.dimensions() == frame.dimensions() {
                *latest = frame;
            }
        }
    }

    pub fn stop(&self) {
        self.stopped
            .lock()
            .unwrap()
            .get_or_insert_with(Instant::now);
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.stop();
    }
}

fn record(
    mut encoder: Encoder,
    latest: Arc<Mutex<Arc<RgbaImage>>>,
    stopped: Arc<Mutex<Option<Instant>>>,
    fps: u32,
) -> Result<(), String> {
    let mut stdin = encoder.child.stdin.take().unwrap();
    // Nonblocking writes let stop and encoder failure interrupt a stalled pipe.
    let fd = stdin.as_raw_fd();
    // SAFETY: fd is borrowed from a live pipe, and these calls do not transfer ownership.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(format!(
            "Cannot configure recording pipe: {}",
            std::io::Error::last_os_error()
        ));
    }
    let period = Duration::from_secs_f64(1.0 / f64::from(fps));
    let started = Instant::now();
    let mut next = started;
    let mut frames_written = 0_u64;
    let mut stop_deadline = None;
    loop {
        let frame = latest.lock().unwrap().clone();
        let mut bytes = frame.as_raw().as_slice();
        while !bytes.is_empty() {
            if stopped.lock().unwrap().is_some() && stop_deadline.is_none() {
                stop_deadline = Some(Instant::now() + Duration::from_secs(10));
            }
            if stop_deadline.is_some_and(|deadline| Instant::now() > deadline) {
                return Err("Video encoder did not respond while stopping".into());
            }
            if let Some(status) = encoder.child.try_wait().map_err(|e| e.to_string())? {
                return Err(encoder.error(&format!("Video encoder exited unexpectedly ({status})")));
            }
            match stdin.write(bytes) {
                Ok(0) => return Err(encoder.error("Video encoder closed its input")),
                Ok(n) => bytes = &bytes[n..],
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(2))
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(encoder.error(&format!("Cannot send video frame: {e}"))),
            }
        }
        frames_written += 1;
        if let Some(stop_time) = *stopped.lock().unwrap() {
            let target = (stop_time.saturating_duration_since(started).as_secs_f64()
                * f64::from(fps))
            .ceil() as u64;
            if frames_written >= target.max(1) {
                break;
            }
        }
        next += period;
        // Never queue stale frames: repeat the latest image at each output timestamp.
        // If encoding falls behind, catch up using current frames to retain wall-clock duration.
        while Instant::now() < next && !stopped.lock().unwrap().is_some() {
            std::thread::sleep(
                next.saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(5)),
            );
        }
    }
    drop(stdin);
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match encoder.child.try_wait().map_err(|e| e.to_string())? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => {
                return Err(encoder.error(&format!("Cannot finalize recording ({status})")));
            }
            None if Instant::now() > deadline => {
                return Err(encoder.error("Video finalization timed out"));
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

#[cfg(test)]
#[path = "recorder_tests.rs"]
mod tests;
