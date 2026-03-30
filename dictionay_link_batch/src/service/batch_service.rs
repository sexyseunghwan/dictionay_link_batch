//! Batch service trait definition.
//!
//! This module defines the business logic interface for executing batch jobs
//! dispatched by the CLI controller.

use crate::common::*;
use crate::model::cli_info::CliInfo;

use async_trait::async_trait;


/// Trait defining the batch job execution interface.
///
/// Implementors are responsible for running a named batch job end-to-end.
/// The [`CliInfo`] passed in corresponds to an entry selected from `cli_infos.toml`.
///
/// # Implementors
///
/// - [`BatchServiceImpl`] - Production implementation backed by a SQL Server repository
#[async_trait]
pub trait BatchService: Send + Sync {
    /// Executes the batch job described by `cli_info`.
    ///
    /// # Arguments
    ///
    /// * `cli_info` - The selected [`CliInfo`] entry (contains `cli_name`, `dictionary_path`, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the job completes successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the job name is unknown or if execution fails.
    async fn run_batch(&self, cli_info: &CliInfo) -> Result<()>;
}
