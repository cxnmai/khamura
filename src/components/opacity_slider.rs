use std::{cell::Cell, rc::Rc};

use gpui::{
    Bounds, Context, IntoElement, MouseButton, MouseMoveEvent, Pixels, Render, Window, canvas, div,
    prelude::*, px, relative,
};

use crate::session_settings::SessionSettings;

/// A session-only opacity control. Dragging remains active beyond the track bounds.
pub struct OpacitySlider {
    bounds: Rc<Cell<Bounds<Pixels>>>,
    dragging: bool,
}

impl OpacitySlider {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            bounds: Rc::new(Cell::new(Bounds::default())),
            dragging: false,
        }
    }

    fn set_position(&self, x: Pixels, cx: &mut Context<Self>) {
        let bounds = self.bounds.get();
        if bounds.size.width <= px(0.) {
            return;
        }
        let value = ((x - bounds.origin.x) / bounds.size.width).clamp(0., 1.);
        cx.update_global::<SessionSettings, _>(|settings, _| {
            settings.background_opacity = value;
        });
        cx.notify();
    }
}

impl Render for OpacitySlider {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<SessionSettings>();
        let opacity = settings.background_opacity;
        let color = settings.theme_color;
        let ink = super::settings::foreground(color);
        let bounds = self.bounds.clone();
        let entity = cx.entity().downgrade();
        div()
            .id("opacity-slider")
            .relative()
            .w_full()
            .h(px(28.))
            .px(px(6.))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.dragging = true;
                    this.set_position(event.position.x, cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = false;
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = false;
                }),
            )
            .child(
                div()
                    .relative()
                    .size_full()
                    .child(
                        canvas(
                            move |new_bounds, _, _| bounds.set(new_bounds),
                            move |_, _, window, _| {
                                let entity = entity.clone();
                                window.on_mouse_event(
                                    move |event: &MouseMoveEvent, phase, _, cx| {
                                        if !phase.bubble() {
                                            return;
                                        }
                                        let _ = entity.update(cx, |this, cx| {
                                            if this.dragging {
                                                if event.pressed_button == Some(MouseButton::Left) {
                                                    this.set_position(event.position.x, cx);
                                                } else {
                                                    this.dragging = false;
                                                }
                                            }
                                        });
                                    },
                                );
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(12.))
                            .w_full()
                            .h(px(4.))
                            .rounded_full()
                            .bg(ink.opacity(0.2)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(12.))
                            .w(relative(opacity))
                            .h(px(4.))
                            .rounded_full()
                            .bg(ink),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .left(relative(opacity))
                            .ml(px(-6.))
                            .size(px(12.))
                            .rounded_full()
                            .bg(ink),
                    ),
            )
    }
}
