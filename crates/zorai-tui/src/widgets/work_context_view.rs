include!("work_context_view_parts/work_context_view.rs");
#[path = "work_context_view_selection.rs"]
mod selection;

include!(
    "work_context_view_parts/scrollbar_layout_from_metrics_to_selection_point_from_snapshot.rs"
);
include!("work_context_view_parts/selection_points_from_mouse_to_terminal_image_overlay_spec.rs");
#[cfg(test)]
#[path = "tests/work_context_view.rs"]
mod tests;
