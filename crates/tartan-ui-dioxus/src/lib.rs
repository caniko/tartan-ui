//! Shared, product-neutral Dioxus presentation components.

use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, Identity, Metric, Progress, ResourceLink, ResourceSummary,
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

#[component]
pub fn Header(identity: Option<Identity>) -> Element {
    rsx! {
        header { class: "tartan-header",
            a { class: "tartan-header__brand", href: "/", "Tartanoglu" }
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
}
