use super::decode::Playback;
use crate::runtime_tools::{self, Tool};
use std::{
    io::Read,
    os::fd::AsRawFd,
    path::Path,
    process::{Child, ExitStatus, Stdio},
    time::{Duration, Instant},
};
pub(super) struct Process(pub Child);
impl Process {
    pub fn pause(&self, paused: bool) {
        unsafe {
            libc::kill(
                self.0.id() as i32,
                if paused { libc::SIGSTOP } else { libc::SIGCONT },
            );
        }
    }
    pub fn finish(&mut self, state: &Playback) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(status) = self.0.try_wait().map_err(|e| e.to_string())? {
                return Ok(status);
            }
            if state.stopped() || Instant::now() > deadline {
                return Err("Playback process did not finish".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub(super) fn has_audio(path: &Path, state: &Playback) -> Result<bool, String> {
    let mut process = Process(
        runtime_tools::command(Tool::Ffprobe)
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
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Could not inspect audio: {e}"))?,
    );
    let mut pipe = process.0.stdout.take().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut found = false;
    loop {
        if state.stopped() || Instant::now() > deadline {
            return Err("Audio inspection cancelled or timed out".into());
        }
        let mut fd = libc::pollfd {
            fd: pipe.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        if unsafe { libc::poll(&mut fd, 1, 100) } <= 0 {
            continue;
        }
        let mut bytes = [0; 256];
        match pipe.read(&mut bytes) {
            Ok(0) => break,
            Ok(_) => found = true,
            Err(error) => return Err(error.to_string()),
        }
    }
    if !process.finish(state)?.success() {
        return Err("Could not inspect video audio".into());
    }
    Ok(found)
}

pub(super) fn audio(path: &Path, seconds: f64) -> Result<Process, String> {
    runtime_tools::command(Tool::Ffplay)
        .args([
            "-v",
            "error",
            "-vn",
            "-nodisp",
            "-autoexit",
            "-ss",
            &seconds.to_string(),
            "-i",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .map(Process)
        .map_err(|e| format!("Could not start audio playback: {e}"))
}
