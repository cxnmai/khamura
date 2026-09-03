mod components;

use components::camera::Camera;
use gpui::{App, AppContext, Application, WindowOptions};

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Camera))
            .expect("failed to open window");
    });
}
