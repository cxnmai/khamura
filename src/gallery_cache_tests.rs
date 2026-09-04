use super::*;
use std::cell::Cell;

#[test]
fn unchanged_media_reuses_thumbnails_and_failed_decodes() {
    let directory = tempfile::tempdir().unwrap();
    let photo = directory.path().join("photo.png");
    let video = directory.path().join("video.mp4");
    std::fs::write(&photo, b"photo").unwrap();
    std::fs::write(&video, b"video").unwrap();
    std::fs::write(directory.path().join(".khamura-pending.mp4"), b"partial").unwrap();
    let calls = Cell::new(0);
    let extract = |_: &Path, is_video| {
        calls.set(calls.get() + 1);
        if is_video {
            None // Failed decodes must not repeatedly launch FFmpeg either.
        } else {
            Some(Arc::new(RenderImage::new(vec![Frame::new(
                image::RgbaImage::new(1, 1),
            )])))
        }
    };
    let first = scan_with(directory.path(), &[], extract).unwrap();
    assert_eq!(calls.get(), 2);
    let second = scan_with(directory.path(), &first, extract).unwrap();
    assert_eq!(calls.get(), 2);
    let original = first.iter().find(|item| item.path == photo).unwrap();
    let reused = second.iter().find(|item| item.path == photo).unwrap();
    assert!(Arc::ptr_eq(
        original.thumbnail.as_ref().unwrap(),
        reused.thumbnail.as_ref().unwrap(),
    ));

    let added = directory.path().join("new.mp4");
    std::fs::write(&added, b"new video").unwrap();
    let third = scan_with(directory.path(), &second, extract).unwrap();
    assert_eq!(calls.get(), 3, "only new media should be decoded");
    std::fs::write(&video, b"replacement video").unwrap();
    let fourth = scan_with(directory.path(), &third, extract).unwrap();
    assert_eq!(calls.get(), 4, "changed media should be decoded again");
    std::fs::remove_file(&photo).unwrap();
    let fifth = scan_with(directory.path(), &fourth, extract).unwrap();
    assert_eq!(calls.get(), 4);
    assert_eq!(fifth.len(), 2);
    assert!(fifth.iter().all(|item| item.path != photo));
}
