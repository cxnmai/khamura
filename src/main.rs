mod components;
mod capture_settings;
mod device_catalog;
mod media;
mod config;
mod config_location;
mod config_store;
mod icons;
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
