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
        ((self.complete.min(self.total) * 100) / self.total) as u8
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

#[cfg(test)]
mod tests {
    use super::Progress;

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
}
