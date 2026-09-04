mod capture_settings;
mod components;
mod config;
mod config_location;
mod config_store;
mod device_catalog;
mod gallery;
mod icons;
mod media;
mod runtime_tools;
mod session_settings;
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
            cx.set_global(session_settings::SessionSettings::from(&config));
            cx.set_global(capture_settings::CaptureSettings::from(&config));
            cx.set_global(config);
            gallery::GalleryStore::init(cx);
            cx.open_window(
                WindowOptions {
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| Camera::new(window, cx)),
            )
            .expect("failed to open window");
        });
}
