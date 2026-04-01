//! Agent configuration get/set.

use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

include!("config_types_and_weles.rs");
include!("config_weles_subagents.rs");
include!("config_value_helpers.rs");
include!("config_value_sanitize.rs");
include!("config_engine_projection.rs");
include!("config_engine_provider.rs");
include!("config_engine_subagents.rs");

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
