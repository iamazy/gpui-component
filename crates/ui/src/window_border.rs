// From:
// https://github.com/zed-industries/zed/blob/56daba28d40301ee4c05546fadb691d070b7b2b6/crates/gpui/examples/window_shadow.rs
use gpui::{
    AnyElement, App, Bounds, CursorStyle, Decorations, Edges, HitboxBehavior, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce,
    ResizeEdge, Size, Styled as _, Window, canvas, div, point, prelude::FluentBuilder as _, px,
};

use crate::ActiveTheme;

#[cfg(not(target_os = "linux"))]
const SHADOW_SIZE: Pixels = px(0.0);
#[cfg(target_os = "linux")]
const SHADOW_SIZE: Pixels = px(12.0);
const BORDER_SIZE: Pixels = px(1.0);
pub(crate) const BORDER_RADIUS: Pixels = px(0.0);

/// Create a new window border.
pub fn window_border() -> WindowBorder {
    WindowBorder::new()
}

/// Window border use to render a custom window border and shadow for Linux.
#[derive(IntoElement, Default)]
pub struct WindowBorder {
    children: Vec<AnyElement>,
}

impl WindowBorder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

/// Get the window paddings.
pub fn window_paddings(window: &Window) -> Edges<Pixels> {
    match window.window_decorations() {
        Decorations::Server => Edges::all(px(0.0)),
        Decorations::Client { tiling } => {
            let mut paddings = Edges::all(SHADOW_SIZE);
            if tiling.top {
                paddings.top = px(0.0);
            }
            if tiling.bottom {
                paddings.bottom = px(0.0);
            }
            if tiling.left {
                paddings.left = px(0.0);
            }
            if tiling.right {
                paddings.right = px(0.0);
            }
            paddings
        }
    }
}

impl ParentElement for WindowBorder {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for WindowBorder {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let decorations = window.window_decorations();
        window.set_client_inset(SHADOW_SIZE);

        div()
            .id("window-backdrop")
            .bg(gpui::transparent_black())
            .map(|div| match decorations {
                Decorations::Server => div,
                Decorations::Client { tiling, .. } => div
                    .bg(gpui::transparent_black())
                    .child(
                        canvas(
                            |_bounds, window, _| {
                                window.insert_hitbox(
                                    Bounds::new(
                                        point(px(0.0), px(0.0)),
                                        window.window_bounds().get_bounds().size,
                                    ),
                                    HitboxBehavior::Normal,
                                )
                            },
                            move |_bounds, hitbox, window, _| {
                                let mouse = window.mouse_position();
                                let size = window.window_bounds().get_bounds().size;
                                let Some(edge) = resize_edge(mouse, SHADOW_SIZE, size, tiling) else {
                                    return;
                                };
                                window.set_cursor_style(
                                    match edge {
                                        ResizeEdge::Top | ResizeEdge::Bottom => {
                                            CursorStyle::ResizeUpDown
                                        }
                                        ResizeEdge::Left | ResizeEdge::Right => {
                                            CursorStyle::ResizeLeftRight
                                        }
                                        ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                            CursorStyle::ResizeUpLeftDownRight
                                        }
                                        ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                            CursorStyle::ResizeUpRightDownLeft
                                        }
                                    },
                                    &hitbox,
                                );
                            },
                        )
                        .size_full()
                        .absolute(),
                    )
                    .when(!(tiling.top || tiling.right), |div| {
                        div.rounded_tr(BORDER_RADIUS)
                    })
                    .when(!(tiling.top || tiling.left), |div| {
                        div.rounded_tl(BORDER_RADIUS)
                    })
                    .when(!tiling.top, |div| div.pt(SHADOW_SIZE))
                    .when(!tiling.bottom, |div| div.pb(SHADOW_SIZE))
                    .when(!tiling.left, |div| div.pl(SHADOW_SIZE))
                    .when(!tiling.right, |div| div.pr(SHADOW_SIZE))
                    .on_mouse_down(MouseButton::Left, move |_, window, _| {
                        let size = window.window_bounds().get_bounds().size;
                        let pos = window.mouse_position();

                        match resize_edge(pos, SHADOW_SIZE, size, tiling) {
                            Some(edge) => window.start_window_resize(edge),
                            None => {}
                        };
                    }),
            })
            .size_full()
            .child(
                div()
                    .cursor(CursorStyle::default())
                    .map(|div| match decorations {
                        Decorations::Server => div,
                        Decorations::Client { tiling } => div
                            .when(!(tiling.top || tiling.right), |div| {
                                div.rounded_tr(BORDER_RADIUS)
                            })
                            .when(!(tiling.top || tiling.left), |div| {
                                div.rounded_tl(BORDER_RADIUS)
                            })
                            .border_color(cx.theme().window_border)
                            .when(!tiling.top, |div| div.border_t(BORDER_SIZE))
                            .when(!tiling.bottom, |div| div.border_b(BORDER_SIZE))
                            .when(!tiling.left, |div| div.border_l(BORDER_SIZE))
                            .when(!tiling.right, |div| div.border_r(BORDER_SIZE))
                            .when(!tiling.is_tiled(), |div| {
                                div.shadow(vec![gpui::BoxShadow {
                                    color: Hsla {
                                        h: 0.,
                                        s: 0.,
                                        l: 0.,
                                        a: 0.3,
                                    },
                                    blur_radius: SHADOW_SIZE / 2.,
                                    spread_radius: px(0.),
                                    offset: point(px(0.0), px(0.0)),
                                }])
                            }),
                    })
                    .on_mouse_move(|_e, _, cx| {
                        cx.stop_propagation();
                    })
                    .bg(gpui::transparent_black())
                    .size_full()
                    .children(self.children),
            )
    }
}

