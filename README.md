# tartan-ui

<!-- simit:badges:start -->

![CI](https://img.shields.io/badge/CI-managed-2088ff) [![Nix](https://img.shields.io/badge/Nix-drift-5277c3)](flake.nix) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/tartan-ui-core)

<!-- simit:badges:end -->

Shared framework-neutral presentation contracts and Dioxus components for
bekiper, Pink Raven, SynDB, and Plinth.

The workspace currently targets Dioxus 0.7.10. Experimental Native/Blitz
preview support belongs in a separate gallery target and is not enabled by
default in production consumers. The shared Dioxus crate itself has no
renderer default; applications opt into `web`, `server`, or a native target.
The `native` feature embeds the shared stylesheet (the equivalent explicit
`native-embedded` feature is available for custom renderer compositions), so
raw Blitz launches do not depend on a Dioxus asset-linker pass.

## Crates

- `tartan-ui-core` contains serializable presentation models with no UI
  framework dependency.
- `tartan-ui-assets` exposes opt-in responsive layout styles without a renderer
  dependency. Static generators and interactive components consume the same
  stylesheet; applications supply semantic theme variables.
- `tartan-ui-dioxus` contains shared Dioxus components and library assets.

Theme preference values live in `tartan-ui-core`; `ThemeToggle` in the Dioxus
crate cycles the value without owning persistence or document/window policy.

The first migration primitives are `FeedbackBanner` and `LoadingState`. They
accept already-mapped application values, expose explicit live-region semantics,
and do not make requests or decide application state. Keep those boundaries
when adapting Leptos products so the same contract remains usable from SSR,
hydration, and browser-only routes.

## Shared presentation contracts

`SectionHeader` composes a page heading and wrapping action slot.
`NavigationDisclosure` uses native disclosure behavior around caller-provided
navigation. Hosts retain route selection and authorization policy. The shared
layout stylesheet also supplies `tartan-flow`, `tartan-responsive-grid`, and
`tartan-surface` classes for static HTML compositions.

`Pagination` renders a caller-supplied range string and previous/next URLs; it
does not calculate totals, offsets, or cursors. `DescriptionList` and
`DescriptionItem` render semantic label/value pairs without changing the host
layout (opt into `tartan-description-list--rows` for a shared grid). Use
`DetailDisclosure` for escaped raw text, with `pre_class` if the host already
styles its code blocks.

`SelectField`, `CheckboxFilterGroup`, and `CheckboxFilterOptions` render
controlled choices (`FilterOption`/`FilterChoice` in `tartan-ui-core`). Use the
options-only component inside an existing fieldset or `<details>`; it does not
add another group. Both the form `name` and every option `value` are owned by
the consumer. Count labels are display-only, and an empty checkbox selection
has no shared interpretation. The host retains query parsing, submission,
authorization, and review decisions.

The migration targets reserve Dioxus's opt-in `devtools` and `wasm-split`
features. Devtools/Subsecond-style hotpatching is development-only; WASM
splitting is enabled only for routes with a measurable payload benefit. The
LiveView feature is intentionally not enabled: these applications need normal
SSR/hydration and WebTransport/WebSocket-compatible browser clients.
