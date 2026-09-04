use crate::capture_settings::{CaptureSettings, PhotoAspect, VideoQuality};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Menu { Timer, Aspect, Quality }
#[derive(Clone, Copy)]
pub(super) enum Choice { Timer(u64), Aspect(PhotoAspect), Quality(Option<VideoQuality>) }
impl Choice {
    pub fn apply(self, settings: &mut CaptureSettings) {
        match self {
            Self::Timer(value) => settings.timer_seconds = value,
            Self::Aspect(value) => settings.aspect = value,
            Self::Quality(value) => settings.quality = value,
        }
    }
}
pub(super) fn aspect_label(aspect: PhotoAspect) -> &'static str {
    match aspect {
        PhotoAspect::Native => "Native", PhotoAspect::FourThree => "4:3",
        PhotoAspect::SixteenNine => "16:9", PhotoAspect::Square => "1:1",
    }
}
pub(super) fn quality_label(quality: Option<VideoQuality>) -> String {
    quality.map(|q| format!("{}×{} · {} fps", q.width, q.height, q.fps)).unwrap_or("Native".into())
}
pub(super) fn choices(menu: Menu, settings: &CaptureSettings) -> Vec<(String, Choice, bool)> {
    match menu {
        Menu::Timer => [0, 3, 10].into_iter().map(|n| (if n == 0 { "Off".into() } else { format!("{n} seconds") }, Choice::Timer(n), settings.timer_seconds == n)).collect(),
        Menu::Aspect => [PhotoAspect::Native, PhotoAspect::FourThree, PhotoAspect::SixteenNine, PhotoAspect::Square].into_iter().map(|a| (aspect_label(a).into(), Choice::Aspect(a), settings.aspect == a)).collect(),
        Menu::Quality => std::iter::once(None).chain(settings.qualities.iter().copied().map(Some)).map(|q| (quality_label(q), Choice::Quality(q), settings.quality == q)).collect(),
    }
}
