//! Application-level coordination and workflow management.
//!
//! Handles high-level application operations like file loading, error handling,
//! and coordinating between different subsystems.

use crate::app::AppState;
use crate::io::{AsyncLoader, LoadResult};
use crate::state::SortSpec;
use crate::domain::sorting;
use std::path::PathBuf;
use std::collections::HashMap;
use rjets::{TraceMetadata, TraceData, TraceRecord};

/// Coordinates application-level operations and workflows.
///
/// This struct is responsible for:
/// - Managing file loading workflows
/// - Handling loading completion
/// - Coordinating virtual trace generation
/// - Managing error states
pub struct ApplicationCoordinator;

impl ApplicationCoordinator {
    /// Initiates asynchronous file loading.
    ///
    /// Immediately clears previous trace data to show loading indicator.
    pub fn open_file(
        state: &mut AppState,
        loader: &mut AsyncLoader,
        path: PathBuf,
        ctx: &egui::Context,
    ) {
        // Immediately clear previous trace data to show loading indicator
        state.reset_trace_state();

        // Start async loading
        loader.start_file_load(path, ctx);
    }

    /// Checks for loading completion and applies results to application state.
    ///
    /// Called once per frame in the update loop.
    /// Returns true if a load operation completed (success or error).
    pub fn check_loading_completion(state: &mut AppState, loader: &mut AsyncLoader) -> bool {
        match loader.check_completion() {
            LoadResult::Success { data, path } => {
                // Success: Initialize trace data and viewport
                let (min_clk, max_clk) = data.metadata().trace_extent();

                state.trace.load_trace(data, path);
                state.error_message = None;
                state.tree.clear();
                state.selection.clear();
                state.tree_cache.invalidate();

                state.initialize_viewport(min_clk, max_clk);
                true
            }
            LoadResult::Error(error_msg) => {
                // Error: Display error message
                state.error_message = Some(format!("Error loading trace: {}", error_msg));
                state.trace.clear();
                true
            }
            LoadResult::None => {
                // No result available yet (still loading or no operation active)
                false
            }
        }
    }

    /// Generates and loads a virtual trace in-memory.
    ///
    /// This is useful for testing and demonstration purposes.
    pub fn open_virtual_trace(state: &mut AppState, loader: &mut AsyncLoader) {
        match loader.load_virtual_trace() {
            Ok(data) => {
                // Get trace extent from metadata
                let (min_clk, max_clk) = data.metadata().trace_extent();

                state.trace.load_trace(data, None);
                state.error_message = None;
                state.tree.clear();
                state.selection.clear();
                state.tree_cache.invalidate();

                state.initialize_viewport(min_clk, max_clk);
            }
            Err(e) => {
                state.error_message = Some(format!("Error generating virtual trace: {}", e));
            }
        }
    }

    /// Handles tree node selection interaction.
    ///
    /// Updates selection state and auto-selects first event for new selections.
    pub fn handle_node_selection(
        state: &mut AppState,
        record_id: u64,
        was_already_selected: bool,
        first_event_clk: Option<i64>,
    ) {
        Self::update_record_selection(state, record_id, was_already_selected, first_event_clk);
    }

    /// Updates record selection with consistent auto-selection logic.
    ///
    /// If the record was not already selected, auto-selects the first event if available.
    /// If already selected, updates the record selection without changing the event.
    fn update_record_selection(
        state: &mut AppState,
        record_id: u64,
        was_already_selected: bool,
        first_event_clk: Option<i64>,
    ) {
        let event_to_select = if !was_already_selected {
            first_event_clk
        } else {
            // Already selected: just update record selection without changing event
            None
        };

        state.selection.select_record(record_id, event_to_select);
    }

    /// Handles tree node expand/collapse interaction.
    ///
    /// Updates expansion state and invalidates cache.
    pub fn handle_node_expand_toggle(state: &mut AppState, record_id: u64, was_expanded: bool) {
        if was_expanded {
            state.tree.collapse(record_id);
        } else {
            state.tree.expand(record_id);
        }
        // Invalidate cache when expand/collapse changes
        state.tree_cache.invalidate();
    }

    /// Handles timeline bar click interaction.
    ///
    /// Updates selection state and auto-selects first event for new selections.
    pub fn handle_timeline_bar_click(
        state: &mut AppState,
        record_id: u64,
        was_already_selected: bool,
        first_event_clk: Option<i64>,
    ) {
        // Use the intent-revealing selection API
        if !was_already_selected {
            // New selection: auto-select first event if available
            state.selection.select_record(record_id, first_event_clk);
        } else {
            // Already selected: just update record selection without changing event
            state.selection.select_record(record_id, None);
        }
    }
    /// Handles timeline event click interaction.
    ///
    /// Updates event selection and record selection.
    pub fn handle_timeline_event_click(state: &mut AppState, record_id: u64, event_clk: i64) {
        state.selection.select_event(record_id, event_clk);
    }

    /// Requests sorting of tree nodes.
    ///
    /// Sets the active sort and computes sorted child indices for all parents.
    /// Currently implemented synchronously; could be made async if needed for large traces.
    ///
    /// # Arguments
    /// * `state` - Application state
    /// * `spec` - Sort specification (key and direction)
    pub fn request_sorting(state: &mut AppState, spec: SortSpec) {
        // Set the active sort
        state.tree.set_active_sort(Some(spec));

        // Clear previous sorted children cache
        state.tree_cache.sorted_children.clear();

        // If we have trace data, compute sorted orderings
        if let Some(trace) = state.trace.trace_data() {
            let mut sorted_map: HashMap<(u64, SortSpec), Vec<usize>> = HashMap::new();

            // Compute sorted children for all parents recursively
            for root_id in trace.root_ids().iter().copied() {
                Self::compute_sorted_children_recursive(trace, root_id, spec, &mut sorted_map);
            }

            // Merge results into cache
            state.tree_cache.sorted_children.extend(sorted_map);
        }
    }

    /// Recursively computes sorted children for a subtree.
    ///
    /// # Arguments
    /// * `trace` - Trace data
    /// * `parent_id` - Parent record ID
    /// * `spec` - Sort specification
    /// * `out` - Output map to populate with sorted indices
    fn compute_sorted_children_recursive(
        trace: &rjets::DynTraceData,
        parent_id: u64,
        spec: SortSpec,
        out: &mut HashMap<(u64, SortSpec), Vec<usize>>,
    ) {
        if let Some(parent) = trace.get_record(parent_id) {
            // Only cache if parent has children
            if parent.num_children() > 0 {
                let order = sorting::sort_child_indices_for_parent(trace, &parent, spec);
                out.insert((parent_id, spec), order.clone());

                // Recurse into children using the sorted order
                for &i in &order {
                    if let Some(child) = parent.child_at(i) {
                        Self::compute_sorted_children_recursive(trace, child.id(), spec, out);
                    }
                }
            }
        }
    }
}
