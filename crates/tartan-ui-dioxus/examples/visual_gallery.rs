use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, Feedback, FeedbackKind, Identity, Metric, NavigationLink, ResourceSummary,
    ThemePreference,
};
use tartan_ui_dioxus::{
    CardGrid, DashboardHeader, FeedbackBanner, LoadingState, MediaAspect, MediaFit, MediaPreview,
    MetricStrip, NavigationList, ProductShell, ResourceDashboard, TagList, ThemeToggle,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut preference = use_signal(|| ThemePreference::Dark);
    let navigation = vec![
        NavigationLink {
            key: "overview".to_string(),
            label: "Overview".to_string(),
            href: "#overview".to_string(),
            current: true,
        },
        NavigationLink {
            key: "resources".to_string(),
            label: "Resources".to_string(),
            href: "#resources".to_string(),
            current: false,
        },
        NavigationLink {
            key: "settings".to_string(),
            label: "Settings".to_string(),
            href: "#settings".to_string(),
            current: false,
        },
    ];
    let resources = vec![
        AccessibleResource {
            id: "atlas".to_string(),
            name: "Atlas workspace".to_string(),
            href: "#atlas".to_string(),
        },
        AccessibleResource {
            id: "field-notes".to_string(),
            name: "Field notes".to_string(),
            href: "#field-notes".to_string(),
        },
        AccessibleResource {
            id: "release-review".to_string(),
            name: "Release review".to_string(),
            href: "#release-review".to_string(),
        },
    ];
    let summaries = vec![
        (
            "atlas".to_string(),
            ResourceSummary {
                total: 12,
                complete: 8,
                secondary: 2,
                remaining: 2,
            },
        ),
        (
            "field-notes".to_string(),
            ResourceSummary {
                total: 7,
                complete: 4,
                secondary: 1,
                remaining: 2,
            },
        ),
    ];

    rsx! {
        ProductShell {
            title: "Tartan UI component gallery".to_string(),
            brand: "Tartan UI".to_string(),
            home_href: "#overview".to_string(),
            identity: Some(Identity {
                display_name: "Can".to_string(),
                email: Some("can@example.test".to_string()),
                account_url: None,
            }),
            div { class: "tartan-gallery__nav",
                NavigationList { items: navigation, aria_label: "Gallery navigation".to_string() }
                ThemeToggle {
                    preference: preference(),
                    on_toggle: move |next| preference.set(next),
                }
            }
            div { id: "overview", class: "tartan-gallery",
                DashboardHeader {
                    eyebrow: "Shared primitives".to_string(),
                    heading: "A small surface with real data contracts".to_string(),
                    description: "This fixture renders the Dioxus components directly so layout, states, and accessibility stay visible to the rubric runner.".to_string(),
                }
                FeedbackBanner {
                    feedback: Feedback {
                        kind: FeedbackKind::Success,
                        message: "All three component groups are ready for review.".to_string(),
                    }
                }
                MetricStrip {
                    metrics: vec![
                        Metric { label: "Components".to_string(), value: "16".to_string(), description: Some("shared".to_string()) },
                        Metric { label: "States".to_string(), value: "4".to_string(), description: Some("covered".to_string()) },
                        Metric { label: "A11y".to_string(), value: "100%".to_string(), description: Some("named".to_string()) },
                    ]
                }
                section { id: "resources", class: "tartan-gallery__section",
                    ResourceDashboard {
                        eyebrow: "Accessible resources".to_string(),
                        heading: "Choose a workspace".to_string(),
                        description: "Resource cards keep authorization data and progress summaries explicit.".to_string(),
                        resources,
                        summaries,
                    }
                }
                section { class: "tartan-gallery__section",
                    h2 { "Supporting states" }
                    CardGrid {
                        article { class: "tartan-card",
                            h3 { "Media preview" }
                            MediaPreview {
                                src: None,
                                alt: "No preview available".to_string(),
                                unavailable_label: "Preview will appear after processing.".to_string(),
                                fit: MediaFit::Contain,
                                aspect: MediaAspect::FourThree,
                            }
                        }
                        article { class: "tartan-card",
                            h3 { "Tags and loading" }
                            TagList { tags: vec!["web".to_string(), "native".to_string(), "accessible".to_string()] }
                            LoadingState { label: "Refreshing summary".to_string() }
                        }
                    }
                }
            }
        }
    }
}
