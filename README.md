# tartan-ui

<!-- simit:badges:start -->

[![Nix](https://img.shields.io/badge/Nix-managed-5277c3)](flake.nix) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/tartan-ui-core)

<!-- simit:badges:end -->

Shared framework-neutral presentation contracts and Dioxus components for
bekiper, Pink Raven, SynDB, and Plinth.

The workspace currently targets stable Dioxus 0.7.9. Experimental Native/Blitz
preview support belongs in a separate gallery target and is not enabled by
default in production consumers. The shared Dioxus crate itself has no
renderer default; applications opt into `web`, `server`, or a native target.
The `native` feature embeds the shared stylesheet (the equivalent explicit
`native-embedded` feature is available for custom renderer compositions), so
raw Blitz launches do not depend on a Dioxus asset-linker pass.

## Crates

- `tartan-ui-core` contains serializable presentation models with no UI
  framework dependency.
- `tartan-ui-dioxus` contains shared Dioxus components and library assets.

Theme preference values live in `tartan-ui-core`; `ThemeToggle` in the Dioxus
crate cycles the value without owning persistence or document/window policy.

The first migration primitives are `FeedbackBanner` and `LoadingState`. They
accept already-mapped application values, expose explicit live-region semantics,
and do not make requests or decide application state. Keep those boundaries
when adapting Leptos products so the same contract remains usable from SSR,
hydration, and browser-only routes.

The migration targets reserve Dioxus's opt-in `devtools` and `wasm-split`
features. Devtools/Subsecond-style hotpatching is development-only; WASM
splitting is enabled only for routes with a measurable payload benefit. The
LiveView feature is intentionally not enabled: these applications need normal
SSR/hydration and WebTransport/WebSocket-compatible browser clients.
