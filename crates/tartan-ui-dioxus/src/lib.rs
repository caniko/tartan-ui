//! Shared, product-neutral Dioxus presentation components.

// Dioxus expands interpolated RSX attributes/text into `format!` calls. The
// interpolation is the renderer's supported dynamic-binding syntax, so this
// lint is a false positive for the component crate rather than an actionable
// allocation simplification.
#![allow(clippy::useless_format)]

use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, Feedback, FeedbackKind, Identity, Metric, NavigationLink, Progress,
    ResourceLink, ResourceSummary, ThemePreference,
};

#[cfg(all(feature = "web", feature = "native-embedded"))]
compile_error!("tartan-ui-dioxus features `web` and `native-embedded` are mutually exclusive");

#[cfg(not(feature = "native-embedded"))]
pub const SHARED_STYLES: Asset = asset!("/assets/tartan-ui.css");

#[cfg(feature = "native-embedded")]
pub const SHARED_STYLES: &str = include_str!("../assets/tartan-ui.css");

fn shared_styles() -> Element {
    #[cfg(feature = "native-embedded")]
    {
        rsx! {
            style { dangerous_inner_html: SHARED_STYLES }
        }
    }

    #[cfg(not(feature = "native-embedded"))]
    {
        rsx! {
            document::Stylesheet { href: SHARED_STYLES }
        }
    }
}

/// How a source image should occupy its preview frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaFit {
    #[default]
    Contain,
    Cover,
}

impl MediaFit {
    fn as_str(self) -> &'static str {
        match self {
            Self::Contain => "contain",
            Self::Cover => "cover",
        }
    }
}

/// Stable aspect contracts for shared media surfaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaAspect {
    #[default]
    Auto,
    Square,
    FourThree,
    Wide,
}

impl MediaAspect {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Square => "square",
            Self::FourThree => "four-three",
            Self::Wide => "wide",
        }
    }
}

#[component]
pub fn AppShell(title: String, identity: Option<Identity>, children: Element) -> Element {
    rsx! {
        {shared_styles()}
        document::Title { "{title}" }
        div { class: "tartan-shell",
            Header { identity }
            main { class: "tartan-shell__main", {children} }
        }
    }
}

/// Product-neutral shell composition. Branding, navigation and product state
/// stay with the caller while the document and layout contract is shared.
#[component]
pub fn ProductShell(
    title: String,
    brand: String,
    home_href: String,
    identity: Option<Identity>,
    #[props(default)] header_actions: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        {shared_styles()}
        document::Title { "{title}" }
        div { class: "tartan-shell",
            BrandHeader { brand, home_href, identity, header_actions }
            main { class: "tartan-shell__main", {children} }
        }
    }
}

/// A renderer-neutral theme control. The host owns persistence and applies
/// the returned preference to its document or native window.
#[component]
pub fn ThemeToggle(
    preference: ThemePreference,
    on_toggle: EventHandler<ThemePreference>,
) -> Element {
    let next = preference.next();
    let label = match preference {
        ThemePreference::System => "Use light theme",
        ThemePreference::Light => "Use dark theme",
        ThemePreference::Dark => "Use system theme",
    };
    rsx! {
        button {
            class: "tartan-theme-toggle",
            type: "button",
            aria_label: "{label}",
            aria_pressed: preference != ThemePreference::System,
            onclick: move |_| on_toggle.call(next),
            "Theme: {preference.as_str()}"
        }
    }
}

#[component]
pub fn Header(identity: Option<Identity>) -> Element {
    rsx! {
        BrandHeader {
            brand: "Tartanoglu".to_string(),
            home_href: "/".to_string(),
            identity,
        }
    }
}

