//! Shared, product-neutral Dioxus presentation components.

// Dioxus expands interpolated RSX attributes/text into `format!` calls. The
// interpolation is the renderer's supported dynamic-binding syntax, so this
// lint is a false positive for the component crate rather than an actionable
// allocation simplification.
#![allow(clippy::useless_format)]

use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, Feedback, FeedbackKind, FilterChoice, FilterOption, Identity, Metric,
    NavigationLink, Progress, ResourceLink, ResourceSummary, ThemePreference,
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
/// Render an image or unavailable state with an optional positioned overlay.
pub fn MediaPreview(
    src: Option<String>,
    alt: String,
    unavailable_label: String,
    #[props(default)] fit: MediaFit,
    #[props(default)] aspect: MediaAspect,
    #[props(default)] class: Option<String>,
    #[props(default)] overlay: Option<Element>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        div {
            class: "tartan-media-preview {class}",
            "data-media-fit": fit.as_str(),
            "data-media-aspect": aspect.as_str(),
            if let Some(src) = src {
                img { src: "{src}", alt: "{alt}", "data-media-image": "true" }
                if let Some(overlay) = overlay {
                    div { class: "tartan-media-preview__overlay", {overlay} }
                }
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

/// Associate a visible label with caller-provided form controls. The caller
/// owns form submission, validation, and the control's ID and name attributes.
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
            if let Some(hint) = hint { p { class: "tartan-field__hint", "{hint}" } }
            if let Some(error) = error { p { class: "tartan-field__error", role: "alert", "{error}" } }
        }
    }
}

/// Layout-neutral card for arbitrary caller-owned content.
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
                        if let Some(title) = title { h2 { class: "tartan-panel__title", "{title}" } }
                        if let Some(description) = description { p { class: "tartan-panel__description", "{description}" } }
                    }
                }
                {children}
            }
        }
    }
}

/// Link-based pagination; the host builds URLs and the summary (including
/// whether totals are exact). Missing destinations are omitted, not disabled.
#[component]
pub fn Pagination(
    label: String,
    #[props(default)] summary: Option<String>,
    #[props(default)] previous_href: Option<String>,
    #[props(default)] next_href: Option<String>,
    #[props(default)] previous_label: Option<String>,
    #[props(default)] next_label: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] link_class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    let link_class = link_class.unwrap_or_default();
    let default_link_class = if link_class.is_empty() {
        "tartan-pagination__link--default"
    } else {
        ""
    };
    let previous_label = previous_label.unwrap_or_else(|| "Previous".to_string());
    let next_label = next_label.unwrap_or_else(|| "Next".to_string());
    rsx! {
        nav { class: "tartan-pagination {class}", aria_label: "{label}",
            if let Some(summary) = summary { span { class: "tartan-pagination__summary", "{summary}" } }
            if let Some(href) = previous_href { a { class: "tartan-pagination__link {default_link_class} {link_class}", href: "{href}", "{previous_label}" } }
            if let Some(href) = next_href { a { class: "tartan-pagination__link {default_link_class} {link_class}", href: "{href}", "{next_label}" } }
        }
    }
}

/// Compose semantic label/value rows without imposing a product data model or
/// layout. Pass class `tartan-description-list--rows` for a shared grid layout.
#[component]
pub fn DescriptionList(children: Element, #[props(default)] class: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    rsx! { dl { class: "tartan-description-list {class}", {children} } }
}

#[component]
pub fn DescriptionItem(label: String, children: Element) -> Element {
    rsx! { div { class: "tartan-description-list__item", dt { "{label}" } dd { {children} } } }
}

/// Collapsed disclosure of caller-provided, escaped source text. An optional
/// pre class lets hosts retain their existing code-block presentation.
#[component]
pub fn DetailDisclosure(
    label: String,
    children: Element,
    #[props(default)] pre_class: Option<String>,
) -> Element {
    let pre_class = pre_class.unwrap_or_default();
    rsx! { details { class: "tartan-disclosure", summary { "{label}" }, pre { class: "{pre_class}", {children} } } }
}

/// Controlled native select. Form name and ID are the caller-provided ID.
#[component]
pub fn SelectField(
    id: String,
    label: String,
    value: String,
    options: Vec<FilterOption>,
    on_change: EventHandler<String>,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        div { class: "tartan-filter-field {class}",
            label { r#for: "{id}", "{label}" }
            select { id: "{id}", name: "{id}", value: "{value}", onchange: move |event| on_change.call(event.value()),
                for option in options {
                    option { value: "{option.value}", selected: option.value == value,
                        "{option.label}"
                        if let Some(count) = option.count { " ({count})" }
                    }
                }
            }
        }
    }
}

/// Explicitly controlled checkbox group. Caller interprets an empty selection.
#[component]
pub fn CheckboxFilterGroup(
    legend: String,
    name: String,
    options: Vec<FilterChoice>,
    #[props(default)] on_change: Option<EventHandler<(String, bool)>>,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        fieldset { class: "tartan-filter-group {class}",
            legend { "{legend}" }
            CheckboxFilterOptions { name, options, on_change }
        }
    }
}

