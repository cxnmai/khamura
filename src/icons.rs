use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

// These SVGs are Lucide icons, kept in one set for consistent stroke styling.
pub const APERTURE: &str = "icons/aperture.svg";
pub const VIDEO: &str = "icons/video.svg";
pub const CIRCLE_STOP: &str = "icons/circle-stop.svg";
pub const SLIDERS_HORIZONTAL: &str = "icons/sliders-horizontal.svg";

pub struct LucideAssets;

impl AssetSource for LucideAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            APERTURE => Some(include_bytes!("../assets/icons/aperture.svg")),
            VIDEO => Some(include_bytes!("../assets/icons/video.svg")),
            CIRCLE_STOP => Some(include_bytes!("../assets/icons/circle-stop.svg")),
            SLIDERS_HORIZONTAL => Some(include_bytes!("../assets/icons/sliders-horizontal.svg")),
            _ => None,
        };

        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
