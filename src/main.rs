mod components;
mod config;
mod icons;
mod theme;

use components::camera::Camera;
use gpui::{App, AppContext, Application, WindowBackgroundAppearance, WindowOptions};

fn main() {
    let config = config::Config::load().unwrap_or_else(|error| {
        eprintln!("Configuration error: {error}");
        std::process::exit(1);
    });
    Application::new()
        .with_assets(icons::LucideAssets)
        .run(move |cx: &mut App| {
            cx.set_global(config);
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
