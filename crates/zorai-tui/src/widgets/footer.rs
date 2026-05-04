include!("footer_parts/to_core_color_to_render_status_bar.rs");
include!("footer_parts/status_bar_hit_test.rs");
#[cfg(test)]
mod tests {
    include!(
        "footer_tests_parts/footer_handles_empty_state_to_status_bar_shows_playing_indicator.rs"
    );
}
