use super::*;

#[test]
fn full_hd_recording_stops_promptly_without_losing_duration() {
    if Command::new("ffmpeg").arg("-version").output().is_err()
        || Command::new("ffprobe").arg("-version").output().is_err()
    {
        eprintln!("Skipping full-HD integration test: ffmpeg/ffprobe unavailable");
        return;
    }
    let output = tempfile::tempdir().unwrap();
    // Full-size raw frames expose pipe backpressure that thumbnail tests miss.
    // Simple blocks keep encoding CPU costs modest and avoid accessing any device.
    let frame = Arc::new(RgbaImage::from_fn(1920, 1080, |x, y| {
        image::Rgba([
            ((x / 120) * 16) as u8,
            ((y / 120) * 28) as u8,
            128,
            255,
        ])
    }));
    let (recorder, completion) =
        Recorder::start(output.path(), frame, false, None, 30).unwrap();
    std::thread::sleep(Duration::from_secs(2));
    recorder.stop();
    let deadline = Instant::now() + Duration::from_secs(8);
    let path = loop {
        match completion.try_recv() {
            Ok(result) => break result.expect("Full-HD recording must finalize successfully"),
            Err(async_channel::TryRecvError::Empty) => {
                assert!(Instant::now() < deadline, "Full-HD recording stop stalled");
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(error) => panic!("Recording worker disconnected: {error}"),
        }
    };
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,width,height,duration",
            "-of",
            "json",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(probe.status.success(), "{:?}", probe.stderr);
    let metadata: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    let stream = &metadata["streams"][0];
    assert_eq!(stream["codec_name"], "h264");
    assert_eq!(stream["width"], 1920);
    assert_eq!(stream["height"], 1080);
    let duration: f64 = stream["duration"].as_str().unwrap().parse().unwrap();
    assert!((1.9..2.5).contains(&duration), "Unexpected duration: {duration}");
    // Decode the entire recording, not merely its container header or first frame.
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-xerror", "-i"])
        .arg(&path)
        .args(["-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    assert_eq!(std::fs::read_dir(output.path()).unwrap().count(), 1);
}
