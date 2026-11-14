/// A renderer-neutral viewport: `screen = content * zoom + pan`. `pan_*` is
/// in logical screen pixels, while geometry stays in content coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}

/// Screen-space handle / hit radius.
pub const HANDLE_HIT_PX: f64 = 14.0;

impl Viewport {
    pub fn clamped_zoom(self) -> Self {
        Self {
            zoom: self.zoom.clamp(0.05, 32.0),
            ..self
        }
    }

    pub fn to_screen(self, point: [f64; 2]) -> [f64; 2] {
        [
            point[0] * self.zoom + self.pan_x,
            point[1] * self.zoom + self.pan_y,
        ]
    }

    pub fn to_content(self, point: [f64; 2]) -> [f64; 2] {
        let zoom = if self.zoom == 0.0 { 1.0 } else { self.zoom };
        [
            (point[0] - self.pan_x) / zoom,
            (point[1] - self.pan_y) / zoom,
        ]
    }

    pub fn panned_by(self, dx: f64, dy: f64) -> Self {
        Self {
            pan_x: self.pan_x + dx,
            pan_y: self.pan_y + dy,
            ..self
        }
    }

    /// Zoom by `factor` keeping the content under `cursor_screen` fixed.
    pub fn zoomed_at(self, cursor_screen: [f64; 2], factor: f64) -> Self {
        if factor <= 0.0 || !factor.is_finite() {
            return self;
        }
        let content = self.to_content(cursor_screen);
        let next = Self {
            zoom: (self.zoom * factor).clamp(0.05, 32.0),
            pan_x: self.pan_x,
            pan_y: self.pan_y,
        };
        let after = next.to_screen(content);
        Self {
            zoom: next.zoom,
            pan_x: next.pan_x + cursor_screen[0] - after[0],
            pan_y: next.pan_y + cursor_screen[1] - after[1],
        }
    }

    pub fn fit() -> Self {
        Self::default()
    }

    /// Fit `content` into `layout`, centred, preserving aspect ratio.
    pub fn contain(
        content_width: f64,
        content_height: f64,
        layout_width: f64,
        layout_height: f64,
    ) -> Self {
        if content_width <= 0.0
            || content_height <= 0.0
            || layout_width <= 0.0
            || layout_height <= 0.0
        {
            return Self::default();
        }
        let zoom = (layout_width / content_width).min(layout_height / content_height);
        Self {
            zoom,
            pan_x: (layout_width - content_width * zoom) / 2.0,
            pan_y: (layout_height - content_height * zoom) / 2.0,
        }
    }

    /// [`HANDLE_HIT_PX`] expressed in content units.
    pub fn hit_tolerance(self) -> f64 {
        HANDLE_HIT_PX / self.clamped_zoom().zoom
    }
}

/// Map a client (CSS pixel) point into the content coordinates of an SVG
/// whose `view_box` (`[min_x, min_y, width, height]`) is drawn into `rect`
/// (`[left, top, width, height]`) with `preserveAspectRatio="xMidYMid meet"`
/// and whose content sits in a `translate(pan) scale(zoom)` group (`inner`).
pub fn svg_meet_to_content(
    client: [f64; 2],
    rect: [f64; 4],
    view_box: [f64; 4],
    inner: Viewport,
) -> Option<[f64; 2]> {
    let [left, top, width, height] = rect;
    let [min_x, min_y, vb_width, vb_height] = view_box;
    if vb_width <= 0.0 || vb_height <= 0.0 || width <= 0.0 || height <= 0.0 || inner.zoom <= 0.0 {
        return None;
    }
    let [x, y] = Viewport::contain(vb_width, vb_height, width, height)
        .to_content([client[0] - left, client[1] - top]);
    Some(inner.to_content([x + min_x, y + min_y]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_points_map_through_meet_fit_pan_and_zoom() {
        // 100×100 view box drawn into a 400×200 rect: scale 2, letterboxed by 100px.
        let rect = [10.0, 20.0, 400.0, 200.0];
        let view_box = [0.0, 0.0, 100.0, 100.0];
        let identity = Viewport::default();
        assert_eq!(
            svg_meet_to_content([110.0, 20.0], rect, view_box, identity),
            Some([0.0, 0.0])
        );
        assert_eq!(
            svg_meet_to_content([310.0, 220.0], rect, view_box, identity),
            Some([100.0, 100.0])
        );
        // Group transform: translate(10 0) scale(2).
        let inner = Viewport {
            zoom: 2.0,
            pan_x: 10.0,
            pan_y: 0.0,
        };
        assert_eq!(
            svg_meet_to_content([150.0, 60.0], rect, view_box, inner),
            Some([5.0, 10.0])
        );
        assert_eq!(
            svg_meet_to_content([0.0, 0.0], [0.0, 0.0, 0.0, 10.0], view_box, identity),
            None
        );
    }

    #[test]
    fn zoom_keeps_the_cursor_anchored() {
        let viewport = Viewport {
            zoom: 1.5,
            pan_x: 20.0,
            pan_y: -5.0,
        };
        let cursor = [120.0, 80.0];
        let content = viewport.to_content(cursor);
        let zoomed = viewport.zoomed_at(cursor, 2.0);
        let back = zoomed.to_screen(content);
        assert!((back[0] - cursor[0]).abs() < 1e-9 && (back[1] - cursor[1]).abs() < 1e-9);
    }
}
