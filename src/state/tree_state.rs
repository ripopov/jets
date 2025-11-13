//! Tree expansion state management.
//!
//! This module encapsulates all state related to the tree view,
//! specifically which nodes are expanded or collapsed.

use std::collections::HashSet;

/// Sort key for tree node ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortKey {
    Description,
    StartClock,
    Duration,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortDir {
    Asc,
    Desc,
}

/// Complete sorting specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SortSpec {
    pub key: SortKey,
    pub dir: SortDir,
}

/// State related to tree node expansion.
///
/// Responsibilities:
/// - Tracking which tree nodes are expanded
/// - Providing intent-revealing expansion queries
/// - Managing bulk expansion operations
/// - Managing sorting specification
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    /// Set of expanded node IDs
    expanded_nodes: HashSet<u64>,
    /// Active sort specification (None = default backend order)
    active_sort: Option<SortSpec>,
}

impl TreeState {
    /// Creates a new tree state with no expanded nodes.
    pub fn new() -> Self {
        Self {
            expanded_nodes: HashSet::new(),
            active_sort: None,
        }
    }

    /// Clears all expansion state (collapses all nodes).
    pub fn clear(&mut self) {
        self.expanded_nodes.clear();
    }

    // ===== Sorting State =====

    /// Returns the active sort specification.
    pub fn active_sort(&self) -> Option<SortSpec> {
        self.active_sort
    }

    /// Sets the active sort specification.
    ///
    /// # Arguments
    /// * `spec` - The sort specification to apply, or None for default order
    pub fn set_active_sort(&mut self, spec: Option<SortSpec>) {
        self.active_sort = spec;
    }

    // ===== Expansion Queries =====

    /// Returns a reference to the set of expanded node IDs.
    ///
    /// This is useful for virtual scrolling and other performance-critical
    /// operations that need direct access to the HashSet.
    pub fn expanded_nodes_set(&self) -> &HashSet<u64> {
        &self.expanded_nodes
    }

    // ===== Expansion Mutations =====

    /// Expands the given node.
    ///
    /// # Arguments
    /// * `node_id` - The node to expand
    ///
    /// # Returns
    /// `true` if the node was newly expanded, `false` if already expanded.
    pub fn expand(&mut self, node_id: u64) -> bool {
        self.expanded_nodes.insert(node_id)
    }

    /// Collapses the given node.
    ///
    /// # Arguments
    /// * `node_id` - The node to collapse
    ///
    /// # Returns
    /// `true` if the node was expanded and is now collapsed, `false` if already collapsed.
    pub fn collapse(&mut self, node_id: u64) -> bool {
        self.expanded_nodes.remove(&node_id)
    }

}
