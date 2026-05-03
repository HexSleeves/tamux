//! Predefined LLM provider definitions.
//!
//! This keeps the TUI's built-in provider defaults aligned with the app-wide
//! provider registry while remaining a TUI-local source for picker/config UX.

include!("providers_parts/providers.rs");
#[path = "providers/model_catalog.rs"]
mod model_catalog;

mod context;

include!("providers_parts/normalize_model_lookup_value_to_default_model_for_provider_auth.rs");
include!("providers_parts/known_models_for_provider_auth.rs");
#[cfg(test)]
#[path = "providers/tests.rs"]
mod tests;
