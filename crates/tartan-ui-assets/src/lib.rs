//! Shared, opt-in presentation assets without a UI framework or asset linker.
//!
//! Static generators include this stylesheet in their output. Interactive
//! renderers use the same bytes alongside their component-specific styles.

pub const LAYOUT_STYLES: &str = include_str!("../assets/layout.css");

#[cfg(test)]
mod tests {
    const STYLESHEET: &str = include_str!("../assets/layout.css");

    fn declaration(name: &str) -> &'static str {
        STYLESHEET
            .lines()
            .find(|line| line.trim_start().starts_with(name))
            .unwrap_or_else(|| panic!("missing declaration {name}"))
    }

    fn rule(selector: &str) -> &'static str {
        let start = STYLESHEET
            .find(selector)
            .unwrap_or_else(|| panic!("missing rule {selector}"));
        let rule = &STYLESHEET[start..];
        &rule[..rule.find('}').unwrap_or(rule.len())]
    }

    #[test]
    fn aliases_consume_semantic_theme_roles() {
        for (alias, role) in [
            ("--tartan-accent:", "--tb-ui-accent"),
            ("--tartan-accent-hover:", "--tb-ui-accent-hover"),
            ("--tartan-accent-active:", "--tb-ui-accent-active"),
            ("--tartan-on-accent:", "--tb-ui-on-accent"),
            ("--tartan-focus:", "--tb-ui-focus"),
            (
                "--tartan-selection-background:",
                "--tb-ui-selection-background",
            ),
            (
                "--tartan-selection-foreground:",
                "--tb-ui-selection-foreground",
            ),
            ("--tartan-surface:", "--tb-ui-surface"),
        ] {
            let line = declaration(alias);
            assert!(line.contains(role), "{alias} must consume {role}: {line}");
        }
    }

    #[test]
    fn on_accent_prefers_the_explicit_role_and_keeps_the_old_background_fallback() {
        let line = declaration("--tartan-on-accent:");
        assert!(
            line.trim_start()
                .starts_with("--tartan-on-accent: var(--tb-ui-on-accent,"),
            "{line}"
        );
        assert!(line.contains("var(--tb-ui-background, #ffffff)"), "{line}");
    }

    #[test]
    fn aliases_stay_palette_agnostic() {
        assert!(!STYLESHEET.to_ascii_lowercase().contains("honey"));
    }

    #[test]
    fn layout_styles_do_not_install_selection_rules() {
        assert!(!STYLESHEET.contains("::selection"));
    }

    #[test]
    fn focus_ring_uses_the_focus_role() {
        let focus = rule(":focus-visible");
        assert!(focus.contains("outline: 2px solid var(--tartan-focus)"));
        assert!(!focus.contains("var(--tartan-accent"));
    }
}
