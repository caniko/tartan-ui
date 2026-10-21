/// The smallest ring an edit may leave behind.
pub const MIN_POLYGON_VERTICES: usize = 3;

/// An in-progress polygon edit. `points` is an open ring: the closing vertex
/// is implied. Hosts bind a draft to whatever revision it was opened on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolygonDraft {
    pub points: Vec<[f64; 2]>,
    pub selected: Option<usize>,
}

impl PolygonDraft {
    /// Start from an open or closed ring.
    pub fn from_ring(ring: &[[f64; 2]]) -> Self {
        let mut points = ring.to_vec();
        if points.len() > 1 && points.first() == points.last() {
            points.pop();
        }
        Self {
            points,
            selected: None,
        }
    }

    pub fn can_delete(&self) -> bool {
        self.selected.is_some() && self.points.len() > MIN_POLYGON_VERTICES
    }

    pub fn delete_selected(&mut self) {
        if let Some(index) = self.selected.filter(|_| self.can_delete()) {
            self.points.remove(index);
            self.selected = None;
        }
    }

    /// Insert the midpoint of edge `index` → `index + 1` and select it.
    pub fn insert_after(&mut self, index: usize) {
        let len = self.points.len();
        if index >= len {
            return;
        }
        let [ax, ay] = self.points[index];
        let [bx, by] = self.points[(index + 1) % len];
        self.points
            .insert(index + 1, [(ax + bx) / 2.0, (ay + by) / 2.0]);
        self.selected = Some(index + 1);
    }

    pub fn nudge(&mut self, dx: f64, dy: f64) {
        if let Some(point) = self.selected.and_then(|index| self.points.get_mut(index)) {
            point[0] += dx;
            point[1] += dy;
        }
    }

    pub fn set_selected_point(&mut self, point: [f64; 2]) {
        if let Some(target) = self.selected.and_then(|index| self.points.get_mut(index)) {
            *target = point;
        }
    }

    /// The ring with its closing vertex repeated, as GeoJSON and SVG expect.
    pub fn closed_points(&self) -> Vec<[f64; 2]> {
        let mut ring = self.points.clone();
        if let Some(first) = ring.first().copied() {
            ring.push(first);
        }
        ring
    }
}

/// Even-odd ray cast; `vertices` is an open or closed ring.
pub fn point_in_polygon(vertices: &[[f64; 2]], point: [f64; 2]) -> bool {
    if vertices.is_empty() {
        return false;
    }
    let [x, y] = point;
    let mut inside = false;
    let mut previous = vertices.len() - 1;
    for current in 0..vertices.len() {
        let [xi, yi] = vertices[current];
        let [xj, yj] = vertices[previous];
        let denom = yj - yi;
        let intersects = (yi > y) != (yj > y)
            && denom.abs() > f64::EPSILON
            && x < (xj - xi) * (y - yi) / denom + xi;
        if intersects {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

pub fn point_segment_distance(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> f64 {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f64::EPSILON {
        return (point[0] - start[0]).hypot(point[1] - start[1]);
    }
    let t = ((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    (point[0] - (start[0] + t * dx)).hypot(point[1] - (start[1] + t * dy))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [[f64; 2]; 5] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];

    #[test]
    fn draft_edits_an_open_ring() {
        let mut draft = PolygonDraft::from_ring(&SQUARE);
        assert_eq!(draft.points.len(), 4, "closing vertex is implied");
        draft.insert_after(3);
        assert_eq!(draft.points[4], [0.0, 2.0], "midpoint of the closing edge");
        assert_eq!(draft.selected, Some(4));
        draft.nudge(-1.0, 0.5);
        assert_eq!(draft.points[4], [-1.0, 2.5]);
        let ring = draft.closed_points();
        assert_eq!(ring.len(), 6);
        assert_eq!(ring[0], ring[5], "closed ring repeats its first vertex");
    }

    #[test]
    fn delete_never_drops_below_a_triangle() {
        let mut draft = PolygonDraft::from_ring(&SQUARE);
        draft.selected = Some(0);
        draft.delete_selected();
        assert_eq!(draft.points.len(), 3);
        draft.selected = Some(0);
        assert!(!draft.can_delete());
        draft.delete_selected();
        assert_eq!(draft.points.len(), 3);
    }

    #[test]
    fn hit_geometry_respects_concave_rings() {
        // L-shape: the notch at (3, 3) is inside the bounding box but outside the ring.
        let l_shape = [
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 2.0],
            [2.0, 2.0],
            [2.0, 4.0],
            [0.0, 4.0],
        ];
        assert!(point_in_polygon(&l_shape, [1.0, 3.0]));
        assert!(!point_in_polygon(&l_shape, [3.0, 3.0]));
        assert!(!point_in_polygon(&[], [0.0, 0.0]));
        assert_eq!(
            point_segment_distance([2.0, 1.0], [0.0, 0.0], [4.0, 0.0]),
            1.0
        );
        assert_eq!(
            point_segment_distance([7.0, 4.0], [0.0, 0.0], [4.0, 0.0]),
            5.0
        );
    }
}
