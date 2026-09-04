use super::*;

#[path = "gallery_tile.rs"]
mod tile;

pub(super) fn grid(gallery: &Gallery, window: &Window, cx: &Context<Gallery>) -> AnyElement {
    let ink = super::super::super::settings::foreground(cx.global::<SessionSettings>().theme_color);
    let store = cx.global::<GalleryStore>();
    let width = (f32::from(window.viewport_size().width) - 40.).max(1.);
    let columns = ((width + 8.) / 168.).floor().max(1.) as usize;
    let edge = ((width - (columns - 1) as f32 * 8.) / columns as f32).max(1.);
    let rows = store.items.len().div_ceil(columns);
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .px(px(20.))
        .when_some(store.error.clone(), |view, error| {
            view.child(
                div()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(ink.opacity(0.5))
                    .pb(px(12.))
                    .child(error),
            )
        })
        .child(
            gpui::uniform_list(
                "gallery-scroll",
                rows,
                cx.processor(move |_, range: std::ops::Range<usize>, _, cx| {
                    let items = &cx.global::<GalleryStore>().items;
                    range
                        .map(|row| {
                            let start = (row * columns).min(items.len());
                            let end = (start + columns).min(items.len());
                            div().h(px(edge + 8.)).flex().gap(px(8.)).children(
                                items[start..end].iter().enumerate().map(|(offset, item)| {
                                    tile::tile(item, start + offset, edge, ink, cx)
                                }),
                            )
                        })
                        .collect()
                }),
            )
            .track_scroll(gallery.grid_scroll.clone())
            .flex_1()
            .min_h_0()
            // Each row already contributes 8px below its tiles: preserve 20px total.
            .pb(px(12.)),
        )
        .into_any_element()
}
