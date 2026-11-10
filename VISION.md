# tartan-ui vision

`tartan-ui` is the shared presentation stack for Tartanoglu web applications
and sites: framework-neutral presentation models (`tartan-ui-core`), a
renderer-neutral layout stylesheet (`tartan-ui-assets`), shared Dioxus
components (`tartan-ui-dioxus`), and renderer-neutral annotation primitives
(`tartan-ui-annotate`). Consumers named in the README are bekiper, Pink Raven,
SynDB and Plinth; SynDB is the regression reference and QueryFabric the first
migration canary.

## Problem

Several first-party applications need the same accessible markup — banners,
loading states, section headers, pagination, description lists, disclosures,
filters, theme toggles — but each product keeps its own domain model, routes,
APIs, authorization and persistence. Copying markup per site drifts, and moving
it into a framework couples every consumer to that framework. The project
exists so a component that a second site needs lives in one place, with a
contract that static renderers and interactive Dioxus apps can both use.

## Principles

- **Contracts are framework-neutral.** `tartan-ui-core` holds serializable
  presentation models with no UI framework dependency; assets carry no
  renderer dependency; only `tartan-ui-dioxus` depends on Dioxus by default, and
  `tartan-ui-annotate` gates its Dioxus handle layer behind an opt-in feature.
- **Components accept mapped values and make no decisions.** Shared components
  take already-mapped application values, add explicit live-region and ARIA
  semantics, and never make requests or decide query parsing, submission,
  authorization or review outcomes. Hosts keep those.
- **Renderer is an explicit consumer choice.** The Dioxus crate has no renderer
  default; applications opt into `web`, `server` or a native target. Native and
  Blitz previews live in a separate gallery target and stay out of production
  consumers.
- **Reuse beats copy, but shared changes land upstream.** A component needed by
  a second site moves into tartan-ui, or into a plinth brick when it is site
  composition. Consumers pin by revision, a shared change lands here first, and
  each consumer bumps its own pin.
- **Style stays minimal and themed by the host.** 1px borders, one 6px radius,
  no shadows or gradients, a 4px spacing scale, accent only for primary
  actions, selection and the focus ring. `--tartan-*` aliases read the
  nix-theme-broker `--tb-*` values with Blackbox light fallbacks, so the crates
  work without the broker and gain dark mode by loading its stylesheet.
- **Accessibility and payload are acceptance criteria.** Contrast, keyboard
  focus, reduced motion and semantic markup are part of the quality bar;
  WASM splitting and devtools are opt-in per route, not defaults.
- **Third-party code keeps its licence.** Lucide icons are vendored with their
  ISC/MIT notices in `LICENSE-lucide` rather than re-licensed.

## Scope

- Framework-neutral presentation contracts and filter/query models.
- A responsive layout stylesheet for both static HTML and interactive apps.
- Accessible Dioxus components: feedback banners, loading state, section
  headers, navigation disclosure, pagination, description lists, detail
  disclosures, controlled checkboxes, select fields, theme toggle, Lucide
  icons, and a guided query combobox.
- Renderer-neutral annotation primitives: viewport pan/zoom/fit, polygon
  drafting with vertex editing, hit geometry, and an opt-in SVG handle layer.
- A gallery binary that renders the components and captures visual evidence.

## Non-goals

- Owning a product's palette, type scale or brand; those stay in each site's
  OpenPencil tokens and stylesheet, never inside tartan-ui.
- Owning routes, data fetching, authorization, persistence or state machines.
- Shipping a Leptos adapter; the migration target is Dioxus.
- Being a full design system or replacing OpenPencil as the design source.
- Enabling Dioxus LiveView or devtools in production paths.

## Direction

Recent commits build out the Dioxus layer: a guided query combobox (caret,
groups, marks, autoselect) and its follow-up fixes, persisted theme with
Lucide icons, broker theme aliases, and shared filter/detail primitives. The
canix migration plan records the wider direction: every active first-party UI
on stable Dioxus with this released presentation stack, `queryfabric-dioxus`
replacing the Leptos adapter, and each product keeping its own domain code. Its
remaining work is release and downstream pinning, with `tzu`, `bekiper`,
`plinth`, `pink-raven` and `SynDB` as the disposition list.

## Place in the fleet

Canix's design catalog names tartan-ui as the home for accessible Dioxus markup
and framework-neutral contracts for every site; the Paperclip design company
owns it, and the design company policy says a shared change lands upstream
first, then each consumer bumps its pin in its own pull request.
