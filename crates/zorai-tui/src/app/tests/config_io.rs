include!(
    "config_io_parts/normalize_provider_auth_source_falls_back_for_invalid_values_to_apply.rs"
);
include!("config_io_parts/build_config_patch_value_round_trips_daemon_backed_settings_to_build.rs");
include!(
    "config_io_parts/apply_config_json_loads_nested_compaction_settings_to_build_config_patch.rs"
);
include!("config_io_parts/build_config_patch_value_preserves_explicit_rarog.rs");
