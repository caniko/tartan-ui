# Changelog

All notable changes to this workspace are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Changed

- Target Dioxus `0.8.0-alpha.3` (from `0.7.10`) across the workspace, including
  the `dioxus-ssr` test dependency and the version recorded in the README.
  Component APIs, feature names and rendered markup are unchanged.
- Raise the `tartan-ui-dioxus` minimum Rust version to 1.88: Dioxus 0.8 uses
  let-chains. `tartan-ui-core` keeps 1.85.
