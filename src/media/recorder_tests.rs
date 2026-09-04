use super::*;
use std::process::Command;

#[path = "recorder_hd_tests.rs"]
mod hd;

#[test]
fn mp4_is_playable_mirrored_and_tracks_wall_clock() {
    if Command::new("ffmpeg").arg("-version").output().is_err()
        || Command::new("ffprobe").arg("-version").output().is_err()
    {
        eprintln!("Skipping MP4 integration test: ffmpeg/ffprobe unavailable");
        return;
    }
    let output = tempfile::tempdir().unwrap();
    let frame = Arc::new(RgbaImage::from_fn(64, 48, |x, _| {
        if x < 32 {
            image::Rgba([0, 0, 255, 255])
        } else {
            image::Rgba([255, 0, 0, 255])
        }
    }));
    let (recorder, completion) =
        Recorder::start(output.path(), frame.clone(), true, None, 10).unwrap();
    // A static camera still produces frames covering elapsed recording time.
    std::thread::sleep(Duration::from_millis(650));
    recorder.update_frame(frame);
    recorder.stop();
    let path = completion.recv_blocking().unwrap().unwrap();
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,width,height,duration",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        probe.status.success(),
        "{}",
        String::from_utf8_lossy(&probe.stderr)
    );
    let metadata = String::from_utf8(probe.stdout).unwrap();
    assert!(metadata.contains("codec_name=h264"), "{metadata}");
    assert!(
        metadata.contains("width=64") && metadata.contains("height=48"),
        "{metadata}"
    );
    let duration: f64 = metadata
        .lines()
        .find_map(|line| line.strip_prefix("duration="))
        .unwrap()
        .parse()
        .unwrap();
    assert!((0.5..1.5).contains(&duration), "duration={duration}");
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args([
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(decoded.status.success());
    assert!(
        decoded.stdout[2] > 200 && decoded.stdout[0] < 40,
        "Left side should be mirrored blue"
    );
    assert_eq!(std::fs::read_dir(output.path()).unwrap().count(), 1);
}

#[test]
fn invalid_microphone_reports_failure_without_publishing() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("Skipping microphone failure test: ffmpeg unavailable");
        return;
    }
    let output = tempfile::tempdir().unwrap();
    let frame = Arc::new(RgbaImage::new(64, 48));
    let result = Recorder::start(
        output.path(),
        frame,
        false,
        Some("khamura-nonexistent-test-source".into()),
        10,
    );
    if let Ok((recorder, completion)) = result {
        recorder.stop();
        assert!(completion.recv_blocking().unwrap().is_err());
    }
    assert_eq!(std::fs::read_dir(output.path()).unwrap().count(), 0);
}
