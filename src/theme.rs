use gpui::Rgba;

/// The base RGB color used for the camera's letterbox bars.
///
/// This is the RGB equivalent of HSL(199.3, 94.8%, 22.5%).
pub const CAMERA_LETTERBOX_COLOR: Rgba = Rgba {
    r: 0.0117,
    g: 0.301077,
    b: 0.4383,
    a: 1.0,
};

/// The opacity applied to the camera's letterbox bars.
pub const CAMERA_LETTERBOX_OPACITY: f32 = 0.7;
