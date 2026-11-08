use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::polygon::{point_in_polygon, point_segment_distance};

/// The point inside `ring` farthest from its edges (the pole of
/// inaccessibility), found to within `precision` content units. Unlike a
/// vertex average it stays inside thin, L-shaped and concave rooms, so a label
/// placed there names the shape it sits in. `None` for fewer than three
/// vertices or a ring without area.
pub fn label_anchor(ring: &[[f64; 2]], precision: f64) -> Option<[f64; 2]> {
    let mut ring = ring.to_vec();
    if ring.len() > 1 && ring.first() == ring.last() {
        ring.pop();
    }
    if ring.len() < 3 {
        return None;
    }
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for [x, y] in &ring {
        min = [min[0].min(*x), min[1].min(*y)];
        max = [max[0].max(*x), max[1].max(*y)];
    }
    let cell_size = (max[0] - min[0]).min(max[1] - min[1]);
    let (area, centroid) = area_centroid(&ring);
    if cell_size <= 0.0 || area.abs() <= f64::EPSILON {
        return None;
    }
    let precision = precision.max(cell_size * 1e-6);
    let mut queue = BinaryHeap::new();
    let half = cell_size / 2.0;
    let mut x = min[0];
    while x < max[0] {
        let mut y = min[1];
        while y < max[1] {
            queue.push(Cell::new([x + half, y + half], half, &ring));
            y += cell_size;
        }
        x += cell_size;
    }
    let mut best = Cell::new(centroid, 0.0, &ring);
    let middle = Cell::new(
        [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
        0.0,
        &ring,
    );
    if middle.distance > best.distance {
        best = middle;
    }
    while let Some(cell) = queue.pop() {
        if cell.potential - best.distance.max(cell.distance) > precision {
            let half = cell.half / 2.0;
            for [dx, dy] in [[-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0], [1.0, 1.0]] {
                let center = [cell.center[0] + dx * half, cell.center[1] + dy * half];
                queue.push(Cell::new(center, half, &ring));
            }
        }
        if cell.distance > best.distance {
            best = cell;
        }
    }
    Some(best.center)
}

/// A label box to place: centred on `anchor` when possible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelRequest {
    pub anchor: [f64; 2],
    /// Width and height in the same units as `anchor`.
    pub size: [f64; 2],
    /// Higher priorities are placed first and so keep their anchors.
    pub priority: i32,
}

/// Where a label landed. `displaced` labels moved off their anchor; hosts
/// usually draw a short leader line back to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelPlacement {
    pub center: [f64; 2],
    pub displaced: bool,
}

/// Place label boxes inside `bounds` (`[min_x, min_y, max_x, max_y]`) so they
/// do not overlap. Each label tries its anchor, then offsets in eight
/// directions at growing distances; when none is free it takes the candidate
/// with the least overlap. The result is in request order.
// ponytail: greedy O(n² · candidates) scan, fine for the dozens of rooms on a
// floorplan; switch to a spatial index if a host places thousands of labels.
pub fn layout_labels(requests: &[LabelRequest], bounds: [f64; 4]) -> Vec<LabelPlacement> {
    const DIRECTIONS: [[f64; 2]; 8] = [
        [0.0, -1.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [-1.0, 0.0],
        [1.0, -1.0],
        [-1.0, -1.0],
        [1.0, 1.0],
        [-1.0, 1.0],
    ];
    let mut order: Vec<usize> = (0..requests.len()).collect();
    order.sort_by_key(|&index| std::cmp::Reverse(requests[index].priority));
    let mut placed: Vec<([f64; 2], [f64; 2])> = Vec::with_capacity(requests.len());
    let mut result = vec![
        LabelPlacement {
            center: [0.0, 0.0],
            displaced: false,
        };
        requests.len()
    ];
    for index in order {
        let LabelRequest { anchor, size, .. } = requests[index];
        let clamp = |center: [f64; 2]| {
            let fit = |value: f64, low: f64, high: f64, extent: f64| {
                let (low, high) = (low + extent / 2.0, high - extent / 2.0);
                if low > high {
                    (low + high) / 2.0
                } else {
                    value.clamp(low, high)
                }
            };
            [
                fit(center[0], bounds[0], bounds[2], size[0]),
                fit(center[1], bounds[1], bounds[3], size[1]),
            ]
        };
        let overlap = |center: [f64; 2]| -> f64 {
            placed
                .iter()
                .map(|(other, other_size)| {
                    let w = (size[0] + other_size[0]) / 2.0 - (center[0] - other[0]).abs();
                    let h = (size[1] + other_size[1]) / 2.0 - (center[1] - other[1]).abs();
                    if w > 0.0 && h > 0.0 { w * h } else { 0.0 }
                })
                .sum()
        };
        let candidates = std::iter::once(anchor).chain((1..=8).flat_map(|ring| {
            DIRECTIONS.iter().map(move |[dx, dy]| {
                let step = ring as f64;
                [
                    anchor[0] + dx * step * size[0] * 0.6,
                    anchor[1] + dy * step * size[1] * 1.1,
                ]
            })
        }));
        let mut best = (clamp(anchor), f64::MAX);
        for candidate in candidates.map(clamp) {
            let cost = overlap(candidate);
            if cost < best.1 {
                best = (candidate, cost);
            }
            if cost == 0.0 {
                break;
            }
        }
        placed.push((best.0, size));
        result[index] = LabelPlacement {
            center: best.0,
            displaced: (best.0[0] - anchor[0]).hypot(best.0[1] - anchor[1]) > f64::EPSILON,
        };
    }
    result
}

struct Cell {
    center: [f64; 2],
    half: f64,
    /// Signed distance to the ring: positive inside.
    distance: f64,
    /// The best distance any point in this cell could reach.
    potential: f64,
}

impl Cell {
    fn new(center: [f64; 2], half: f64, ring: &[[f64; 2]]) -> Self {
        let edge = (0..ring.len())
            .map(|index| {
                point_segment_distance(center, ring[index], ring[(index + 1) % ring.len()])
            })
            .fold(f64::MAX, f64::min);
        let distance = if point_in_polygon(ring, center) {
            edge
        } else {
            -edge
        };
        Self {
            center,
            half,
            distance,
            potential: distance + half * std::f64::consts::SQRT_2,
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.potential == other.potential
    }
}

impl Eq for Cell {}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.potential.total_cmp(&other.potential)
    }
}