/// Controlled checkbox labels for callers that already own a fieldset or a
/// disclosure. All options share the supplied form name; the caller decides
/// whether to submit the form or update local state on change.
#[component]
pub fn CheckboxFilterOptions(
    name: String,
    options: Vec<FilterChoice>,
    #[props(default)] on_change: Option<EventHandler<(String, bool)>>,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        for option in options {
            label { key: "{option.value}", class: "{class}",
                input { r#type: "checkbox", name: "{name}", value: "{option.value}", checked: option.checked,
                    onchange: {
                        let value = option.value.clone();
                        move |event| {
                            if let Some(on_change) = on_change {
                                on_change.call((value.clone(), event.checked()));
                            }
                        }
                    }
                }
                " {option.label}"
                if let Some(count) = option.count { " ({count})" }
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
    fn media_contracts_have_stable_dom_values() {
        assert_eq!(MediaFit::default().as_str(), "contain");
        assert_eq!(MediaFit::Cover.as_str(), "cover");
        assert_eq!(MediaAspect::default().as_str(), "auto");
        assert_eq!(MediaAspect::FourThree.as_str(), "four-three");
    }

    #[test]
    fn field_and_panel_compose_with_caller_owned_controls() {
        let html = dioxus_ssr::render_element(rsx! {
            Panel { title: "Search".to_string(), description: "Find records".to_string(),
                Field { id: "q".to_string(), label: "Search term".to_string(),
                    hint: "Optional".to_string(), error: "Try again".to_string(),
                    input { id: "q", name: "q", r#type: "search" }
                }
            }
        });
        assert!(html.contains("class=\"tartan-panel\""));
        assert!(html.contains("for=\"q\""));
        assert!(html.contains("name=\"q\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Find records"));
    }

    #[test]
    fn pagination_renders_known_and_unknown_ranges_without_inventing_links() {
        let html = dioxus_ssr::render_element(rsx! {
            Pagination {
                label: "Search result pages".to_string(),
                summary: "1–20 loaded".to_string(),
                next_href: "/search?offset=20".to_string(),
                next_label: "Next page".to_string(),
            }
        });
        assert!(html.contains("aria-label=\"Search result pages\""));
        assert!(html.contains("1–20 loaded"));
        assert!(!html.contains(">Previous</a>"));
        assert!(html.contains("href=\"/search?offset=20\""));
        assert!(html.contains("Next page"));
    }

    #[test]
    fn descriptions_and_disclosures_escape_source_text() {
        let untrusted = "<untrusted>".to_string();
        let raw = r#"{"key":"<raw>"}"#.to_string();
        let html = dioxus_ssr::render_element(rsx! {
            DescriptionList {
                DescriptionItem { label: "Source".to_string(), "{untrusted}" }
            }
            DetailDisclosure { label: "Raw JSON".to_string(), pre_class: "code-block".to_string(), "{raw}" }
        });
        assert!(html.contains("<dt>Source</dt>"));
        assert!(html.contains("&#60;untrusted&#62;"), "{html}");
        assert!(html.contains("<summary>Raw JSON</summary>"));
        assert!(html.contains("<pre class=\"code-block\">"));
        assert!(html.contains("&#60;raw&#62;"));
    }

    #[test]
    fn controlled_filter_fields_keep_caller_values_and_counts() {
        #[component]
        fn Fixture() -> Element {
            rsx! {
            SelectField {
                id: "species".to_string(), label: "Species".to_string(),
                value: "mouse".to_string(), options: vec![
                    FilterOption { value: "".to_string(), label: "All".to_string(), count: None },
                    FilterOption { value: "mouse".to_string(), label: "Mouse".to_string(), count: Some(3) },
                ], on_change: |_| {},
            }
            CheckboxFilterGroup {
                legend: "Kinds".to_string(), name: "kind".to_string(),
                options: vec![FilterChoice { value: "pdf".to_string(), label: "PDF".to_string(), count: Some(2), checked: true }],
                on_change: |_| {},
            }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert!(html.contains("for=\"species\""));
        assert!(html.contains("name=\"species\""));
        assert!(html.contains("selected"));
        assert!(html.contains("Mouse (3)"));
        assert!(html.contains("name=\"kind\""));
        assert!(html.contains("checked"));
        assert!(html.contains("PDF (2)"));
    }

    #[test]
    fn checkbox_options_fit_caller_owned_disclosures() {
        #[component]
        fn Fixture() -> Element {
            rsx! {
                details {
                    summary { "File types" }
                    CheckboxFilterOptions {
                        name: "type".to_string(),
                        options: vec![
                            FilterChoice { value: "pdf".to_string(), label: "PDF".to_string(), count: Some(2), checked: true },
                            FilterChoice { value: "json".to_string(), label: "JSON".to_string(), count: None, checked: false },
                        ],
                    }
                }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert_eq!(html.matches("<fieldset").count(), 0);
        assert_eq!(html.matches("name=\"type\"").count(), 2);
        assert!(html.contains("value=\"pdf\" checked"));
        assert!(html.contains("PDF (2)"));
        assert!(html.contains("<summary>File types</summary>"));
    }
}
