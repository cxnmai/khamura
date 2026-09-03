use gpui::{App, AppContext, Application, EmptyView, WindowOptions};

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| EmptyView))
            .expect("failed to open window");
    });
}
