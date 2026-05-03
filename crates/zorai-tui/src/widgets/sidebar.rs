include!("sidebar_parts/sidebar.rs");
#[path = "sidebar/spawned_agents.rs"]
mod spawned_agents;
#[path = "sidebar/tab_layout.rs"]
mod tab_layout;

include!("sidebar_parts/show_spawned_to_selected_file_path.rs");
include!("sidebar_parts/filtered_file_index_to_render.rs");
include!("sidebar_parts/render_cached_to_spawned_sidebar_flatten_call_count.rs");
#[cfg(test)]
#[path = "tests/sidebar.rs"]
mod tests;
