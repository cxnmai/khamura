mod components;
mod icons;
mod theme;

use components::camera::Camera;
use gpui::{App, AppContext, Application, WindowBackgroundAppearance, WindowOptions};

fn main() {
    Application::new()
        .with_assets(icons::LucideAssets)
        .run(|cx: &mut App| {
            cx.open_window(
                WindowOptions {
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |_, cx| cx.new(Camera::new),
            )
            .expect("failed to open window");
        });
}
