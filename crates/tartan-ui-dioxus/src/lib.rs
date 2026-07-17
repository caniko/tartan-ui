//! Shared, product-neutral Dioxus presentation components.

use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, Feedback, FeedbackKind, Identity, Metric, NavigationLink, Progress,
    ResourceLink, ResourceSummary,
};

pub const SHARED_STYLES: Asset = asset!("/assets/tartan-ui.css");

#[component]
pub fn AppShell(
    title: String,
    identity: Option<Identity>,
    children: Element,
) -> Element {
    rsx! {
        document::Stylesheet { href: SHARED_STYLES }
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
    children: Element,
) -> Element {
    rsx! {
        document::Stylesheet { href: SHARED_STYLES }
        document::Title { "{title}" }
        div { class: "tartan-shell",
            BrandHeader { brand, home_href, identity }
            main { class: "tartan-shell__main", {children} }
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
fn BrandHeader(brand: String, home_href: String, identity: Option<Identity>) -> Element {
    rsx! {
        header { class: "tartan-header",
            a { class: "tartan-header__brand", href: "{home_href}", "{brand}" }
            div { class: "tartan-header__identity",
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
pub fn MediaPreview(src: Option<String>, alt: String, unavailable_label: String) -> Element {
    rsx! {
        div { class: "tartan-media-preview",
            if let Some(src) = src {
                img { src: "{src}", alt: "{alt}" }
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
pub fn EmptyState(heading: String, message: String) -> Element {
    rsx! {
        section { class: "tartan-empty", role: "status",
            h2 { "{heading}" }
            p { class: "tartan-muted", "{message}" }
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
        div { class: "tartan-loading", role: "status", aria_live: "polite",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_progress_model_is_renderable() {
        let progress = Progress { total: 4, complete: 1, secondary: 1 };
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
}
