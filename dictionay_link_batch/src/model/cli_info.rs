//! CLI configuration models.
//!
//! This module provides the deserialization structures for `cli_infos.toml`,
//! which defines the list of batch jobs available to the user.

use serde::Deserialize;

/// Top-level structure representing the full contents of `cli_infos.toml`.
#[derive(Debug, Deserialize)]
pub struct CliInfos {
    /// All declared batch job entries.
    pub cli_info: Vec<CliInfo>,
}

/// A single batch job entry declared in `cli_infos.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct CliInfo {
    /// Unique identifier for the batch job. Must match a handler in `BatchServiceImpl`.
    pub cli_name: String,
    /// Whether this job is shown in the menu and allowed to run.
    pub enabled: bool,
    pub dictionary_path: String
}
