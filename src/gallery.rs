use crate::config::Config;
use gpui::{App, BorrowAppContext, RenderImage};
use image::{Frame, ImageReader};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

#[derive(Clone)]
pub struct GalleryItem {
    pub path: PathBuf,
    pub thumbnail: Option<Arc<RenderImage>>,
}

#[derive(Default)]
pub struct GalleryStore {
    pub items: Vec<GalleryItem>,
    pub loading: bool,
    pub error: Option<String>,
    revision: u64,
}

impl gpui::Global for GalleryStore {}

impl GalleryStore {
    pub fn init(cx: &mut App) {
        cx.set_global(Self::default());
        Self::refresh(cx);
    }

    pub fn refresh(cx: &mut App) {
        let directory = cx.global::<Config>().photo_directory.clone();
        let revision = cx.update_global::<Self, _>(|store, _| {
            store.revision += 1;
            store.loading = true;
            store.error = None;
            store.revision
        });
        let query = cx
            .background_executor()
            .spawn(async move { scan(&directory) });
        cx.spawn(async move |cx| {
            let result = query.await;
            let _ = cx.update(|cx| {
                cx.update_global::<Self, _>(|store, _| {
                    if revision != store.revision {
                        return;
                    }
                    store.loading = false;
                    match result {
                        Ok(items) => store.items = items,
                        Err(error) => {
                            store.items.clear();
                            store.error = Some(error);
                        }
                    }
                });
            });
        })
        .detach();
    }
}

fn scan(directory: &Path) -> Result<Vec<GalleryItem>, String> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("Could not read gallery: {error}")),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("Could not read gallery entry: {error}"))?;
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("Could not read photo: {error}"))?;
        if metadata.is_file() {
            paths.push((metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), path));
        }
    }
    paths.sort_by(|a, b| b.cmp(a));
    Ok(paths
        .into_iter()
        .map(|(_, path)| GalleryItem {
            thumbnail: thumbnail(&path),
            path,
        })
        .collect())
}

fn thumbnail(path: &Path) -> Option<Arc<RenderImage>> {
    // Decode only one source at a time; retain bounded thumbnails, never originals.
    let image = ImageReader::open(path).ok()?.decode().ok()?;
    let mut pixels = image.thumbnail(240, 240).into_rgba8();
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2); // GPUI RenderImage expects BGRA.
    }
    Some(Arc::new(RenderImage::new(vec![Frame::new(pixels)])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn scan_filters_files_orders_newest_first_and_keeps_corrupt_entries() {
        let directory = tempfile::tempdir().unwrap();
        let older = directory.path().join("older.png");
        let newer = directory.path().join("newer.PNG");
        RgbaImage::from_pixel(800, 400, Rgba([255, 0, 0, 255]))
            .save(&older)
            .unwrap();
        std::fs::write(&newer, b"corrupt photo").unwrap();
        std::fs::File::open(&older)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100))
            .unwrap();
        std::fs::write(directory.path().join("video.mp4"), b"video").unwrap();
        std::fs::create_dir(directory.path().join("folder.png")).unwrap();
        let photos = scan(directory.path()).unwrap();
        assert_eq!(photos.len(), 2);
        assert_eq!(photos[0].path, newer);
        assert!(photos[0].thumbnail.is_none());
        assert_eq!(photos[1].path, older);
        assert!(photos[1].thumbnail.is_some());
        assert!(scan(&directory.path().join("missing")).unwrap().is_empty());
    }
}
