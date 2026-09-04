#[path = "gallery_video.rs"]
mod video;

use crate::config::Config;
use gpui::{App, BorrowAppContext, RenderImage};
use image::{Frame, ImageReader};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

#[derive(Clone)]
pub struct GalleryItem {
    pub path: PathBuf,
    pub is_video: bool,
    pub thumbnail: Option<Arc<RenderImage>>,
    fingerprint: (u64, Option<SystemTime>),
}

#[derive(Default)]
pub struct GalleryStore {
    pub items: Vec<GalleryItem>,
    pub loading: bool,
    pub error: Option<String>,
    refresh_pending: bool,
    directory: Option<PathBuf>,
}

impl gpui::Global for GalleryStore {}

impl GalleryStore {
    pub fn init(cx: &mut App) {
        cx.set_global(Self::default());
        Self::refresh(cx);
    }

    fn clear_images(&mut self, cx: &mut App) {
        for item in self.items.drain(..) {
            if let Some(image) = item.thumbnail {
                cx.drop_image(image, None);
            }
        }
    }

    pub fn refresh(cx: &mut App) {
        let directory = cx.global::<Config>().photo_directory.clone();
        // Coalesce saves/device changes while a thumbnail scan is running. Reuse its
        // completed work in the follow-up scan instead of starting duplicate decoders.
        let previous = cx.update_global::<Self, _>(|store, cx| {
            if store.loading {
                store.refresh_pending = true;
                return None;
            }
            if store.directory.as_ref() != Some(&directory) {
                store.clear_images(cx);
                store.directory = Some(directory.clone());
            }
            store.loading = true;
            store.error = None;
            Some(store.items.clone())
        });
        let Some(previous) = previous else { return };
        let scanned_directory = directory.clone();
        let query = cx
            .background_executor()
            .spawn(async move { scan(&scanned_directory, &previous) });
        cx.spawn(async move |cx| {
            let result = query.await;
            let _ = cx.update(|cx| {
                let changed_directory = cx.global::<Config>().photo_directory != directory;
                let refresh_again = cx.update_global::<Self, _>(|store, cx| {
                    store.loading = false;
                    if !changed_directory {
                        match result {
                            Ok(items) => {
                                // Do not evict unchanged thumbnails from GPUI's GPU cache.
                                let retained: HashSet<_> = items
                                    .iter()
                                    .filter_map(|item| item.thumbnail.as_ref().map(Arc::as_ptr))
                                    .collect();
                                for old in &store.items {
                                    if let Some(image) = &old.thumbnail {
                                        if !retained.contains(&Arc::as_ptr(image)) {
                                            cx.drop_image(image.clone(), None);
                                        }
                                    }
                                }
                                store.items = items;
                            }
                            Err(error) => store.error = Some(error),
                        }
                    }
                    std::mem::take(&mut store.refresh_pending) || changed_directory
                });
                if refresh_again {
                    Self::refresh(cx);
                }
            });
        })
        .detach();
    }
}

fn scan(directory: &Path, previous: &[GalleryItem]) -> Result<Vec<GalleryItem>, String> {
    scan_with(directory, previous, thumbnail)
}

fn scan_with(
    directory: &Path,
    previous: &[GalleryItem],
    mut extract: impl FnMut(&Path, bool) -> Option<Arc<RenderImage>>,
) -> Result<Vec<GalleryItem>, String> {
    let cached: HashMap<_, _> = previous.iter().map(|item| (&item.path, item)).collect();
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
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("mp4"))
        {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("Could not read media: {error}"))?;
        if metadata.is_file() {
            paths.push((
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                path,
                (metadata.len(), metadata.modified().ok()),
            ));
        }
    }
    paths.sort_by(|a, b| b.cmp(a));
    Ok(paths
        .into_iter()
        .map(|(_, path, fingerprint)| {
            if let Some(item) = cached.get(&path)
                && item.fingerprint == fingerprint
                && fingerprint.1.is_some()
            {
                return (*item).clone();
            }
            let is_video = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("mp4"));
            GalleryItem {
                thumbnail: extract(&path, is_video),
                fingerprint,
                path,
                is_video,
            }
        })
        .collect())
}

fn thumbnail(path: &Path, is_video: bool) -> Option<Arc<RenderImage>> {
    // Decode only one source at a time; retain bounded thumbnails, never originals.
    let image = if is_video {
        video::thumbnail(path)?
    } else {
        ImageReader::open(path).ok()?.decode().ok()?
    };
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
        let video = directory.path().join("video.MP4");
        std::fs::write(&video, b"video").unwrap();
        std::fs::File::open(&video)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(200))
            .unwrap();
        std::fs::write(directory.path().join("ignored.txt"), b"text").unwrap();
        std::fs::create_dir(directory.path().join("folder.png")).unwrap();
        let photos = scan(directory.path(), &[]).unwrap();
        assert_eq!(photos.len(), 3);
        assert_eq!(photos[0].path, newer);
        assert!(photos[0].thumbnail.is_none());
        assert_eq!(photos[1].path, video);
        assert!(photos[1].is_video);
        assert!(photos[1].thumbnail.is_none());
        assert_eq!(photos[2].path, older);
        assert!(!photos[2].is_video);
        let thumbnail = photos[2].thumbnail.as_ref().unwrap();
        assert_eq!(thumbnail.size(0).width.0, 240);
        assert_eq!(thumbnail.size(0).height.0, 120);
        assert_eq!(&thumbnail.as_bytes(0).unwrap()[..4], &[0, 0, 255, 255]);
        assert!(
            scan(&directory.path().join("missing"), &[])
                .unwrap()
                .is_empty()
        );
    }
}
