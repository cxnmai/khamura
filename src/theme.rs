use gpui::Rgba;

/// An RGB color with channels represented in the usual 0..=255 range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to GPUI's normalized color format and apply alpha.
    pub fn to_gpui(self, alpha: f32) -> Rgba {
        Rgba {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: alpha.clamp(0.0, 1.0),
        }
    }
}

/// The base RGB color used for the camera's letterbox bars.
pub const CAMERA_LETTERBOX_COLOR: Rgb = Rgb::new(0, 0, 0);

/// The opacity applied to the camera's letterbox bars.
pub const CAMERA_LETTERBOX_OPACITY: f32 = 0.7;
