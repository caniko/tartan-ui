//! Shared, opt-in presentation assets without a UI framework or asset linker.
//!
//! Static generators include this stylesheet in their output. Interactive
//! renderers use the same bytes alongside their component-specific styles.

pub const LAYOUT_STYLES: &str = include_str!("../assets/layout.css");
