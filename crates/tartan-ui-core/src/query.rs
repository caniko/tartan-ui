//! Generic boolean query tree shared by text and facet editors.
//!
//! The leaf type is product-owned. Variant names match the serde shape of the
//! products' former local expression enums, so persisted JSON stays valid.

use std::collections::BTreeMap;
use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryExpr<L> {
    And(Vec<QueryExpr<L>>),
    Or(Vec<QueryExpr<L>>),
    Not(Box<QueryExpr<L>>),
    Pred(L),
}

/// All leaves in depth-first order.
pub fn leaves<L>(expr: Option<&QueryExpr<L>>) -> Vec<&L> {
    fn collect<'a, L>(expr: &'a QueryExpr<L>, out: &mut Vec<&'a L>) {
        match expr {
            QueryExpr::And(parts) | QueryExpr::Or(parts) => {
                parts.iter().for_each(|part| collect(part, out))
            }
            QueryExpr::Not(inner) => collect(inner, out),
            QueryExpr::Pred(pred) => out.push(pred),
        }
    }
    let mut out = Vec::new();
    if let Some(expr) = expr {
        collect(expr, &mut out);
    }
    out
}

pub fn leaf_count<L>(expr: Option<&QueryExpr<L>>) -> usize {
    leaves(expr).len()
}

/// One leaf with its display context. `index` is the depth-first position
/// shared with [`leaves`] and [`remove_leaf`]; `negated` reports an odd number
/// of enclosing `NOT`s so pills can show an exclusion as an exclusion.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedLeaf<L> {
    pub predicate: L,
    pub negated: bool,
    pub index: usize,
}

pub fn annotated_leaves<L: Clone>(expr: Option<&QueryExpr<L>>) -> Vec<AnnotatedLeaf<L>> {
    fn collect<L: Clone>(expr: &QueryExpr<L>, negated: bool, out: &mut Vec<AnnotatedLeaf<L>>) {
        match expr {
            QueryExpr::And(parts) | QueryExpr::Or(parts) => {
                parts.iter().for_each(|part| collect(part, negated, out))
            }
            QueryExpr::Not(inner) => collect(inner, !negated, out),
            QueryExpr::Pred(pred) => out.push(AnnotatedLeaf {
                predicate: pred.clone(),
                negated,
                index: out.len(),
            }),
        }
    }
    let mut out = Vec::new();
    if let Some(expr) = expr {
        collect(expr, false, &mut out);
    }
    out
}

/// Remove the leaf at depth-first `index` (the order [`leaves`] returns),
/// collapsing emptied groups. Returns `None` when the whole tree is gone.
pub fn remove_leaf<L: Clone>(expr: &QueryExpr<L>, index: usize) -> Option<QueryExpr<L>> {
    fn remove<L: Clone>(
        expr: &QueryExpr<L>,
        target: &mut usize,
        done: &mut bool,
    ) -> Option<QueryExpr<L>> {
        match expr {
            QueryExpr::Pred(pred) => {
                if !*done && *target == 0 {
                    *done = true;
                    None
                } else {
                    if !*done {
                        *target -= 1;
                    }
                    Some(QueryExpr::Pred(pred.clone()))
                }
            }
            QueryExpr::Not(inner) => {
                remove(inner, target, done).map(|kept| QueryExpr::Not(Box::new(kept)))
            }
            QueryExpr::And(parts) => collapse(QueryExpr::And(
                parts
                    .iter()
                    .filter_map(|part| remove(part, target, done))
                    .collect(),
            )),
            QueryExpr::Or(parts) => collapse(QueryExpr::Or(
                parts
                    .iter()
                    .filter_map(|part| remove(part, target, done))
                    .collect(),
            )),
        }
    }
    fn collapse<L>(expr: QueryExpr<L>) -> Option<QueryExpr<L>> {
        match expr {
            QueryExpr::And(parts) | QueryExpr::Or(parts) if parts.is_empty() => None,
            QueryExpr::And(mut parts) if parts.len() == 1 => parts.pop(),
            QueryExpr::Or(mut parts) if parts.len() == 1 => parts.pop(),
            expr => Some(expr),
        }
    }
    remove(expr, &mut { index }, &mut false)
}