fn area_centroid(ring: &[[f64; 2]]) -> (f64, [f64; 2]) {
    let (mut area, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for index in 0..ring.len() {
        let [ax, ay] = ring[index];
        let [bx, by] = ring[(index + 1) % ring.len()];
        let cross = ax * by - bx * ay;
        area += cross;
        cx += (ax + bx) * cross;
        cy += (ay + by) * cross;
    }
    if area.abs() <= f64::EPSILON {
        return (0.0, ring[0]);
    }
    (area / 2.0, [cx / (3.0 * area), cy / (3.0 * area)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_stays_inside_an_l_shaped_room() {
        // A thin L whose vertex average (≈ 3.3, 3.3) lies outside it.
        let ring = [
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 1.0],
            [1.0, 1.0],
            [1.0, 10.0],
            [0.0, 10.0],
        ];
        let average = [
            ring.iter().map(|p| p[0]).sum::<f64>() / 6.0,
            ring.iter().map(|p| p[1]).sum::<f64>() / 6.0,
        ];
        assert!(!point_in_polygon(&ring, average));
        let anchor = label_anchor(&ring, 0.01).expect("ring has area");
        assert!(point_in_polygon(&ring, anchor), "{anchor:?} is inside");
        assert!(label_anchor(&[[0.0, 0.0], [1.0, 1.0]], 0.1).is_none());
        assert!(label_anchor(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]], 0.1).is_none());
    }

    #[test]
    fn anchor_of_a_square_is_its_centre() {
        let anchor =
            label_anchor(&[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], 0.001).unwrap();
        assert!(
            (anchor[0] - 2.0).abs() < 0.01 && (anchor[1] - 2.0).abs() < 0.01,
            "{anchor:?}"
        );
    }

    #[test]
    fn stacked_labels_are_separated_by_priority() {
        let request = |priority| LabelRequest {
            anchor: [50.0, 50.0],
            size: [20.0, 8.0],
            priority,
        };
        let placed = layout_labels(
            &[request(0), request(5), request(1)],
            [0.0, 0.0, 100.0, 100.0],
        );
        assert_eq!(
            placed[1],
            LabelPlacement {
                center: [50.0, 50.0],
                displaced: false
            }
        );
        assert!(placed[0].displaced && placed[2].displaced);
        for (i, a) in placed.iter().enumerate() {
            for b in &placed[i + 1..] {
                let apart = (a.center[0] - b.center[0]).abs() >= 20.0
                    || (a.center[1] - b.center[1]).abs() >= 8.0;
                assert!(apart, "{a:?} overlaps {b:?}");
            }
            assert!(
                a.center[0] >= 10.0
                    && a.center[0] <= 90.0
                    && a.center[1] >= 4.0
                    && a.center[1] <= 96.0
            );
        }
    }
}
