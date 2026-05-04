#![allow(dead_code)]

include!("modal_parts/default_to_last_error.rs");
include!("modal_parts/default_command_items.rs");
#[cfg(test)]
#[path = "tests/modal.rs"]
mod tests;
