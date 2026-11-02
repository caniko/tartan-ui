//! Lucide icons (ISC, see `LICENSE-lucide`) vendored as inline SVG components.
//!
//! Every icon is a 24x24 stroke glyph drawn in `currentColor`. The icon is
//! decorative (`aria-hidden`) unless a `label` is supplied, in which case it
//! becomes `role="img"` with that accessible name. Path data is copied
//! verbatim from Lucide; do not edit it by hand.

use dioxus::prelude::*;

#[component]
fn IconBase(size: u32, stroke_width: f32, label: Option<String>, children: Element) -> Element {
    rsx! {
        svg {
            class: "tartan-icon",
            xmlns: "http://www.w3.org/2000/svg",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "{stroke_width}",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            role: label.is_some().then_some("img"),
            "aria-label": label.clone(),
            "aria-hidden": label.is_none().then_some("true"),
            "focusable": "false",
            {children}
        }
    }
}

macro_rules! icon {
    ($(#[$meta:meta])* $name:ident, $($shape:tt)*) => {
        $(#[$meta])*
        #[component]
        pub fn $name(
            #[props(default = 16)] size: u32,
            #[props(default = 1.5)] stroke_width: f32,
            #[props(default)] label: Option<String>,
        ) -> Element {
            rsx! { IconBase { size, stroke_width, label, $($shape)* } }
        }
    };
}

icon!(
    ChevronLeft,
    path {
        d: "m15 18-6-6 6-6"
    }
);
icon!(ChevronRight, path { d: "m9 18 6-6-6-6" });
icon!(
    Circle,
    circle {
        cx: "12",
        cy: "12",
        r: "10"
    }
);
icon!(
    Square,
    rect {
        width: "18",
        height: "18",
        x: "3",
        y: "3",
        rx: "2"
    }
);
icon!(Undo2,
    path { d: "M9 14 4 9l5-5" }
    path { d: "M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11" }
);
icon!(Redo2,
    path { d: "m15 14 5-5-5-5" }
    path { d: "M20 9H9.5A5.5 5.5 0 0 0 4 14.5A5.5 5.5 0 0 0 9.5 20H13" }
);
icon!(Save,
    path { d: "M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" }
    path { d: "M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" }
    path { d: "M7 3v4a1 1 0 0 0 1 1h7" }
);
icon!(
    /// Lucide `ellipsis` (formerly `more-horizontal`).
    MoreHorizontal,
    circle { cx: "12", cy: "12", r: "1" }
    circle { cx: "19", cy: "12", r: "1" }
    circle { cx: "5", cy: "12", r: "1" }
);
icon!(Plus,
    path { d: "M5 12h14" }
    path { d: "M12 5v14" }
);
icon!(Minus, path { d: "M5 12h14" });
icon!(Sun,
    circle { cx: "12", cy: "12", r: "4" }
    path { d: "M12 2v2" }
    path { d: "M12 20v2" }
    path { d: "m4.93 4.93 1.41 1.41" }
    path { d: "m17.66 17.66 1.41 1.41" }
    path { d: "M2 12h2" }
    path { d: "M20 12h2" }
    path { d: "m6.34 17.66-1.41 1.41" }
    path { d: "m19.07 4.93-1.41 1.41" }
);
icon!(
    Moon,
    path {
        d: "M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"
    }
);
icon!(Monitor,
    rect { width: "20", height: "14", x: "2", y: "3", rx: "2" }
    line { x1: "8", x2: "16", y1: "21", y2: "21" }
    line { x1: "12", x2: "12", y1: "17", y2: "21" }
);
icon!(ExternalLink,
    path { d: "M15 3h6v6" }
    path { d: "M10 14 21 3" }
    path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
);
icon!(Copy,
    rect { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" }
    path { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }
);

#[cfg(test)]
mod tests {
    use super::*;

    fn render(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn decorative_icons_are_hidden_with_defaults() {
        let html = render(rsx! { Sun {} });
        assert!(html.contains("aria-hidden=\"true\""), "{html}");
        assert!(!html.contains("role="), "{html}");
        assert!(html.contains("width=\"16\"") && html.contains("height=\"16\""));
        assert!(html.contains("stroke-width=\"1.5\""), "{html}");
        assert!(html.contains("stroke=\"currentColor\""));
        assert!(html.contains("viewBox=\"0 0 24 24\""), "{html}");
    }

    #[test]
    fn labelled_icons_expose_an_image_role() {
        let html = render(rsx! { Save { size: 24, label: "Save".to_string() } });
        assert!(
            html.contains("role=\"img\"") && html.contains("aria-label=\"Save\""),
            "{html}"
        );
        assert!(!html.contains("aria-hidden"), "{html}");
        assert!(html.contains("width=\"24\""));
    }

    #[test]
    fn every_icon_renders_shapes() {
        let all = [
            render(rsx! { ChevronLeft {} }),
            render(rsx! { ChevronRight {} }),
            render(rsx! { Circle {} }),
            render(rsx! { Square {} }),
            render(rsx! { Undo2 {} }),
            render(rsx! { Redo2 {} }),
            render(rsx! { Save {} }),
            render(rsx! { MoreHorizontal {} }),
            render(rsx! { Plus {} }),
            render(rsx! { Minus {} }),
            render(rsx! { Sun {} }),
            render(rsx! { Moon {} }),
            render(rsx! { Monitor {} }),
            render(rsx! { ExternalLink {} }),
            render(rsx! { Copy {} }),
        ];
        assert_eq!(all.len(), 15);
        for html in all {
            assert!(
                ["<path", "<circle", "<rect"]
                    .iter()
                    .any(|shape| html.contains(shape)),
                "{html}"
            );
        }
    }
}
