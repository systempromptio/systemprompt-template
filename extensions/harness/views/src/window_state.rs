//! The main window's geometry: where it opens (the saved place when it is still
//! on a display) and what is saved when it moves.

use gpui_kit::{App, Bounds, Pixels, Size, WindowBounds, point, px, size};

use crate::app::settings::WindowGeometry;

const DEFAULT_WIDTH: f32 = 1240.0;
const DEFAULT_HEIGHT: f32 = 820.0;
const MIN_WIDTH: f32 = 760.0;
const MIN_HEIGHT: f32 = 520.0;

pub const fn min_size() -> Size<Pixels> {
    size(px(MIN_WIDTH), px(MIN_HEIGHT))
}

pub fn initial_bounds(saved: Option<WindowGeometry>, cx: &App) -> WindowBounds {
    let Some(saved) = saved.filter(WindowGeometry::is_sane) else {
        return WindowBounds::centered(size(px(DEFAULT_WIDTH), px(DEFAULT_HEIGHT)), cx);
    };
    let rect = Bounds::new(
        point(px(saved.x), px(saved.y)),
        size(px(saved.width), px(saved.height)),
    );
    let reachable = cx
        .displays()
        .iter()
        .any(|display| display.bounds().intersects(&rect));
    let rect = if reachable {
        rect
    } else {
        Bounds::centered(None, rect.size, cx)
    };
    if saved.maximized {
        WindowBounds::Maximized(rect)
    } else {
        WindowBounds::Windowed(rect)
    }
}

pub(crate) fn geometry(bounds: WindowBounds) -> WindowGeometry {
    let (rect, maximized) = match bounds {
        WindowBounds::Windowed(rect) => (rect, false),
        WindowBounds::Maximized(rect) | WindowBounds::Fullscreen(rect) => (rect, true),
    };
    WindowGeometry {
        x: f32::from(rect.origin.x),
        y: f32::from(rect.origin.y),
        width: f32::from(rect.size.width),
        height: f32::from(rect.size.height),
        maximized,
    }
}
