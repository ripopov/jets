//! Table header component rendering
//!
//! Handles the rendering of resizable column headers for the tree view.

use eframe::egui;
use egui::Color32;
use crate::rendering::text_utils::truncate_text_to_fit;
use crate::state::{LayoutState, SortSpec, SortKey, SortDir};

/// Interaction result from table header rendering.
pub enum TableHeaderInteraction {
    /// User clicked on a sortable column header.
    SortRequested(SortSpec),
}

/// Renders the resizable column headers for the tree view table
///
/// # Arguments
/// * `ui` - The egui UI context for drawing
/// * `layout` - Mutable reference to layout state containing expand_width and column_widths
/// * `current_sort` - Currently active sort specification
///
/// # Returns
/// * `Option<TableHeaderInteraction>` - Interaction result (e.g., sort request)
pub fn render_table_header(
    ui: &mut egui::Ui,
    layout: &mut LayoutState,
    current_sort: Option<SortSpec>,
) -> Option<TableHeaderInteraction> {
    let column_names = ["Name", "Description", "Start Clock", "Duration", "ID"];

    let mut x_offset = 0.0;
    let header_height = 24.0;
    let start_pos = ui.cursor().min;
    let mut interaction: Option<TableHeaderInteraction> = None;

    // Reserve space for the entire header row
    let (_header_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover()
    );

    let font_id = egui::FontId::proportional(14.0);
    let painter = ui.painter();

    // Get current expand_width value
    let mut expand_width = layout.expand_width();

    // Render expand/collapse column header (tree visualization area)
    let expand_label_rect = egui::Rect::from_min_size(
        egui::pos2(start_pos.x + x_offset, start_pos.y),
        egui::vec2(expand_width, header_height),
    );

    // Draw "Tree" label for the expand column
    painter.text(
        expand_label_rect.left_center() + egui::vec2(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "Tree",
        font_id.clone(),
        ui.visuals().strong_text_color(),
    );

    x_offset += expand_width;

    // Resize handle for expand column
    let handle_width = 8.0;
    let expand_handle_rect = egui::Rect::from_center_size(
        egui::pos2(start_pos.x + x_offset, start_pos.y + header_height / 2.0),
        egui::vec2(handle_width, header_height),
    );

    let expand_handle_id = ui.id().with("header_resize_expand");
    let expand_handle_response = ui.interact(expand_handle_rect, expand_handle_id, egui::Sense::drag());

    // Handle dragging for expand column
    if expand_handle_response.dragged() {
        let delta = expand_handle_response.drag_delta().x;
        expand_width = (expand_width + delta).max(50.0);
        *layout.expand_width_mut() = expand_width;
    }

    // Visual feedback for expand column resize handle
    let expand_handle_color = if expand_handle_response.hovered() || expand_handle_response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        Color32::from_rgb(100, 150, 255)
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke.color.gamma_multiply(0.5)
    };

    painter.rect_filled(expand_handle_rect.shrink(2.0), 0.0, expand_handle_color);

    // Render remaining column headers
    for (i, name) in column_names.iter().enumerate() {
        let width = layout.column_widths()[i];

        // Draw column header label
        let label_rect = egui::Rect::from_min_size(
            egui::pos2(start_pos.x + x_offset, start_pos.y),
            egui::vec2(width, header_height),
        );

        // Check if this column is sortable
        let sort_key = map_column_index_to_sort_key(i);
        let is_sortable = sort_key.is_some();

        // Make header clickable if sortable
        if is_sortable {
            let header_id = ui.id().with(format!("header_click_{}", i));
            let header_response = ui.interact(label_rect, header_id, egui::Sense::click());

            if header_response.clicked() {
                if let Some(key) = sort_key {
                    let new_spec = toggle_sort_direction(current_sort, key);
                    interaction = Some(TableHeaderInteraction::SortRequested(new_spec));
                }
            }

            // Highlight on hover
            if header_response.hovered() {
                painter.rect_filled(label_rect, 0.0, Color32::from_white_alpha(10));
            }
        }

        // Determine if this column is currently sorted
        let is_active_sort = current_sort.and_then(|spec|
            if sort_key == Some(spec.key) { Some(spec.dir) } else { None }
        );

        // Build display text with sort indicator
        let mut display_text = name.to_string();
        if let Some(dir) = is_active_sort {
            let arrow = match dir {
                SortDir::Asc => " ▲",
                SortDir::Desc => " ▼",
            };
            display_text.push_str(arrow);
        }

        let truncated_name = truncate_text_to_fit(&display_text, width, &font_id, painter);
        painter.text(
            label_rect.left_center() + egui::vec2(4.0, 0.0),
            egui::Align2::LEFT_CENTER,
            &truncated_name,
            font_id.clone(),
            ui.visuals().strong_text_color(),
        );

        x_offset += width;

        // Column resize handle
        if i < column_names.len() - 1 {
            let handle_width = 8.0;
            let handle_rect = egui::Rect::from_center_size(
                egui::pos2(start_pos.x + x_offset, start_pos.y + header_height / 2.0),
                egui::vec2(handle_width, header_height),
            );

            let handle_id = ui.id().with(format!("header_resize_{}", i));
            let handle_response = ui.interact(handle_rect, handle_id, egui::Sense::drag());

            // Handle dragging
            if handle_response.dragged() {
                let delta = handle_response.drag_delta().x;
                let new_width = (layout.column_widths()[i] + delta).max(50.0);
                layout.column_widths_mut()[i] = new_width;
            }

            // Visual feedback
            let color = if handle_response.hovered() || handle_response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                Color32::from_rgb(100, 150, 255)
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke.color.gamma_multiply(0.5)
            };

            ui.painter().rect_filled(handle_rect.shrink(2.0), 0.0, color);
        }
    }

    interaction
}

/// Maps column index to sort key (if sortable).
///
/// # Arguments
/// * `index` - Column index (0-4: Name, Description, Start Clock, Duration, ID)
///
/// # Returns
/// * `Some(SortKey)` if the column is sortable, `None` otherwise
fn map_column_index_to_sort_key(index: usize) -> Option<SortKey> {
    match index {
        1 => Some(SortKey::Description),   // Description column
        2 => Some(SortKey::StartClock),    // Start Clock column
        3 => Some(SortKey::Duration),      // Duration column
        _ => None,                         // Name and ID are not sortable
    }
}

/// Toggles sort direction or sets new sort key.
///
/// If the same key is clicked, toggles between Asc and Desc.
/// If a different key is clicked, sets Asc.
///
/// # Arguments
/// * `current` - Currently active sort specification
/// * `new_key` - The key that was clicked
///
/// # Returns
/// * `SortSpec` - New sort specification
fn toggle_sort_direction(current: Option<SortSpec>, new_key: SortKey) -> SortSpec {
    match current {
        Some(spec) if spec.key == new_key => {
            // Same key: toggle direction
            let new_dir = match spec.dir {
                SortDir::Asc => SortDir::Desc,
                SortDir::Desc => SortDir::Asc,
            };
            SortSpec { key: new_key, dir: new_dir }
        }
        _ => {
            // Different key or no current sort: start with Asc
            SortSpec { key: new_key, dir: SortDir::Asc }
        }
    }
}
