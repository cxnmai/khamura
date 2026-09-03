use gpui::{Context, IntoElement, Render, Window, div, prelude::*};

pub struct Camera;

impl Render for Camera {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
