//! Framework-neutral facet and suggestion contracts for query editors.

use serde::{Deserialize, Serialize};

use crate::FilterChoice;

/// One qualifier (`period:`) with its selectable values. Non-editable
/// categories are set in the query text and shown read-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacetCategory {
    pub key: String,
    pub label: String,
    pub options: Vec<FilterChoice>,
    pub editable: bool,
    /// Option counts are lower bounds (rendered as `10000+`).
    pub counts_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacetGroup {
    pub title: String,
    pub categories: Vec<FacetCategory>,
}

/// A completion offered by a query box. `insert` replaces the current token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub insert: String,
    pub label: String,
    pub detail: Option<String>,
}
