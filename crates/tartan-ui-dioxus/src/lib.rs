//! Shared, product-neutral Dioxus presentation components.

// Dioxus expands interpolated RSX attributes/text into `format!` calls. The
// interpolation is the renderer's supported dynamic-binding syntax, so this
// lint is a false positive for the component crate rather than an actionable
// allocation simplification.
#![allow(clippy::useless_format)]

pub mod icons;

use dioxus::prelude::*;
use tartan_ui_core::{
    AccessibleResource, FacetGroup, Feedback, FeedbackKind, FilterChoice, FilterOption, Identity,
    Metric, NavigationLink, Progress, ResourceLink, ResourceSummary, Suggestion, ThemePreference,
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
            style { dangerous_inner_html: tartan_ui_assets::LAYOUT_STYLES }
        }
    }

    #[cfg(not(feature = "native-embedded"))]
    {
        rsx! {
            document::Stylesheet { href: SHARED_STYLES }
            style { dangerous_inner_html: tartan_ui_assets::LAYOUT_STYLES }
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

/// `localStorage` key holding the persisted [`ThemePreference`].
pub const THEME_STORAGE_KEY: &str = "tartan-theme";

/// Inline `<head>` script that restores a persisted light/dark choice before
/// first paint. Optional: [`ThemeToggle`] restores on mount without it.
pub const THEME_INIT_SCRIPT: &str = "try{var t=localStorage.getItem('tartan-theme');\
if(t==='light'||t==='dark')document.documentElement.setAttribute('data-theme',t)}catch(e){}";

const THEME_RESTORE_SCRIPT: &str =
    "try{dioxus.send(localStorage.getItem('tartan-theme')||'')}catch(e){dioxus.send('')}";

/// JavaScript that writes `preference` to `<html data-theme>` (removed for
/// `System`, so `prefers-color-scheme` applies) and persists it.
pub fn theme_apply_script(preference: ThemePreference) -> String {
    let value = preference.as_str();
    let attribute = match preference {
        ThemePreference::System => "r.removeAttribute('data-theme')".to_string(),
        _ => format!("r.setAttribute('data-theme','{value}')"),
    };
    format!(
        "try{{var r=document.documentElement;{attribute};\
         localStorage.setItem('{THEME_STORAGE_KEY}','{value}')}}catch(e){{}}"
    )
}

fn theme_next_label(next: ThemePreference) -> &'static str {
    match next {
        ThemePreference::System => "Switch to system theme",
        ThemePreference::Light => "Switch to light theme",
        ThemePreference::Dark => "Switch to dark theme",
    }
}

/// Cycles system, light and dark. A click writes `data-theme` on `<html>`,
/// persists the choice in `localStorage` (`tartan-theme`) and calls
/// `on_toggle`. Without `preference` the toggle owns its state and restores the
/// persisted value on mount (client only; SSR renders `System`). With
/// `preference` the host controls the displayed value. The icon shows the
/// current preference; the accessible name announces the next one.
#[component]
pub fn ThemeToggle(
    #[props(default)] preference: Option<ThemePreference>,
    #[props(default)] on_toggle: Option<EventHandler<ThemePreference>>,
) -> Element {
    let mut stored = use_signal(|| preference.unwrap_or_default());
    // Tracked on `preference`: a host-driven change must reach the document and
    // persistence, not only the rendered control.
    use_effect(use_reactive!(|preference| {
        spawn(async move {
            if let Some(preference) = preference {
                let _ = document::eval(&theme_apply_script(preference));
            } else if let Ok(value) = document::eval(THEME_RESTORE_SCRIPT).recv::<String>().await {
                if let Some(restored) = ThemePreference::parse(&value) {
                    stored.set(restored);
                    // Without THEME_INIT_SCRIPT the document has no `data-theme`
                    // yet; restoring the control alone leaves it unthemed.
                    let _ = document::eval(&theme_apply_script(restored));
                }
            }
        });
    }));
    let current = preference.unwrap_or_else(|| *stored.read());
    let next = current.next();
    rsx! {
        button {
            class: "tartan-theme-toggle",
            type: "button",
            aria_label: theme_next_label(next),
            title: theme_next_label(next),
            "data-theme-preference": current.as_str(),
            onclick: move |_| {
                stored.set(next);
                let _ = document::eval(&theme_apply_script(next));
                if let Some(on_toggle) = on_toggle {
                    on_toggle.call(next);
                }
            },
            match current {
                ThemePreference::System => rsx! { icons::Monitor {} },
                ThemePreference::Light => rsx! { icons::Sun {} },
                ThemePreference::Dark => rsx! { icons::Moon {} },
            }
        }
    }
}

/// Settings-page form of [`ThemeToggle`]: a radio group for system, light and
/// dark that applies and persists the choice the same way and restores it on
/// mount (SSR renders `System` checked). As with [`ThemeToggle`], a
/// `preference` lets the host control the checked option, for example to keep
/// it in step with a header toggle. `labels` are `[system, light, dark]`
/// and default to English; each option carries `data-theme-preference` for
/// host swatches.
#[component]
pub fn ThemeChoice(
    #[props(default = "Theme".to_string())] legend: String,
    #[props(default)] preference: Option<ThemePreference>,
    #[props(default)] labels: Option<[String; 3]>,
    #[props(default)] on_change: Option<EventHandler<ThemePreference>>,
) -> Element {
    let mut stored = use_signal(|| preference.unwrap_or_default());
    // Tracked on `preference`, like the toggle: a host-driven change reaches
    // the document and persistence.
    use_effect(use_reactive!(|preference| {
        spawn(async move {
            if let Some(preference) = preference {
                let _ = document::eval(&theme_apply_script(preference));
            } else if let Ok(value) = document::eval(THEME_RESTORE_SCRIPT).recv::<String>().await {
                if let Some(restored) = ThemePreference::parse(&value) {
                    stored.set(restored);
                    // The checked radio alone does not theme the document on
                    // hosts without THEME_INIT_SCRIPT.
                    let _ = document::eval(&theme_apply_script(restored));
                }
            }
        });
    }));
    let labels = labels.unwrap_or_else(|| ["System", "Light", "Dark"].map(String::from));
    let current = preference.unwrap_or_else(|| *stored.read());
    let options = [
        ThemePreference::System,
        ThemePreference::Light,
        ThemePreference::Dark,
    ];
    rsx! {
        fieldset { class: "tartan-theme-choice",
            legend { "{legend}" }
            for (preference, label) in options.into_iter().zip(labels) {
                label {
                    key: "{preference.as_str()}",
                    class: "tartan-theme-choice__option",
                    "data-theme-preference": preference.as_str(),
                    input {
                        r#type: "radio",
                        name: THEME_STORAGE_KEY,
                        value: preference.as_str(),
                        checked: current == preference,
                        onchange: move |_| {
                            stored.set(preference);
                            let _ = document::eval(&theme_apply_script(preference));
                            if let Some(on_change) = on_change {
                                on_change.call(preference);
                            }
                        },
                    }
                    " {label}"
                }
            }
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

/// A native, keyboard-accessible navigation group. The host supplies links and
/// decides whether the current route should initially expand the disclosure.
#[component]
pub fn NavigationDisclosure(
    label: String,
    children: Element,
    #[props(default)] open: bool,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        details { class: "tartan-navigation-disclosure {class}", open,
            summary { "{label}" }
            {children}
        }
    }
}

/// A page heading with optional actions that wrap independently of its text.
#[component]
pub fn SectionHeader(
    heading: String,
    #[props(default)] heading_id: Option<String>,
    #[props(default)] eyebrow: Option<String>,
    #[props(default)] description: Option<String>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        header { class: "tartan-section-header {class}",
            div { class: "tartan-section-header__body",
                if let Some(eyebrow) = eyebrow { p { class: "tartan-eyebrow", "{eyebrow}" } }
                h1 { id: heading_id, "{heading}" }
                if let Some(description) = description { p { class: "tartan-muted", "{description}" } }
            }
            if let Some(actions) = actions { div { class: "tartan-action-group", {actions} } }
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
///
/// `title_prefix` adds a `title="{prefix}{value}"` tooltip to each option and
/// `counts_truncated` renders counts as lower bounds (`10000+`).
#[component]
pub fn CheckboxFilterOptions(
    name: String,
    options: Vec<FilterChoice>,
    #[props(default)] on_change: Option<EventHandler<(String, bool)>>,
    #[props(default)] class: Option<String>,
    #[props(default)] title_prefix: Option<String>,
    #[props(default)] counts_truncated: bool,
) -> Element {
    let class = class.unwrap_or_default();
    let plus = if counts_truncated { "+" } else { "" };
    rsx! {
        for option in options {
            label {
                key: "{option.value}",
                class: "{class}",
                title: title_prefix.as_ref().map(|prefix| format!("{prefix}{}", option.value)),
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
                if let Some(count) = option.count { " ({count}{plus})" }
            }
        }
    }
}

/// Sidebar editor for facet selections. Editable categories toggle through
/// `on_toggle((key, value, checked))`; the caller rewrites its query and
/// passes fresh groups back. Categories set in the query text are read-only.
#[component]
pub fn FacetSidebar(
    groups: Vec<FacetGroup>,
    aria_label: String,
    on_toggle: EventHandler<(String, String, bool)>,
    #[props(default)] class: Option<String>,
    /// Note under read-only categories; defaults to "Set in the query text".
    #[props(default)]
    read_only_note: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    let read_only_note = read_only_note.unwrap_or_else(|| "Set in the query text".to_string());
    rsx! {
        nav { class: "tartan-facet-sidebar {class}", aria_label: "{aria_label}",
            for group in groups {
                NavigationDisclosure {
                    key: "{group.title}",
                    label: group.title.clone(),
                    open: true,
                    class: "tartan-facet-group".to_string(),
                    for category in group.categories {
                        FacetCategoryFields { key: "{category.key}", category, on_toggle, read_only_note: read_only_note.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn FacetCategoryFields(
    category: tartan_ui_core::FacetCategory,
    on_toggle: EventHandler<(String, String, bool)>,
    read_only_note: String,
) -> Element {
    let key = category.key.clone();
    rsx! {
        fieldset { class: "tartan-facet-category",
            legend {
                "{category.label} "
                code { class: "tartan-facet-key", "{category.key}:" }
            }
            if category.editable {
                CheckboxFilterOptions {
                    name: category.key.clone(),
                    options: category.options.clone(),
                    title_prefix: format!("{key}:"),
                    counts_truncated: category.counts_truncated,
                    on_change: move |(value, checked): (String, bool)| {
                        on_toggle.call((key.clone(), value, checked));
                    },
                }
            } else {
                ul { class: "tartan-facet-values",
                    for option in category.options.iter().filter(|option| option.checked) {
                        li { key: "{option.value}", "{option.label}" }
                    }
                }
                p { class: "tartan-facet-note", "{read_only_note}" }
            }
        }
    }
}

/// What a keydown does in [`QueryCombobox`]; decided by [`key_action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    /// Accept the suggestion at this flat index.
    Pick(usize),
    Submit,
    /// Enter while the caller reports blocking errors: do not submit.
    Blocked,
    /// Move the highlight (arrows wrap).
    Move(Option<usize>),
    Close,
    PassThrough,
}

/// The highlighted row: the explicit one, else row 0 when `autoselect` and the
/// list is showing rows.
fn effective_active(
    active: Option<usize>,
    expanded: bool,
    count: usize,
    autoselect: bool,
) -> Option<usize> {
    active
        .or((autoselect && expanded && count > 0).then_some(0))
        .filter(|index| expanded && *index < count)
}

fn key_action(
    key: &str,
    active: Option<usize>,
    expanded: bool,
    count: usize,
    autoselect: bool,
    blocking: bool,
) -> KeyAction {
    // Tab completes only an option the user highlighted with the arrow keys.
    // Autoselected options never capture Tab: the popup reopens with fresh
    // suggestions after every pick, so a keyboard user could not leave the box.
    let highlighted = active.filter(|index| expanded && *index < count);
    let active = effective_active(active, expanded, count, autoselect);
    match key {
        "ArrowDown" if count > 0 => {
            KeyAction::Move(Some(active.map_or(0, |index| (index + 1) % count)))
        }
        "ArrowUp" if count > 0 => KeyAction::Move(Some(
            active.map_or(count - 1, |index| (index + count - 1) % count),
        )),
        "Enter" => match active {
            Some(index) => KeyAction::Pick(index),
            None if blocking => KeyAction::Blocked,
            None => KeyAction::Submit,
        },
        "Tab" => highlighted.map_or(KeyAction::PassThrough, KeyAction::Pick),
        "Escape" => KeyAction::Close,
        _ => KeyAction::PassThrough,
    }
}

/// UTF-16 code-unit index (`selectionStart`) to a byte offset into `value`,
/// clamped to the end. An index inside a surrogate pair rounds up.
fn utf16_to_byte(value: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in value.char_indices() {
        if units >= utf16 {
            return byte;
        }
        units += ch.len_utf16();
    }
    value.len()
}

/// Caret (byte offset into `new`) after one contiguous edit of `old`: the end
/// of the changed region. Typing, deleting and pasting are single edits.
///
/// This is the fallback for environments without a DOM selection: repeated
/// text makes the changed region ambiguous (inserting `a` at the start of `aa`
/// guesses the end), so a browser passes the input's own `selectionStart`.
fn caret_after_edit(old: &str, new: &str) -> usize {
    let prefix: usize = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    let suffix: usize = old[prefix..]
        .chars()
        .rev()
        .zip(new[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    new.len() - suffix
}

/// Byte offset into `value` to a UTF-16 index (`setSelectionRange`). An offset
/// inside a character rounds up past it.
fn byte_to_utf16(value: &str, byte: usize) -> usize {
    value
        .char_indices()
        .take_while(|(index, _)| *index < byte)
        .map(|(_, ch)| ch.len_utf16())
        .sum()
}

/// The caret (byte offset into `value`) a `value`-replacing edit leaves behind.
/// The element's own `selection_start` (UTF-16 units) is authoritative; the
/// prefix/suffix comparison is the fallback outside a browser.
fn caret_for_input(old: &str, value: &str, selection_start: Option<usize>) -> usize {
    match selection_start {
        Some(units) => utf16_to_byte(value, units),
        None => caret_after_edit(old, value),
    }
}

/// The focused input's `selectionStart`, read synchronously so the caret can
/// travel with the same event as the value. `None` off the web or before the
/// DOM node exists.
#[cfg(target_arch = "wasm32")]
fn input_selection_start(id: &str) -> Option<usize> {
    use web_sys::wasm_bindgen::JsCast;
    let document = web_sys::window()?.document()?;
    let input = document
        .get_element_by_id(id)?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()?;
    input
        .selection_start()
        .ok()
        .flatten()
        .map(|units| units as usize)
}

#[cfg(not(target_arch = "wasm32"))]
fn input_selection_start(_id: &str) -> Option<usize> {
    None
}

/// Split `value` into `(text, kind)` runs for the highlight mirror. Marks that
/// are empty, out of range, off a char boundary or overlap an earlier mark are
/// ignored; gaps come back with no kind.
fn mark_segments<'a>(
    value: &'a str,
    marks: &'a [(usize, usize, String)],
) -> Vec<(&'a str, Option<&'a str>)> {
    let mut sorted: Vec<_> = marks.iter().collect();
    sorted.sort_by_key(|mark| (mark.0, mark.1));
    let mut out = Vec::new();
    let mut at = 0;
    for (start, end, kind) in sorted {
        let valid = at <= *start
            && start < end
            && *end <= value.len()
            && value.is_char_boundary(*start)
            && value.is_char_boundary(*end);
        if !valid {
            continue;
        }
        if at < *start {
            out.push((&value[at..*start], None));
        }
        out.push((&value[*start..*end], Some(kind.as_str())));
        at = *end;
    }
    if at < value.len() {
        out.push((&value[at..], None));
    }
    out
}

/// Caret byte offset of the input `id`. Falls back to the end of `value` when
/// the document cannot evaluate script (SSR, native) or the element is gone.
async fn read_caret(id: &str, value: &str) -> usize {
    let js = format!(
        "var e=document.getElementById({id:?});return e&&e.selectionStart!=null?e.selectionStart:-1;"
    );
    match document::eval(&js).join::<i64>().await {
        Ok(utf16) if utf16 >= 0 => utf16_to_byte(value, utf16 as usize),
        _ => value.len(),
    }
}

/// Optionally move the caret of input `id` (UTF-16 index), then copy its
/// horizontal scroll to the highlight mirror.
fn sync_input(id: &str, caret_utf16: Option<usize>) {
    let select = caret_utf16.map_or(String::new(), |at| {
        format!("i.setSelectionRange({at},{at});")
    });
    document::eval(&format!(
        "var i=document.getElementById({id:?});if(i){{{select}var m=document.getElementById({id:?}+'-mirror');if(m)m.scrollLeft=i.scrollLeft;}}"
    ));
}

/// Generic ARIA combobox for query text. The caller owns the value, the
/// suggestions, validation and any fetching or debouncing; this component
/// handles open state, the highlighted row, keyboard interaction, caret
/// reporting and the optional highlight mirror.
///
/// Errors never hide the list: they render under the field as a status hint,
/// and become an alert when Enter is pressed while `blocking`. Enter picks the
/// highlighted row (row 0 under `autoselect`), otherwise submits; Tab accepts
/// only a row the user highlighted with the arrow keys and otherwise moves
/// focus on, so the box is never a keyboard trap.
///
/// A pick keeps the list open, so a completion can lead straight into the
/// next step; a pick that navigates or submits replaces the page anyway.
#[component]
pub fn QueryCombobox(
    id: String,
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] hint: Option<String>,
    suggestions: Vec<Suggestion>,
    #[props(default)] errors: Vec<String>,
    /// Errors forbid submitting: Enter does not call `on_submit`.
    #[props(default)]
    blocking: bool,
    /// Enter completes row 0 while the list shows rows and nothing is
    /// highlighted (the token under the caret is a `key:prefix`).
    #[props(default)]
    autoselect: bool,
    /// `(byte range, kind)` runs painted behind a transparent-text input.
    #[props(default)]
    marks: Vec<(usize, usize, String)>,
    /// Byte offset to place the caret at whenever this value changes.
    #[props(default)]
    caret: Option<usize>,
    #[props(default)] footer: Option<Element>,
    /// New value and the caret as a byte offset into it.
    on_input: EventHandler<(String, usize)>,
    /// Caret moved by click, ArrowLeft/Right, Home or End (byte offset).
    #[props(default)]
    on_caret: Option<EventHandler<usize>>,
    on_pick: EventHandler<Suggestion>,
    on_submit: EventHandler<String>,
    #[props(default)] on_focus: Option<EventHandler<()>>,
) -> Element {
    let mut open = use_signal(|| false);
    // `None` until arrow keys: typing or focusing and pressing Enter submits
    // instead of picking a suggestion (unless `autoselect`).
    let mut active = use_signal(|| None::<usize>);
    // Set by a blocked Enter so screen readers announce the errors.
    let mut alerting = use_signal(|| false);
    let count = suggestions.len();
    let has_errors = !errors.is_empty();
    let expanded = open() && count > 0;
    let active_index = effective_active(active(), expanded, count, autoselect);
    let keydown_suggestions = suggestions.clone();
    let keydown_value = value.clone();
    let caret_value = value.clone();
    let click_value = value.clone();
    let input_id = id.clone();
    let input_value = value.clone();
    let selection_id = id.clone();
    let keyup_id = id.clone();
    let click_id = id.clone();
    let scroll_id = id.clone();
    let effect_id = id.clone();
    let has_marks = !marks.is_empty();
    // The hint and current errors describe the input, so focusing it
    // announces them, not only the live region when they change.
    let described_by = [
        hint.as_ref().map(|_| format!("{id}-hint")),
        has_errors.then(|| format!("{id}-errors")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    // `Cell` so the effect only moves the caret when `caret` itself changed,
    // not on every keystroke while the caller still holds the old offset.
    let applied_caret = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(None::<usize>)));
    use_effect(use_reactive!(|caret, value, has_marks| {
        let moved = applied_caret.replace(caret) != caret;
        let target = caret
            .filter(|_| moved)
            .map(|byte| byte_to_utf16(&value, byte));
        if target.is_some() || has_marks {
            sync_input(&effect_id, target);
        }
    }));
    // A heading precedes the first row of each consecutive run of one group.
    let mut previous: Option<String> = None;
    let rows: Vec<(Option<String>, usize, Suggestion)> = suggestions
        .into_iter()
        .enumerate()
        .map(|(index, suggestion)| {
            let heading = if suggestion.group != previous {
                suggestion.group.clone()
            } else {
                None
            };
            previous = suggestion.group.clone();
            (heading, index, suggestion)
        })
        .collect();
    rsx! {
        div { class: "tartan-combobox",
            label { class: "tartan-field__label", r#for: "{id}", "{label}" }
            div { class: "tartan-combobox-field",
                if has_marks {
                    div { id: "{id}-mirror", class: "tartan-combobox-mirror", aria_hidden: "true",
                        for (index, (text, kind)) in mark_segments(&value, &marks).into_iter().enumerate() {
                            if let Some(kind) = kind {
                                span { key: "{index}", class: "tartan-mark tartan-mark--{kind}", "{text}" }
                            } else {
                                span { key: "{index}", "{text}" }
                            }
                        }
                    }
                }
                input {
                    id: "{id}",
                    class: if has_marks { "tartan-combobox-input has-marks" } else { "tartan-combobox-input" },
                    r#type: "text",
                    enterkeyhint: "search",
                    placeholder: "{placeholder}",
                    value: "{value}",
                    autocomplete: "off",
                    spellcheck: "false",
                    role: "combobox",
                    aria_expanded: expanded.to_string(),
                    aria_controls: "{id}-listbox",
                    aria_autocomplete: "list",
                    aria_activedescendant: active_index.map(|index| format!("{id}-option-{index}")),
                    aria_invalid: has_errors.to_string(),
                    aria_describedby: (!described_by.is_empty()).then_some(described_by),
                    aria_errormessage: has_errors.then(|| format!("{id}-errors")),
                    oninput: move |event| {
                        active.set(None);
                        alerting.set(false);
                        open.set(true);
                        // Synchronous: an async report would let a re-render write
                        // the stale value back, which moves the caret to the end.
                        let value = event.value();
                        let caret = caret_for_input(
                            &input_value,
                            &value,
                            input_selection_start(&selection_id),
                        );
                        on_input.call((value, caret));
                        sync_input(&input_id, None);
                    },
                    onfocus: move |_| {
                        active.set(None);
                        open.set(true);
                        if let Some(on_focus) = on_focus {
                            on_focus.call(());
                        }
                    },
                    // Reopen after Escape without leaving the focused box.
                    onclick: move |_| {
                        open.set(true);
                        if let Some(on_caret) = on_caret {
                            let (id, value) = (click_id.clone(), click_value.clone());
                            spawn(async move { on_caret.call(read_caret(&id, &value).await) });
                        }
                    },
                    onscroll: move |_| sync_input(&scroll_id, None),
                    onblur: move |_| open.set(false),
                    onkeyup: move |event| {
                        let key = event.key().to_string();
                        if matches!(key.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                            if let Some(on_caret) = on_caret {
                                let (id, value) = (keyup_id.clone(), caret_value.clone());
                                spawn(async move { on_caret.call(read_caret(&id, &value).await) });
                            }
                        }
                    },
                    onkeydown: move |event| {
                        let key = event.key().to_string();
                        match key_action(&key, active(), expanded, count, autoselect, blocking && has_errors) {
                            KeyAction::Move(next) => {
                                event.prevent_default();
                                active.set(next);
                                open.set(true);
                            }
                            KeyAction::Pick(index) => {
                                event.prevent_default();
                                active.set(None);
                                on_pick.call(keydown_suggestions[index].clone());
                            }
                            KeyAction::Submit => {
                                event.prevent_default();
                                on_submit.call(keydown_value.clone());
                            }
                            KeyAction::Blocked => {
                                event.prevent_default();
                                alerting.set(true);
                            }
                            KeyAction::Close => open.set(false),
                            KeyAction::PassThrough => {}
                        }
                    },
                }
            }
            if let Some(hint) = hint { p { id: "{id}-hint", class: "tartan-field__hint", "{hint}" } }
            if has_errors {
                div {
                    id: "{id}-errors",
                    class: "tartan-combobox-errors",
                    role: if alerting() { "alert" } else { "status" },
                    for error in errors {
                        p { key: "{error}", "{error}" }
                    }
                }
            }
            if count > 0 {
                // mousedown keeps focus in the input, so blur cannot close the
                // popup before a pick or a footer click lands.
                div {
                    class: "tartan-combobox-popup",
                    hidden: !expanded,
                    onmousedown: move |event| event.prevent_default(),
                    ul {
                        id: "{id}-listbox",
                        class: "tartan-combobox-list",
                        role: "listbox",
                        // A scrollable list is keyboard-focusable in Chromium;
                        // options are reached through aria-activedescendant, and
                        // focusing the list would blur the input and close it.
                        tabindex: "-1",
                        aria_label: "{label} suggestions",
                        for (heading, index, suggestion) in rows {
                            Fragment { key: "{index}:{suggestion.label}",
                            if let Some(group) = heading {
                                li { role: "presentation", class: "tartan-combobox-group", "{group}" }
                            }
                            li {
                                id: "{id}-option-{index}",
                                role: "option",
                                aria_selected: (Some(index) == active_index).to_string(),
                                class: if Some(index) == active_index { "tartan-combobox-option is-active" } else { "tartan-combobox-option" },
                                onmousedown: {
                                    let suggestion = suggestion.clone();
                                    move |event| {
                                        event.prevent_default();
                                        active.set(None);
                                        on_pick.call(suggestion.clone());
                                    }
                                },
                                strong { "{suggestion.label}" }
                                if let Some(detail) = &suggestion.detail { span { class: "tartan-muted", " {detail}" } }
                            }
                            }
                        }
                    }
                    if expanded {
                        if let Some(footer) = footer {
                            div { class: "tartan-combobox-footer", {footer} }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_after_edit_lands_after_the_change() {
        // Typing in the middle, at the end, deleting, pasting, multibyte.
        assert_eq!(caret_after_edit("type: area", "type:u area"), 6);
        assert_eq!(caret_after_edit("type:", "type:p"), 6);
        assert_eq!(caret_after_edit("aXa", "aa"), 1);
        assert_eq!(caret_after_edit("a b", "a req: b"), 7);
        assert_eq!(caret_after_edit("m²", "m²x"), "m²x".len());
        assert_eq!(caret_after_edit("", ""), 0);
    }

    #[test]
    fn the_inputs_own_selection_beats_the_repeated_text_guess() {
        // Inserting `a` at the start of `aa` is ambiguous: prefix/suffix
        // comparison guesses the end, the element reports 1.
        assert_eq!(caret_after_edit("aa", "aaa"), 3);
        assert_eq!(caret_for_input("aa", "aaa", Some(1)), 1);
        assert_eq!(caret_for_input("aa", "aaa", None), 3);
        // UTF-16 selection indexes become byte offsets; one inside a surrogate
        // pair rounds up; one past the end clamps.
        assert_eq!(caret_for_input("", "m²", Some(1)), 1);
        assert_eq!(caret_for_input("", "m²", Some(2)), "m²".len());
        assert_eq!(caret_for_input("", "ab", Some(9)), 2);
        // The ordinary cases agree with either source.
        assert_eq!(caret_for_input("type:", "type:p", Some(6)), 6);
        assert_eq!(caret_for_input("aXa", "aa", Some(1)), 1);
    }

    #[test]
    fn key_action_enter_submits_unless_autoselect_or_highlighted() {
        use KeyAction::*;
        assert_eq!(key_action("Enter", None, true, 3, false, false), Submit);
        assert_eq!(key_action("Enter", None, true, 3, true, false), Pick(0));
        assert_eq!(key_action("Enter", Some(2), true, 3, false, false), Pick(2));
        // Autoselect does nothing without rows or while collapsed.
        assert_eq!(key_action("Enter", None, true, 0, true, false), Submit);
        assert_eq!(key_action("Enter", None, false, 3, true, false), Submit);
        // A stale highlight past the end of a shrunken list is ignored.
        assert_eq!(key_action("Enter", Some(5), true, 3, false, false), Submit);
    }

    #[test]
    fn key_action_enter_while_blocking_never_submits() {
        use KeyAction::*;
        assert_eq!(key_action("Enter", None, true, 3, false, true), Blocked);
        assert_eq!(key_action("Enter", None, false, 0, false, true), Blocked);
        // Completing a value is still allowed.
        assert_eq!(key_action("Enter", None, true, 3, true, true), Pick(0));
    }

    #[test]
    fn key_action_tab_accepts_only_a_highlighted_row() {
        use KeyAction::*;
        assert_eq!(key_action("Tab", None, true, 3, false, false), PassThrough);
        assert_eq!(key_action("Tab", None, true, 3, true, false), PassThrough);
        assert_eq!(key_action("Tab", Some(1), true, 3, false, false), Pick(1));
        assert_eq!(key_action("Tab", None, true, 0, false, false), PassThrough);
        assert_eq!(key_action("Tab", None, false, 3, false, false), PassThrough);
    }

    #[test]
    fn key_action_arrows_wrap_and_escape_closes() {
        use KeyAction::*;
        assert_eq!(
            key_action("ArrowDown", None, true, 3, false, false),
            Move(Some(0))
        );
        assert_eq!(
            key_action("ArrowDown", Some(2), true, 3, false, false),
            Move(Some(0))
        );
        assert_eq!(
            key_action("ArrowUp", None, true, 3, false, false),
            Move(Some(2))
        );
        assert_eq!(
            key_action("ArrowUp", Some(0), true, 3, false, false),
            Move(Some(2))
        );
        // Autoselect's implicit row 0 is the starting point.
        assert_eq!(
            key_action("ArrowDown", None, true, 3, true, false),
            Move(Some(1))
        );
        assert_eq!(
            key_action("ArrowDown", None, true, 0, false, false),
            PassThrough
        );
        assert_eq!(key_action("Escape", None, true, 3, false, false), Close);
        assert_eq!(key_action("a", None, true, 3, false, false), PassThrough);
    }

    #[test]
    fn caret_offsets_convert_between_utf16_and_bytes() {
        // 'é' is 2 bytes / 1 unit; '😀' is 4 bytes / 2 units.
        let value = "aé😀b";
        assert_eq!(utf16_to_byte(value, 0), 0);
        assert_eq!(utf16_to_byte(value, 2), 3);
        assert_eq!(utf16_to_byte(value, 4), 7);
        assert_eq!(utf16_to_byte(value, 5), 8);
        assert_eq!(utf16_to_byte(value, 99), value.len());
        assert_eq!(byte_to_utf16(value, 3), 2);
        assert_eq!(byte_to_utf16(value, 7), 4);
        assert_eq!(byte_to_utf16(value, 99), 5);
        for byte in value.char_indices().map(|(i, _)| i).chain([value.len()]) {
            assert_eq!(utf16_to_byte(value, byte_to_utf16(value, byte)), byte);
        }
    }

    #[test]
    fn mark_segments_cover_value_and_skip_invalid_marks() {
        let marks = |list: &[(usize, usize, &str)]| -> Vec<(usize, usize, String)> {
            list.iter()
                .map(|(a, b, k)| (*a, *b, k.to_string()))
                .collect()
        };
        let value = "type:é x";
        assert_eq!(
            mark_segments(value, &marks(&[(5, 7, "value"), (0, 4, "key")])),
            vec![
                ("type", Some("key")),
                (":", None),
                ("é", Some("value")),
                (" x", None)
            ]
        );
        // Out of range, inside 'é', empty, reversed and overlapping marks vanish.
        let bad = marks(&[
            (0, 99, "key"),
            (6, 7, "value"),
            (2, 2, "op"),
            (3, 1, "op"),
            (0, 4, "key"),
            (2, 4, "error"),
        ]);
        assert_eq!(
            mark_segments(value, &bad),
            vec![("type", Some("key")), (":é x", None)]
        );
        assert_eq!(mark_segments("", &[]), vec![]);
    }

    #[test]
    fn query_combobox_renders_groups_marks_footer_and_status_errors() {
        #[component]
        fn Fixture() -> Element {
            let row = |label: &str, group: Option<&str>| Suggestion {
                insert: label.to_string(),
                label: label.to_string(),
                group: group.map(str::to_string),
                ..Default::default()
            };
            rsx! {
                QueryCombobox {
                    id: "q".to_string(), label: "Search".to_string(), value: "type:".to_string(),
                    suggestions: vec![row("a", Some("G1")), row("b", Some("G1")), row("c", Some("G2")), row("d", None)],
                    errors: vec!["needs a value".to_string()],
                    marks: vec![(0, 4, "key".to_string()), (4, 5, "op".to_string())],
                    footer: rsx! { span { "Enter to search" } },
                    on_input: |_| {}, on_pick: |_| {}, on_submit: |_| {},
                }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert_eq!(html.matches("tartan-combobox-group").count(), 2, "{html}");
        assert_eq!(html.matches("role=\"presentation\"").count(), 2);
        assert_eq!(html.matches("role=\"option\"").count(), 4);
        assert!(html.contains("id=\"q-option-3\""));
        assert!(
            html.contains("tartan-mark tartan-mark--key") && html.contains("tartan-mark--op"),
            "{html}"
        );
        assert!(html.contains("id=\"q-mirror\"") && html.contains("aria-hidden=\"true\""));
        // Tab leaves the input for the next control, never the scrollable list.
        assert!(html.contains("role=\"listbox\" tabindex=\"-1\""), "{html}");
        // Errors no longer hide or alert by themselves.
        assert!(html.contains("role=\"status\"") && !html.contains("role=\"alert\""));
        // Collapsed until opened, so the footer is not rendered yet.
        assert!(!html.contains("Enter to search"));
    }

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
    fn section_header_composes_wrapping_actions_and_escaped_content() {
        let html = dioxus_ssr::render_element(rsx! {
            SectionHeader {
                heading: "Plans <private>".to_string(),
                description: "Authorized records".to_string(),
                class: "product-heading".to_string(),
                actions: rsx! { a { href: "/plans/new", "New plan" } },
            }
        });
        assert!(html.contains("tartan-section-header product-heading"));
        assert!(html.contains("<h1>Plans &#60;private&#62;</h1>"));
        assert!(html.contains("tartan-action-group"));
        assert!(html.contains("href=\"/plans/new\""));
    }

    #[test]
    fn navigation_disclosure_keeps_host_controlled_open_state() {
        let html = dioxus_ssr::render_element(rsx! {
            NavigationDisclosure {
                label: "Discovery tools".to_string(), open: true,
                class: "product-nav-group".to_string(),
                NavigationList {
                    items: vec![NavigationLink { key: "atlas".into(), label: "Atlas".into(), href: "/atlas".into(), current: true }],
                    aria_label: "Discovery tools".to_string(),
                }
            }
        });
        assert!(html.contains("tartan-navigation-disclosure product-nav-group"));
        assert!(html.contains("open"));
        assert!(html.contains("<summary>Discovery tools</summary>"));
        assert!(html.contains("aria-current=\"page\""));
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

    fn facet_fixture() -> Vec<FacetGroup> {
        let option = |value: &str, count, checked| FilterChoice {
            value: value.to_string(),
            label: value.to_uppercase(),
            count,
            checked,
        };
        vec![FacetGroup {
            title: "Provenance".to_string(),
            categories: vec![
                tartan_ui_core::FacetCategory {
                    key: "period".to_string(),
                    label: "Period".to_string(),
                    options: vec![
                        option("1900", Some(10000), true),
                        option("1950", None, false),
                    ],
                    editable: true,
                    counts_truncated: true,
                },
                tartan_ui_core::FacetCategory {
                    key: "kind".to_string(),
                    label: "Kind".to_string(),
                    options: vec![option("plan", Some(2), true), option("map", Some(1), false)],
                    editable: false,
                    counts_truncated: false,
                },
            ],
        }]
    }

    #[test]
    fn facet_sidebar_renders_keys_titles_and_read_only_categories() {
        #[component]
        fn Fixture() -> Element {
            rsx! { FacetSidebar { groups: facet_fixture(), aria_label: "Filters".to_string(), on_toggle: |_| {} } }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert!(html.contains("aria-label=\"Filters\""), "{html}");
        assert!(html.contains("class=\"tartan-facet-category\""));
        assert!(html.contains("period:</code>"), "{html}");
        assert!(html.contains("kind:</code>"));
        assert!(html.contains("title=\"period:1900\""));
        assert!(html.contains("value=\"1900\" checked"));
        assert!(html.contains("1900 (10000+)"), "{html}");
        // Read-only category: plain text for checked values, no checkbox.
        assert!(html.contains("Set in the query text"));
        assert!(html.contains("<li>PLAN</li>"), "{html}");
        #[component]
        fn Translated() -> Element {
            rsx! { FacetSidebar { groups: facet_fixture(), aria_label: "Filter".to_string(), on_toggle: |_| {}, read_only_note: "In der Abfrage gesetzt".to_string() } }
        }
        let translated = dioxus_ssr::render_element(rsx! { Translated {} });
        assert!(
            translated.contains("In der Abfrage gesetzt")
                && !translated.contains("Set in the query text")
        );
        assert!(!html.contains("MAP"));
        assert!(!html.contains("name=\"kind\""));
    }

    #[test]
    fn query_combobox_exposes_combobox_listbox_and_errors() {
        #[component]
        fn Fixture() -> Element {
            rsx! {
                        QueryCombobox {
                            id: "q".to_string(), label: "Search".to_string(), value: "per".to_string(),
                            hint: "Try period:1900".to_string(),
                            suggestions: vec![
                                Suggestion { insert: "period:".to_string(), label: "period:".to_string(), detail: Some("Period".to_string()), group: None },
                                Suggestion { insert: "plan".to_string(), label: "plan".to_string(), detail: None, group: None },
                            ],
                            errors: vec!["Unclosed quote".to_string()],
                            on_input: |_| {}, on_pick: |_| {}, on_submit: |_| {},
                        }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("aria-controls=\"q-listbox\""));
        assert!(html.contains("aria-autocomplete=\"list\""));
        assert!(html.contains("for=\"q\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(html.contains("id=\"q-option-0\"") && html.contains("id=\"q-option-1\""));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Unclosed quote"));
        assert!(html.contains("Try period:1900"));
        assert!(
            html.contains("aria-describedby=\"q-hint q-errors\""),
            "{html}"
        );
        assert!(html.contains("aria-errormessage=\"q-errors\""));
        assert!(html.contains("id=\"q-hint\"") && html.contains("id=\"q-errors\""));
    }

    #[test]
    fn query_combobox_without_errors_or_suggestions_has_no_alert_or_listbox() {
        #[component]
        fn Fixture() -> Element {
            rsx! {
                        QueryCombobox {
                            id: "q".to_string(), label: "Search".to_string(), value: String::new(),
                            suggestions: vec![],
                            on_input: |_| {}, on_pick: |_| {}, on_submit: |_| {},
                        }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture {} });
        assert!(html.contains("role=\"combobox\""));
        assert!(!html.contains("role=\"alert\""));
        assert!(!html.contains("role=\"listbox\""));
        assert!(!html.contains("aria-describedby") && !html.contains("aria-errormessage"));
    }

    fn render_toggle(preference: Option<ThemePreference>) -> String {
        #[component]
        fn Fixture(preference: Option<ThemePreference>) -> Element {
            rsx! { ThemeToggle { preference } }
        }
        dioxus_ssr::render_element(rsx! { Fixture { preference } })
    }

    #[test]
    fn theme_toggle_names_the_next_state_and_shows_the_current_icon() {
        let cases = [
            (ThemePreference::System, "Switch to light theme"),
            (ThemePreference::Light, "Switch to dark theme"),
            (ThemePreference::Dark, "Switch to system theme"),
        ];
        for (current, label) in cases {
            let html = render_toggle(Some(current));
            assert!(html.contains(&format!("aria-label=\"{label}\"")), "{html}");
            assert!(html.contains(&format!("data-theme-preference=\"{}\"", current.as_str())));
            assert!(
                html.contains("<svg") && html.contains("aria-hidden=\"true\""),
                "{html}"
            );
        }
        // Uncontrolled SSR output starts at System (restored on the client).
        assert!(render_toggle(None).contains("Switch to light theme"));
    }

    #[test]
    fn theme_choice_offers_three_named_radios_with_translatable_labels() {
        #[component]
        fn Fixture(labels: Option<[String; 3]>) -> Element {
            rsx! { ThemeChoice { labels } }
        }
        let html = dioxus_ssr::render_element(rsx! { Fixture { labels: None } });
        assert_eq!(html.matches("type=\"radio\"").count(), 3, "{html}");
        assert_eq!(html.matches("name=\"tartan-theme\"").count(), 3);
        assert!(html.contains("<legend>Theme</legend>"), "{html}");
        assert!(html.contains("value=\"system\" checked"), "{html}");
        assert!(html.contains("data-theme-preference=\"dark\""));
        let labels = Some(["Système", "Clair", "Sombre"].map(String::from));
        let html = dioxus_ssr::render_element(rsx! { Fixture { labels } });
        assert!(
            html.contains(" Clair") && !html.contains(" Light"),
            "{html}"
        );
        #[component]
        fn Controlled() -> Element {
            rsx! { ThemeChoice { preference: ThemePreference::Dark } }
        }
        let html = dioxus_ssr::render_element(rsx! { Controlled {} });
        assert!(html.contains("value=\"dark\" checked"), "{html}");
        assert!(!html.contains("value=\"system\" checked"), "{html}");
    }

    #[test]
    fn theme_scripts_set_or_clear_the_attribute_and_persist() {
        let system = theme_apply_script(ThemePreference::System);
        assert!(system.contains("removeAttribute('data-theme')"), "{system}");
        assert!(system.contains("setItem('tartan-theme','system')"));
        let dark = theme_apply_script(ThemePreference::Dark);
        assert!(dark.contains("setAttribute('data-theme','dark')"), "{dark}");
        assert!(dark.contains("setItem('tartan-theme','dark')"));
        assert!(THEME_INIT_SCRIPT.contains("tartan-theme"));
    }

    const STYLESHEET: &str = include_str!("../assets/tartan-ui.css");

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
    fn stylesheet_aliases_consume_semantic_theme_roles() {
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
        assert!(!STYLESHEET.to_ascii_lowercase().contains("honey"));
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
    fn button_hover_and_active_use_accent_state_roles_without_affecting_disabled() {
        // `:where` keeps the disabled filter from adding specificity, and the
        // derived fallback resolves at the button so scoped accents apply.
        let mix = "color-mix(in srgb, var(--tartan-accent) 88%, var(--tartan-paper))";
        let hover =
            rule(".tartan-button:where(:not(:disabled):not([aria-disabled=\"true\"])):hover");
        assert!(hover.contains(&format!("var(--tartan-accent-hover, {mix})")));
        let active =
            rule(".tartan-button:where(:not(:disabled):not([aria-disabled=\"true\"])):active");
        assert!(active.contains("var(--tartan-accent-active, var(--tartan-accent-hover,"));
        // States only set the colour variable; a host's literal background wins.
        for state in [hover, active] {
            assert!(state.contains("--tartan-button-bg:"), "{state}");
            assert!(!state.contains("background:"), "{state}");
        }
        assert!(
            rule(".tartan-button {")
                .contains("background: var(--tartan-button-bg, var(--tartan-accent))")
        );
        assert_eq!(
            declaration("--tartan-accent-hover:").trim(),
            "--tartan-accent-hover: var(--tb-ui-accent-hover);"
        );
        assert!(!STYLESHEET.contains(".tartan-button:hover"));
        assert!(!STYLESHEET.contains("opacity: .88"));
    }

    #[test]
    fn selection_colors_are_scoped_to_the_opt_in_shell() {
        let selection = rule(".tartan-shell::selection, .tartan-shell ::selection");
        assert!(selection.contains("background: var(--tartan-selection-background)"));
        assert!(selection.contains("color: var(--tartan-selection-foreground)"));
        assert!(!STYLESHEET.contains("\n::selection"));
        // Without broker roles, selection must not match the card surface.
        assert!(declaration("--tartan-selection-background:").contains("Highlight)"));
        assert!(declaration("--tartan-selection-foreground:").contains("HighlightText)"));
    }

    #[test]
    fn selected_navigation_is_neutral_with_an_accent_indicator() {
        let current = rule(".tartan-nav__link--current");
        assert!(current.contains("background: var(--tartan-surface)"));
        // Inset shadow, not a border, so host padding cannot change the height.
        assert!(current.contains("box-shadow: inset 0 -2px 0 var(--tartan-accent)"));
        assert!(!current.contains("border-bottom"));
        // Forced colours drop the shadow; the outline keeps the indicator.
        assert!(STYLESHEET.contains(
            "@media (forced-colors: active) { .tartan-nav__link--current { outline: 2px solid CanvasText;"
        ));
        assert!(!current.contains("--tartan-accent-soft"));
    }

    #[test]
    fn progress_and_loading_use_the_primary_accent_role() {
        assert!(rule(".tartan-progress__value").contains("background: var(--tartan-accent)"));
        assert!(
            rule(".tartan-loading__spinner").contains("border-top-color: var(--tartan-accent)")
        );
    }

    #[test]
    fn focus_ring_uses_the_focus_role_not_the_accent_role() {
        let focus = rule(":focus-visible");
        assert!(focus.contains("outline: 2px solid var(--tartan-focus)"));
        assert!(!focus.contains("var(--tartan-accent"));
    }

    #[test]
    fn feedback_modifiers_keep_their_status_roles() {
        for (modifier, role) in [
            ("error", "--tartan-danger"),
            ("warning", "--tartan-warning"),
            ("success", "--tartan-success"),
            ("info", "--tartan-info"),
        ] {
            let selector = format!(".tartan-feedback--{modifier}");
            assert!(
                rule(&selector).contains(&format!("border-color: var({role})")),
                "{selector} must keep {role}"
            );
        }
    }
}
