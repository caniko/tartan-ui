# tartan-ui

Shared framework-neutral presentation contracts and Dioxus components for
bekiper, Pink Raven, SynDB, and Plinth.

The workspace currently targets stable Dioxus 0.7.9. Experimental Native/Blitz
preview support belongs in a separate gallery target and is not enabled by
default in production consumers.

## Crates

- `tartan-ui-core` contains serializable presentation models with no UI
  framework dependency.
- `tartan-ui-dioxus` contains shared Dioxus components and library assets.

The migration targets reserve Dioxus's opt-in `devtools` and `wasm-split`
features. Devtools/Subsecond-style hotpatching is development-only; WASM
splitting is enabled only for routes with a measurable payload benefit. The
LiveView feature is intentionally not enabled: these applications need normal
SSR/hydration and WebTransport/WebSocket-compatible browser clients.
