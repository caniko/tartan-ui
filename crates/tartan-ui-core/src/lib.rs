//! Framework-neutral presentation contracts shared by the Tartanoglu apps.
//!
//! This crate deliberately contains no web framework, HTTP client, database, or
//! product-domain dependency. Applications map their own domain values into
//! these small values before rendering them.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub display_name: String,
    pub email: Option<String>,
    pub account_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLink {
    pub key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub href: String,
    pub kind: Option<String>,
}

/// A product-owned navigation destination rendered by the shared shell.
///
/// The product decides which links exist and which route is current; Tartan
/// only provides the framework-neutral representation and accessible markup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationLink {
    pub key: String,
    pub label: String,
    pub href: String,
    pub current: bool,
}

/// Caller-defined select option; counts are display-only, never used to infer filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub count: Option<u64>,
}

/// Explicit checkbox state; an empty selection has no library-defined meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterChoice {
    pub value: String,
    pub label: String,
    pub count: Option<u64>,
    pub checked: bool,
}

/// A resource that the authenticated account may open.
///
/// Applications derive this value from their authorization response; the
/// shared UI never infers access from a client-side route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibleResource {
    pub id: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Progress {
    pub total: u64,
    pub complete: u64,
    pub secondary: u64,
}

/// Server-computed progress for an accessible resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResourceSummary {
    pub total: u64,
    pub complete: u64,
    pub secondary: u64,
    pub remaining: u64,
}

impl ResourceSummary {
    pub fn progress(self) -> Progress {
        Progress {
            total: self.total,
            complete: self.complete,
            secondary: self.secondary,
        }
    }
}

impl Progress {
    pub fn remaining(self) -> u64 {
        self.total
            .saturating_sub(self.complete.saturating_add(self.secondary))
    }

    pub fn percent(self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        // Keep the calculation bounded even when counters come from a
        // large, server-authoritative dataset. Multiplying a u64 by 100 can
        // overflow before the final ratio is narrowed to the UI's 0..=100
        // range.
        let complete = u128::from(self.complete.min(self.total));
        let total = u128::from(self.total);
        ((complete * 100) / total) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feedback {
    pub kind: FeedbackKind,
    pub message: String,
}

/// The product-neutral theme preference used by shared controls.
///
/// Applications own persistence and document-level theme application. The
/// shared crate only provides the stable value and its keyboard-friendly
/// cycle order so web and native consumers do not drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Progress, ThemePreference};

    #[test]
    fn progress_is_saturating_and_bounded() {
        let progress = Progress {
            total: 10,
            complete: 12,
            secondary: 4,
        };
        assert_eq!(progress.remaining(), 0);
        assert_eq!(progress.percent(), 100);
    }

    #[test]
    fn empty_progress_has_zero_percent() {
        assert_eq!(Progress::default().percent(), 0);
    }

    #[test]
    fn large_progress_counters_remain_bounded() {
        let progress = Progress {
            total: u64::MAX,
            complete: u64::MAX,
            secondary: 0,
        };
        assert_eq!(progress.percent(), 100);

        let halfway = Progress {
            total: u64::MAX,
            complete: u64::MAX / 2,
            secondary: 0,
        };
        assert_eq!(halfway.percent(), 49);
    }

    #[test]
    fn theme_preference_cycles_without_application_side_effects() {
        assert_eq!(ThemePreference::default().as_str(), "system");
        assert_eq!(ThemePreference::System.next(), ThemePreference::Light);
        assert_eq!(ThemePreference::Light.next(), ThemePreference::Dark);
        assert_eq!(ThemePreference::Dark.next(), ThemePreference::System);
    }
}
