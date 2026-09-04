use super::*;

#[test]
fn decodes_silent_mp4_and_stops_worker() {
    if runtime_tools::command(Tool::Ffmpeg)
        .arg("-version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("silent.mp4");
    let status = runtime_tools::command(Tool::Ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x48:r=24",
            "-t",
            "0.3",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    let (tx, rx) = async_channel::bounded(2);
    let state = Playback::start(path.clone(), tx);
    let mut frames = 0;
    while let Ok(event) = rx.recv_blocking() {
        match event {
            Event::Frame(pixels) => {
                assert_eq!(pixels.len(), (WIDTH * HEIGHT * 4) as usize);
                frames += 1;
            }
            Event::End => break,
            Event::Error(error) => panic!("{error}"),
        }
    }
    assert!(frames > 0);
    state.stop();
    let (tx, rx) = async_channel::bounded(2);
    let state = Playback::start(path, tx);
    state.stop();
    assert!(
        rx.recv_blocking().is_err(),
        "stopped worker must close its channel"
    );
}