/// Canonical single-line rendering: OR binds loosest, then implicit AND, then
/// `NOT`; a child is parenthesised when it binds looser than its parent.
impl<L: Display> Display for QueryExpr<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_prec(self, 0))
    }
}

fn format_prec<L: Display>(expr: &QueryExpr<L>, parent_prec: u8) -> String {
    const PREC_OR: u8 = 1;
    const PREC_AND: u8 = 2;
    const PREC_NOT: u8 = 3;
    let group = |parts: &[QueryExpr<L>], prec: u8, sep: &str| {
        let body = parts
            .iter()
            .map(|part| format_prec(part, prec))
            .collect::<Vec<_>>()
            .join(sep);
        if prec < parent_prec {
            format!("({body})")
        } else {
            body
        }
    };
    match expr {
        QueryExpr::Or(parts) => group(parts, PREC_OR, " OR "),
        QueryExpr::And(parts) => group(parts, PREC_AND, " "),
        QueryExpr::Not(inner) => format!("NOT {}", format_prec(inner, PREC_NOT)),
        QueryExpr::Pred(pred) => pred.to_string(),
    }
}

/// The host lexer turns a bare `AND`/`OR`/`NOT` into a boolean operator, so a
/// literal with one of those words must be quoted (the comparison ignores case
/// so a future case-insensitive lexer keeps working).
fn reads_as_operator(value: &str) -> bool {
    matches!(value.to_ascii_uppercase().as_str(), "AND" | "OR" | "NOT")
}

/// A leading `-` is negation before a quote, `(`, `_` or a letter; quoting the
/// value keeps it a word. Negative numbers and hyphenated words stay bare.
fn starts_with_negation(value: &str) -> bool {
    match value.strip_prefix('-').and_then(|rest| rest.chars().next()) {
        Some('"' | '(' | '_') => true,
        Some(char) => char.is_ascii_alphabetic(),
        None => false,
    }
}

