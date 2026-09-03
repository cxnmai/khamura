use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

// These SVGs are Lucide icons, kept in one set for consistent stroke styling.
pub const APERTURE: &str = "icons/aperture.svg";
pub const VIDEO: &str = "icons/video.svg";
pub const CIRCLE_STOP: &str = "icons/circle-stop.svg";
pub const MAXIMIZE_2: &str = "icons/maximize-2.svg";
pub const MINIMIZE_2: &str = "icons/minimize-2.svg";

pub struct LucideAssets;

impl AssetSource for LucideAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            APERTURE => Some(include_bytes!("../assets/icons/aperture.svg")),
            VIDEO => Some(include_bytes!("../assets/icons/video.svg")),
            CIRCLE_STOP => Some(include_bytes!("../assets/icons/circle-stop.svg")),
            MAXIMIZE_2 => Some(include_bytes!("../assets/icons/maximize-2.svg")),
            MINIMIZE_2 => Some(include_bytes!("../assets/icons/minimize-2.svg")),
            _ => None,
        };

        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
