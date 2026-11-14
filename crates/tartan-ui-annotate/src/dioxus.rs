// Dioxus expands interpolated RSX attributes into `format!` calls.
#![allow(clippy::useless_format)]

use dioxus::prelude::*;

use crate::PolygonDraft;

/// Vertex and edge-midpoint handles for a [`PolygonDraft`], drawn in the
/// draft's content coordinates inside a host `<svg>`. `pixels_per_unit` keeps
/// handles a fixed on-screen size. Pressing a vertex reports its index;
/// pressing a midpoint reports the edge index so the host can
/// [`PolygonDraft::insert_after`]. The host owns drag tracking, styling
/// (`tartan-annotate-*` classes) and what the edit means.
#[component]
pub fn PolygonHandles(
    draft: PolygonDraft,
    pixels_per_unit: f64,
    on_vertex_down: EventHandler<usize>,
    on_insert_vertex: EventHandler<usize>,
) -> Element {
    let unit = 1.0 / pixels_per_unit.max(1e-9);
    rsx! {
        g {
            class: "tartan-annotate-handles",
            "data-geometry-handles": "true",
            for (index, [ax, ay]) in draft.points.iter().copied().enumerate() {
                {
                    let [bx, by] = draft.points[(index + 1) % draft.points.len()];
                    rsx! {
                        g {
                            key: "edge-{index}",
                            class: "tartan-annotate-midpoint",
                            "data-edge": "{index}",
                            onpointerdown: move |event| {
                                event.prevent_default();
                                event.stop_propagation();
                                on_insert_vertex.call(index);
                            },
                            // 44px touch target around a small visible dot.
                            circle { class: "tartan-annotate-hit", fill: "transparent", pointer_events: "all", cx: "{(ax + bx) / 2.0}", cy: "{(ay + by) / 2.0}", r: "{22.0 * unit}" }
                            circle { class: "tartan-annotate-dot", pointer_events: "none", cx: "{(ax + bx) / 2.0}", cy: "{(ay + by) / 2.0}", r: "{4.0 * unit}" }
                        }
                    }
                }
            }
            for (index, [x, y]) in draft.points.iter().copied().enumerate() {
                g {
                    key: "vertex-{index}",
                    class: "tartan-annotate-vertex",
                    "data-vertex": "{index}",
                    "data-selected": (draft.selected == Some(index)).to_string(),
                    onpointerdown: move |event| {
                        event.prevent_default();
                        event.stop_propagation();
                        on_vertex_down.call(index);
                    },
                    circle { class: "tartan-annotate-hit", fill: "transparent", pointer_events: "all", cx: "{x}", cy: "{y}", r: "{22.0 * unit}" }
                    circle { class: "tartan-annotate-dot", pointer_events: "none", cx: "{x}", cy: "{y}", r: "{7.0 * unit}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_render_one_vertex_and_midpoint_per_edge() {
        // Event handlers need a runtime, so render through a VirtualDom.
        fn app() -> Element {
            rsx! {
                svg {
                    PolygonHandles {
                        draft: PolygonDraft {
                            points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]],
                            selected: Some(1),
                        },
                        pixels_per_unit: 2.0,
                        on_vertex_down: |_| {},
                        on_insert_vertex: |_| {},
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("data-vertex=").count(), 3, "{html}");
        assert_eq!(html.matches("data-edge=").count(), 3, "{html}");
        assert_eq!(html.matches("data-selected=\"true\"").count(), 1, "{html}");
        assert!(html.contains("data-geometry-handles=\"true\""), "{html}");
        assert!(html.contains("fill=\"transparent\""), "{html}");
        // Midpoint of the closing edge (4,4) → (0,0); radii scale by 1/pixels_per_unit.
        assert!(html.contains("cx=\"2\" cy=\"2\" r=\"11\""), "{html}");
    }
}