fn resize_edge(
    pos: Point<Pixels>,
    shadow_size: Pixels,
    size: Size<Pixels>,
    tiling: gpui::Tiling,
) -> Option<ResizeEdge> {
    let near_top = pos.y < shadow_size && !tiling.top;
    let near_bottom = pos.y > size.height - shadow_size && !tiling.bottom;
    let near_left = pos.x < shadow_size && !tiling.left;
    let near_right = pos.x > size.width - shadow_size && !tiling.right;

    if near_top && near_left {
        Some(ResizeEdge::TopLeft)
    } else if near_top && near_right {
        Some(ResizeEdge::TopRight)
    } else if near_top {
        Some(ResizeEdge::Top)
    } else if near_bottom && near_left {
        Some(ResizeEdge::BottomLeft)
    } else if near_bottom && near_right {
        Some(ResizeEdge::BottomRight)
    } else if near_bottom {
        Some(ResizeEdge::Bottom)
    } else if near_left {
        Some(ResizeEdge::Left)
    } else if near_right {
        Some(ResizeEdge::Right)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn resize_edge_is_disabled_when_fully_tiled() {
        let size = size(px(800.0), px(600.0));
        let pos = point(px(799.0), px(599.0));
        assert_eq!(resize_edge(pos, px(12.0), size, gpui::Tiling::tiled()), None);
    }

    #[test]
    fn resize_edge_ignores_tiled_right_edge() {
        let size = size(px(800.0), px(600.0));
        let pos = point(px(799.0), px(599.0));
        let tiling = gpui::Tiling {
            right: true,
            ..Default::default()
        };

        assert_eq!(resize_edge(pos, px(12.0), size, tiling), Some(ResizeEdge::Bottom));
    }

    #[test]
    fn resize_edge_ignores_tiled_bottom_edge() {
        let size = size(px(800.0), px(600.0));
        let pos = point(px(799.0), px(599.0));
        let tiling = gpui::Tiling {
            bottom: true,
            ..Default::default()
        };

        assert_eq!(resize_edge(pos, px(12.0), size, tiling), Some(ResizeEdge::Right));
    }
}
