//! Sorting helpers for tree nodes.
//!
//! This module provides functions to compute sorted child indices for tree nodes,
//! independent of backend ordering. Sorting is per-subtree and can be based on
//! description, start clock, or duration.

use rjets::{DynTraceData, DynTraceRecord, TraceRecord};
use crate::state::{SortSpec, SortKey, SortDir};

/// Computes sorted child indices for a given parent record.
///
/// Returns a vector of indices into the parent's children array, ordered
/// according to the given sort specification. The indices can then be used
/// with `parent.child_at(index)` to iterate children in sorted order.
///
/// # Arguments
/// * `trace` - The trace data (unused but kept for consistency)
/// * `parent` - The parent record whose children should be sorted
/// * `spec` - The sort specification (key and direction)
///
/// # Returns
/// A vector of child indices in sorted order
pub fn sort_child_indices_for_parent(
    _trace: &DynTraceData,
    parent: &DynTraceRecord<'_>,
    spec: SortSpec,
) -> Vec<usize> {
    let n = parent.num_children();
    let mut items: Vec<(usize, ChildKey)> = Vec::with_capacity(n);

    for i in 0..n {
        if let Some(child) = parent.child_at(i) {
            let key = ChildKey::from_record(&child, spec.key);
            items.push((i, key));
        }
    }

    let asc = matches!(spec.dir, SortDir::Asc);
    items.sort_by(|a, b| if asc { a.1.cmp(&b.1) } else { b.1.cmp(&a.1) });

    items.into_iter().map(|(i, _)| i).collect()
}

/// Key used for sorting child records.
///
/// Only one field is populated based on the sort key.
/// This allows natural lexicographic ordering via derived Ord.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ChildKey {
    description: Option<String>,
    start_clk: Option<i64>,
    duration: Option<i64>,
}

impl ChildKey {
    /// Creates a ChildKey from a record based on the sort key.
    fn from_record(rec: &DynTraceRecord<'_>, key: SortKey) -> Self {
        match key {
            SortKey::Description => ChildKey {
                description: Some(rec.description()),
                start_clk: None,
                duration: None,
            },
            SortKey::StartClock => ChildKey {
                description: None,
                start_clk: Some(rec.clk()),
                duration: None,
            },
            SortKey::Duration => ChildKey {
                description: None,
                start_clk: None,
                duration: rec.duration(), // None sorts before Some by default
            },
        }
    }
}
