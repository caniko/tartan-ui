//! Renderer-neutral editing primitives for tools that review and correct
//! geometry over an image: a pan/zoom viewport, an editable polygon ring, hit
//! geometry and label placement inside polygons. Hosts keep their own
//! renderer, persistence, revision binding and keyboard policy. The `dioxus`
//! feature adds an SVG vertex/midpoint handle layer for [`PolygonDraft`].

mod labels;
mod polygon;
mod viewport;

#[cfg(feature = "dioxus")]
mod dioxus;

#[cfg(feature = "dioxus")]
pub use dioxus::PolygonHandles;
pub use labels::{LabelPlacement, LabelRequest, label_anchor, layout_labels};
pub use polygon::{MIN_POLYGON_VERTICES, PolygonDraft, point_in_polygon, point_segment_distance};
pub use viewport::{HANDLE_HIT_PX, Viewport, svg_meet_to_content};