#[component]
fn BrandHeader(
    brand: String,
    home_href: String,
    identity: Option<Identity>,
    #[props(default)] header_actions: Option<Element>,
) -> Element {
    rsx! {
        header { class: "tartan-header",
            a { class: "tartan-header__brand", href: "{home_href}", "{brand}" }
            div { class: "tartan-header__identity",
                if let Some(actions) = header_actions {
                    div { class: "tartan-header__actions", {actions} }
                }
                if let Some(identity) = identity {
                    if let Some(account_url) = identity.account_url {
                        a { href: "{account_url}", target: "_blank", rel: "noopener", "{identity.display_name}" }
                    } else {
                        span { "{identity.display_name}" }
                    }
                    if let Some(email) = identity.email {
                        span { class: "tartan-header__email", "{email}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NavigationList(items: Vec<NavigationLink>, aria_label: String) -> Element {
    rsx! {
        nav { class: "tartan-nav", aria_label: "{aria_label}",
            for item in items {
                a {
                    class: if item.current { "tartan-nav__link tartan-nav__link--current" } else { "tartan-nav__link" },
                    href: "{item.href}",
                    aria_current: if item.current { "page" } else { "false" },
                    key: "{item.key}",
                    "{item.label}"
                }
            }
        }
    }
}

#[component]
pub fn PageLayout(children: Element) -> Element {
    rsx! { div { class: "tartan-page-layout", {children} } }
}

#[component]
pub fn CardGrid(children: Element) -> Element {
    rsx! { div { class: "tartan-card-grid", {children} } }
}

#[component]
pub fn TagList(tags: Vec<String>) -> Element {
    rsx! {
        ul { class: "tartan-tag-list", aria_label: "Tags",
            for tag in tags {
                li { class: "tartan-tag", key: "{tag}", "{tag}" }
            }
        }
    }
}

#[component]
pub fn MediaPreview(
    src: Option<String>,
    alt: String,
    unavailable_label: String,
    #[props(default)] fit: MediaFit,
    #[props(default)] aspect: MediaAspect,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        div {
            class: "tartan-media-preview {class}",
            "data-media-fit": fit.as_str(),
            "data-media-aspect": aspect.as_str(),
            if let Some(src) = src {
                img { src: "{src}", alt: "{alt}", "data-media-image": "true" }
            } else {
                p { class: "tartan-media-preview__unavailable", role: "status", "{unavailable_label}" }
            }
        }
    }
}

#[component]
pub fn DashboardHeader(eyebrow: String, heading: String, description: String) -> Element {
    rsx! {
        section { class: "tartan-dashboard-header",
            p { class: "tartan-eyebrow", "{eyebrow}" }
            h1 { "{heading}" }
            p { class: "tartan-muted", "{description}" }
        }
    }
}

#[component]
pub fn ResourceCard(resource: ResourceLink, action_label: String, children: Element) -> Element {
    rsx! {
        article { class: "tartan-card tartan-resource-card", key: "{resource.key}",
            div { class: "tartan-resource-card__body",
                h2 { "{resource.title}" }
                if let Some(subtitle) = resource.subtitle {
                    p { class: "tartan-muted", "{subtitle}" }
                }
                {children}
            }
            a { class: "tartan-button", href: "{resource.href}", "{action_label}" }
        }
    }
}

#[component]
pub fn ProgressSummary(progress: Progress) -> Element {
    let percent = progress.percent();
    rsx! {
        div { class: "tartan-progress", role: "group", aria_label: "Progress",
            div { class: "tartan-progress__labels",
                span { "{progress.complete} complete" }
                span { "{progress.remaining()} remaining" }
            }
            div { class: "tartan-progress__track", role: "progressbar",
                aria_valuemin: "0", aria_valuemax: "{progress.total}", aria_valuenow: "{progress.complete}",
                div { class: "tartan-progress__value", style: "width: {percent}%" }
            }
        }
    }
}

#[component]
pub fn MetricStrip(metrics: Vec<Metric>) -> Element {
    rsx! {
        div { class: "tartan-metrics",
            for metric in metrics {
                div { class: "tartan-metric", key: "{metric.label}",
                    strong { "{metric.value}" }
                    span { "{metric.label}" }
                    if let Some(description) = metric.description {
                        small { class: "tartan-muted", "{description}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn EmptyState(
    heading: String,
    message: String,
    #[props(default)] actions: Option<Element>,
) -> Element {
    rsx! {
        section { class: "tartan-empty", role: "status",
            h2 { "{heading}" }
            p { class: "tartan-muted", "{message}" }
            if let Some(actions) = actions {
                div { class: "tartan-empty__actions", {actions} }
            }
        }
    }
}

/// Render a product-provided message with a consistent visual treatment and
/// an appropriate live-region role. The application owns when a message is
/// shown; this component does not infer state or perform side effects.
#[component]
pub fn FeedbackBanner(feedback: Feedback) -> Element {
    let (modifier, role, live) = feedback_accessibility(&feedback.kind);

    rsx! {
        div {
            class: "tartan-feedback tartan-feedback--{modifier}",
            role: role,
            aria_live: live,
            "{feedback.message}"
        }
    }
}

fn feedback_accessibility(kind: &FeedbackKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        FeedbackKind::Info => ("info", "status", "polite"),
        FeedbackKind::Success => ("success", "status", "polite"),
        FeedbackKind::Warning => ("warning", "status", "assertive"),
        FeedbackKind::Error => ("error", "alert", "assertive"),
    }
}

/// Render a short-lived loading state while the host application resolves
/// data. The label is intentionally required so screen readers get useful
/// context instead of an unlabeled spinner.
#[component]
pub fn LoadingState(label: String) -> Element {
    rsx! {
        div { class: "tartan-loading", role: "status", aria_live: "polite", aria_busy: true,
            span { class: "tartan-loading__spinner", aria_hidden: "true" }
            span { "{label}" }
        }
    }
}

/// Shared authenticated landing view for products that expose resources such
/// as datasets, projects, or workspaces.
///
/// Authorization and summary loading stay in the host application. This
/// component only renders already-authorized values and navigates using the
/// server-provided link.
#[component]
pub fn ResourceDashboard(
    eyebrow: String,
    heading: String,
    description: String,
    resources: Vec<AccessibleResource>,
    summaries: Vec<(String, ResourceSummary)>,
) -> Element {
    rsx! {
        DashboardHeader { eyebrow, heading, description }
        if resources.is_empty() {
            EmptyState {
                heading: "Nothing shared yet".to_string(),
                message: "Your account is valid, but no resources have been shared with it.".to_string(),
            }
        } else {
            div { class: "tartan-resource-grid",
                for resource in resources {
                    ResourceDashboardCard { resource, summaries: summaries.clone() }
                }
            }
        }
    }
}

#[component]
fn ResourceDashboardCard(
    resource: AccessibleResource,
    summaries: Vec<(String, ResourceSummary)>,
) -> Element {
    let summary = summaries
        .iter()
        .find(|(id, _)| id == &resource.id)
        .map(|(_, summary)| *summary);
    rsx! {
        ResourceCard {
            resource: ResourceLink {
                key: resource.id,
                title: resource.name,
                subtitle: None,
                href: resource.href,
                kind: Some("authorized-resource".to_string()),
            },
            action_label: "Open".to_string(),
            if let Some(summary) = summary {
                ProgressSummary { progress: summary.progress() }
                MetricStrip {
                    metrics: vec![
                        Metric { label: "Complete".to_string(), value: summary.complete.to_string(), description: None },
                        Metric { label: "Secondary".to_string(), value: summary.secondary.to_string(), description: None },
                        Metric { label: "Remaining".to_string(), value: summary.remaining.to_string(), description: None },
                    ]
                }
            } else {
                p { class: "tartan-muted", "Summary is loading…" }
            }
        }
    }
}

/// Associate a visible label with caller-provided form control(s).
///
/// Layout-neutral: the component owns only the label/control association and
/// the hint/error presentation. Grid placement, input behavior, validation,
/// and form actions stay with the caller, which passes any control(s) as
/// children (input, select, textarea, or a control group).
///
/// Presentation lives in `assets/tartan-ui.css` and references only
/// `--tartan-*` tokens. Host applications remap those tokens to their own
/// palette through the `.tartan-shell` scope (one-way shared-token mapping);
/// this component must not depend on host stylesheets.
#[component]
pub fn Field(
    id: String,
    label: String,
    children: Element,
    #[props(default)] hint: Option<String>,
    #[props(default)] error: Option<String>,
) -> Element {
    rsx! {
        div { class: "tartan-field",
            label { class: "tartan-field__label", r#for: id, "{label}" }
            {children}
            if let Some(hint) = hint {
                p { class: "tartan-field__hint", "{hint}" }
            }
            if let Some(error) = error {
                p { class: "tartan-field__error", role: "alert", "{error}" }
            }
        }
    }
}

/// General information card. Accepts any children; a form is not required.
///
/// The card surface, border, padding, and title/description typography are
/// owned by `assets/tartan-ui.css` (token-driven, see [`Field`]). Callers
/// compose routes, grids, forms, and domain content inside.
#[component]
pub fn Panel(
    children: Element,
    #[props(default)] title: Option<String>,
    #[props(default)] description: Option<String>,
) -> Element {
    rsx! {
        section { class: "tartan-panel",
            div { class: "tartan-panel__body",
                if title.is_some() || description.is_some() {
                    div { class: "tartan-panel__header",
                        if let Some(title) = title {
                            h2 { class: "tartan-panel__title", "{title}" }
                        }
                        if let Some(description) = description {
                            p { class: "tartan-panel__description", "{description}" }
                        }
                    }
                }
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_progress_model_is_renderable() {
        let progress = Progress {
            total: 4,
            complete: 1,
            secondary: 1,
        };
        assert_eq!(progress.percent(), 25);
        assert_eq!(progress.remaining(), 2);
    }

    #[test]
    fn feedback_kinds_have_explicit_accessibility_contracts() {
        let cases = [
            (FeedbackKind::Info, "status", "polite"),
            (FeedbackKind::Success, "status", "polite"),
            (FeedbackKind::Warning, "status", "assertive"),
            (FeedbackKind::Error, "alert", "assertive"),
        ];

        for (kind, expected_role, expected_live) in cases {
            let (modifier, role, live) = feedback_accessibility(&kind);
            assert!(!modifier.is_empty());
            assert_eq!(role, expected_role);
            assert_eq!(live, expected_live);
        }
    }

    #[test]
    fn field_associates_label_with_control() {
        let html = dioxus_ssr::render_element(rsx! {
            Field { id: "q".to_string(), label: "Search term".to_string(),
                input { id: "q", name: "q" }
            }
        });

        assert!(html.contains("class=\"tartan-field\""));
        assert!(html.contains("for=\"q\""));
        assert!(html.contains("id=\"q\""));
        assert!(html.contains("Search term"));
    }

    #[test]
    fn field_passes_controls_through_unchanged() {
        let html = dioxus_ssr::render_element(rsx! {
            Field { id: "compartments".to_string(), label: "Compartments".to_string(),
                select { id: "compartments", name: "compartments", multiple: true,
                    option { value: "neuron", "Neuron" }
                }
            }
        });

        assert!(html.contains("multiple"));
        assert!(html.contains("value=\"neuron\""));
        assert!(html.contains("for=\"compartments\""));
    }

    #[test]
    fn field_hint_and_error_carry_their_semantics() {
        let html = dioxus_ssr::render_element(rsx! {
            Field {
                id: "note".to_string(),
                label: "Note".to_string(),
                hint: "Optional context for reviewers.".to_string(),
                error: "A note is required here.".to_string(),
                textarea { id: "note", name: "note" }
            }
        });

        assert!(html.contains("class=\"tartan-field__hint\""));
        assert!(html.contains("Optional context for reviewers."));
        assert!(html.contains("class=\"tartan-field__error\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("A note is required here."));
    }

    #[test]
    fn field_omits_hint_and_error_by_default() {
        let html = dioxus_ssr::render_element(rsx! {
            Field { id: "q".to_string(), label: "Search term".to_string(),
                input { id: "q", name: "q" }
            }
        });

        assert!(!html.contains("tartan-field__hint"));
        assert!(!html.contains("tartan-field__error"));
        assert!(!html.contains("role=\"alert\""));
    }

    #[test]
    fn panel_accepts_non_form_children() {
        let html = dioxus_ssr::render_element(rsx! {
            Panel {
                title: "Dataset created".to_string(),
                description: "The dataset metadata was created.".to_string(),
                p { "No form required." }
            }
        });

        assert!(html.contains("class=\"tartan-panel\""));
        assert!(html.contains("class=\"tartan-panel__body\""));
        assert!(html.contains("Dataset created"));
        assert!(html.contains("The dataset metadata was created."));
        assert!(html.contains("No form required."));
        assert!(!html.contains("<form"));
    }

    #[test]
    fn panel_omits_header_without_title_or_description() {
        let html = dioxus_ssr::render_element(rsx! {
            Panel { p { "Body only." } }
        });

        assert!(html.contains("class=\"tartan-panel\""));
        assert!(!html.contains("tartan-panel__header"));
        assert!(html.contains("Body only."));
    }

    #[test]
    fn media_contracts_have_stable_dom_values() {
        assert_eq!(MediaFit::default().as_str(), "contain");
        assert_eq!(MediaFit::Cover.as_str(), "cover");
        assert_eq!(MediaAspect::default().as_str(), "auto");
        assert_eq!(MediaAspect::FourThree.as_str(), "four-three");
    }
}