/// Quote a value for the text syntax when it is empty, contains whitespace or
/// any of `( ) " : < >`, reads as a boolean operator or starts with negation.
/// Escapes `"` and `\`.
pub fn quote_value(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value
            .chars()
            .any(|char| char.is_whitespace() || matches!(char, '(' | ')' | '"' | ':' | '<' | '>'))
        || reads_as_operator(value)
        || starts_with_negation(value);
    if !needs_quotes {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for char in value.chars() {
        if matches!(char, '"' | '\\') {
            out.push('\\');
        }
        out.push(char);
    }
    out.push('"');
    out
}

/// A leaf that can be a `key:value` facet selection.
pub trait FacetLeaf {
    fn as_facet(&self) -> Option<(&str, &str)>;
    fn facet(key: &str, value: &str) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetSelection {
    /// Every value of the key in the tree, in order of appearance.
    pub values: Vec<String>,
    /// True only when the key is carried by exactly one top-level selection
    /// conjunct and appears nowhere else, so a checkbox can edit it safely.
    pub editable: bool,
}

fn conjuncts<L>(expr: Option<&QueryExpr<L>>) -> &[QueryExpr<L>] {
    match expr {
        Some(QueryExpr::And(parts)) => parts,
        Some(expr) => std::slice::from_ref(expr),
        None => &[],
    }
}

/// The facet key when `expr` is a selection: one facet leaf, or a non-empty
/// `Or` made only of facet leaves of one key.
fn selection_key<L: FacetLeaf>(expr: &QueryExpr<L>) -> Option<&str> {
    fn key_of<L: FacetLeaf>(expr: &QueryExpr<L>) -> Option<&str> {
        match expr {
            QueryExpr::Pred(leaf) => leaf.as_facet().map(|(key, _)| key),
            _ => None,
        }
    }
    match expr {
        QueryExpr::Or(parts) => {
            let key = key_of(parts.first()?)?;
            parts
                .iter()
                .all(|part| key_of(part) == Some(key))
                .then_some(key)
        }
        expr => key_of(expr),
    }
}

pub fn facet_selection<L: FacetLeaf>(
    expr: Option<&QueryExpr<L>>,
) -> BTreeMap<String, FacetSelection> {
    let mut out: BTreeMap<String, FacetSelection> = BTreeMap::new();
    for (key, value) in leaves(expr).into_iter().filter_map(|leaf| leaf.as_facet()) {
        out.entry(key.to_string())
            .or_insert(FacetSelection {
                values: Vec::new(),
                editable: false,
            })
            .values
            .push(value.to_string());
    }
    let mut selections: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for conjunct in conjuncts(expr) {
        if let Some(key) = selection_key(conjunct) {
            let entry = selections.entry(key).or_default();
            entry.0 += 1;
            entry.1 += leaf_count(Some(conjunct));
        }
    }
    for (key, selection) in &mut out {
        selection.editable = selections
            .get(key.as_str())
            .is_some_and(|&(count, leaf_total)| count == 1 && leaf_total == selection.values.len());
    }
    out
}

/// Toggle one `key:value` in the key's selection. Non-editable keys and every
/// other conjunct are left untouched.
pub fn toggle_facet<L: FacetLeaf + Clone>(
    expr: Option<QueryExpr<L>>,
    key: &str,
    value: &str,
) -> Option<QueryExpr<L>> {
    if facet_selection(expr.as_ref())
        .get(key)
        .is_some_and(|selection| !selection.editable)
    {
        return expr;
    }
    let mut parts = match expr {
        Some(QueryExpr::And(parts)) => parts,
        Some(expr) => vec![expr],
        None => Vec::new(),
    };
    let is_value = |part: &QueryExpr<L>| matches!(part, QueryExpr::Pred(leaf) if leaf.as_facet() == Some((key, value)));
    let new = || QueryExpr::Pred(L::facet(key, value));
    match parts
        .iter()
        .position(|part| selection_key(part) == Some(key))
    {
        None => parts.push(new()),
        Some(at) => {
            let mut members = match parts.remove(at) {
                QueryExpr::Or(members) => members,
                single => vec![single],
            };
            if members.iter().any(is_value) {
                members.retain(|member| !is_value(member));
            } else {
                members.push(new());
            }
            match members.len() {
                0 => {}
                1 => parts.insert(at, members.remove(0)),
                _ => parts.insert(at, QueryExpr::Or(members)),
            }
        }
    }
    match parts.len() {
        0 => None,
        1 => parts.pop(),
        _ => Some(QueryExpr::And(parts)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use QueryExpr::{And, Not, Or, Pred};

    #[derive(Debug, Clone, PartialEq)]
    enum T {
        Facet { key: String, value: String },
        Other(String),
    }

    impl Display for T {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                T::Facet { key, value } => write!(f, "{key}:{}", quote_value(value)),
                T::Other(text) => f.write_str(text),
            }
        }
    }

    impl FacetLeaf for T {
        fn as_facet(&self) -> Option<(&str, &str)> {
            match self {
                T::Facet { key, value } => Some((key, value)),
                T::Other(_) => None,
            }
        }
        fn facet(key: &str, value: &str) -> Self {
            T::Facet {
                key: key.into(),
                value: value.into(),
            }
        }
    }

    fn f(key: &str, value: &str) -> QueryExpr<T> {
        Pred(T::facet(key, value))
    }
    fn o(text: &str) -> QueryExpr<T> {
        Pred(T::Other(text.into()))
    }
    fn toggled(expr: Option<QueryExpr<T>>, key: &str, value: &str) -> Option<QueryExpr<T>> {
        toggle_facet(expr, key, value)
    }

    /// The host lexer's word rules, enough to prove a formatted leaf keeps its
    /// value: whitespace and `( ) :` end a word, `"` quotes one, a leading `-`
    /// before a quote, paren, letter or `_` is negation, and a bare
    /// `AND`/`OR`/`NOT` is an operator.
    #[derive(Debug, PartialEq)]
    enum Lex {
        Word(String),
        Operator,
        Negation,
    }

    fn word_kind(word: String) -> Lex {
        match word.as_str() {
            "AND" | "OR" | "NOT" => Lex::Operator,
            _ => Lex::Word(word),
        }
    }

    fn negation_follows(chars: &[char], index: usize) -> bool {
        match chars.get(index) {
            Some('"' | '(' | '_') => true,
            Some(char) => char.is_ascii_alphabetic(),
            None => false,
        }
    }

    fn lex(input: &str) -> Vec<Lex> {
        let chars: Vec<char> = input.chars().collect();
        let mut index = 0;
        let mut out = Vec::new();
        while index < chars.len() {
            let char = chars[index];
            if char.is_whitespace() || matches!(char, '(' | ')' | ':') {
                index += 1;
            } else if char == '"' {
                index += 1;
                let mut word = String::new();
                while index < chars.len() {
                    match chars[index] {
                        '\\' => {
                            index += 1;
                            if index < chars.len() {
                                word.push(chars[index]);
                                index += 1;
                            }
                        }
                        '"' => {
                            index += 1;
                            break;
                        }
                        next => {
                            word.push(next);
                            index += 1;
                        }
                    }
                }
                out.push(Lex::Word(word));
            } else if char == '-' && negation_follows(&chars, index + 1) {
                out.push(Lex::Negation);
                index += 1;
            } else {
                let mut word = String::new();
                while index < chars.len()
                    && !chars[index].is_whitespace()
                    && !matches!(chars[index], '(' | ')' | ':' | '"')
                {
                    word.push(chars[index]);
                    index += 1;
                }
                out.push(word_kind(word));
            }
        }
        out
    }

    #[test]
    fn operator_words_and_negation_prefixes_are_quoted() {
        // A bare AND/OR/NOT is an operator to the host lexer.
        for word in ["AND", "OR", "NOT"] {
            assert_eq!(quote_value(word), format!("\"{word}\""));
            assert_eq!(lex(word), vec![Lex::Operator], "{word}");
            assert_eq!(lex(&quote_value(word)), vec![Lex::Word(word.to_owned())]);
            // The formatted facet keeps the value literal.
            assert_eq!(
                lex(&f("kind", word).to_string()),
                vec![Lex::Word("kind".to_owned()), Lex::Word(word.to_owned())],
                "{word}"
            );
        }
        // The lexer is case-sensitive today; quoting is not, so a literal can
        // never become an operator through a case change in either direction.
        for word in ["and", "Or", "not"] {
            assert_eq!(quote_value(word), format!("\"{word}\""));
            assert_eq!(lex(&quote_value(word)), vec![Lex::Word(word.to_owned())]);
            assert_eq!(
                lex(&f("kind", word).to_string()),
                vec![Lex::Word("kind".to_owned()), Lex::Word(word.to_owned())]
            );
        }
        // A leading `-` before a quote, paren, letter or `_` is negation.
        for word in ["-abc", "-_x", r#"-"q""#, "-("] {
            assert!(quote_value(word).starts_with('"'), "{word}");
            assert!(
                lex(word).contains(&Lex::Negation),
                "the unquoted value should read as negation: {word}"
            );
            assert_eq!(lex(&quote_value(word)), vec![Lex::Word(word.to_owned())]);
            assert_eq!(
                lex(&f("kind", word).to_string()),
                vec![Lex::Word("kind".to_owned()), Lex::Word(word.to_owned())]
            );
        }
        // A hyphen that is not negation stays a bare word.
        for word in ["-40", "e-mail", "-"] {
            assert_eq!(quote_value(word), word);
            assert_eq!(lex(word), vec![Lex::Word(word.to_owned())], "{word}");
        }
    }

    #[test]
    fn display_uses_precedence_and_quotes() {
        assert_eq!(
            And(vec![Or(vec![o("a"), o("b")]), o("c")]).to_string(),
            "(a OR b) c"
        );
        assert_eq!(
            Or(vec![And(vec![o("a"), o("b")]), o("c")]).to_string(),
            "a b OR c"
        );
        assert_eq!(
            Not(Box::new(And(vec![o("a"), o("b")]))).to_string(),
            "NOT (a b)"
        );
        assert_eq!(
            Not(Box::new(Not(Box::new(o("a"))))).to_string(),
            "NOT NOT a"
        );
        assert_eq!(f("kind", "two words").to_string(), "kind:\"two words\"");
        assert_eq!(quote_value(""), "\"\"");
        assert_eq!(quote_value(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(quote_value("plain"), "plain");
    }

    #[test]
    fn remove_leaf_collapses_groups() {
        let expr = And(vec![f("a", "1"), Or(vec![f("b", "2"), f("b", "3")])]);
        assert_eq!(leaf_count(Some(&expr)), 3);
        assert_eq!(
            remove_leaf(&expr, 1),
            Some(And(vec![f("a", "1"), f("b", "3")]))
        );
        assert_eq!(
            remove_leaf(&expr, 0),
            Some(Or(vec![f("b", "2"), f("b", "3")]))
        );
        assert_eq!(remove_leaf(&f("a", "1"), 0), None);
        assert_eq!(remove_leaf(&Not(Box::new(f("a", "1"))), 0), None);
        assert_eq!(remove_leaf(&f("a", "1"), 5), Some(f("a", "1")));
    }

    #[test]
    fn annotated_leaves_track_negation() {
        let expr = And(vec![
            f("a", "1"),
            Not(Box::new(Or(vec![f("b", "2"), o("x")]))),
        ]);
        let leaves = annotated_leaves(Some(&expr));
        let flags: Vec<_> = leaves
            .iter()
            .map(|leaf| (leaf.index, leaf.negated))
            .collect();
        assert_eq!(flags, [(0, false), (1, true), (2, true)]);
        assert!(annotated_leaves::<T>(None).is_empty());
    }

    #[test]
    fn selection_reports_editable_and_non_editable_keys() {
        let sel = facet_selection(Some(&And(vec![
            Or(vec![f("period", "1900"), f("period", "1950")]),
            f("kind", "plan"),
            o("text"),
        ])));
        assert_eq!(sel["period"].values, ["1900", "1950"]);
        assert!(sel["period"].editable && sel["kind"].editable);

        let negated = facet_selection(Some(&And(vec![f("a", "1"), Not(Box::new(f("a", "2")))])));
        assert_eq!(negated["a"].values, ["1", "2"]);
        assert!(!negated["a"].editable);

        let only_negated = facet_selection(Some(&Not(Box::new(f("a", "1")))));
        assert!(!only_negated["a"].editable);

        let mixed = facet_selection(Some(&Or(vec![f("a", "1"), o("x")])));
        assert!(!mixed["a"].editable);

        let duplicate = facet_selection(Some(&And(vec![f("a", "1"), f("a", "2")])));
        assert!(!duplicate["a"].editable);

        assert!(facet_selection::<T>(None).is_empty());
    }

    #[test]
    fn toggle_adds_removes_and_collapses() {
        let one = toggled(None, "a", "1");
        assert_eq!(one, Some(f("a", "1")));
        let two = toggled(one, "a", "2");
        assert_eq!(two, Some(Or(vec![f("a", "1"), f("a", "2")])));
        let three = toggled(two, "a", "3");
        assert_eq!(three, Some(Or(vec![f("a", "1"), f("a", "2"), f("a", "3")])));
        let back = toggled(toggled(three, "a", "3"), "a", "2");
        assert_eq!(back, Some(f("a", "1")));
        assert_eq!(toggled(back, "a", "1"), None);
    }

    #[test]
    fn toggle_appends_to_conjunction_and_keeps_other_conjuncts() {
        let root = Some(o("text"));
        let with = toggled(root, "a", "1");
        assert_eq!(with, Some(And(vec![o("text"), f("a", "1")])));
        let other = toggled(with, "b", "2");
        assert_eq!(other, Some(And(vec![o("text"), f("a", "1"), f("b", "2")])));
        // Removing the only selection leaves the other conjuncts, collapsed.
        let rest = toggled(toggled(other, "a", "1"), "b", "2");
        assert_eq!(rest, Some(o("text")));
        // Adding to an existing selection keeps its position.
        let grown = toggled(Some(And(vec![f("a", "1"), o("text")])), "a", "2");
        assert_eq!(
            grown,
            Some(And(vec![Or(vec![f("a", "1"), f("a", "2")]), o("text")]))
        );
    }

    #[test]
    fn toggle_leaves_non_editable_keys_alone() {
        let negated = Some(And(vec![f("a", "1"), Not(Box::new(f("a", "2")))]));
        assert_eq!(toggled(negated.clone(), "a", "3"), negated);
        assert_eq!(toggled(negated.clone(), "a", "1"), negated);
        let lone = Some(And(vec![f("a", "1")]));
        assert_eq!(toggled(lone.clone(), "a", "1"), None);
        let mixed = Some(Or(vec![f("a", "1"), o("x")]));
        assert_eq!(toggled(mixed.clone(), "a", "2"), mixed);
    }
}
