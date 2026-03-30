//! CLI service trait definition.
//!
//! This module defines the interface for presenting an interactive menu to the user
//! and collecting their batch job selection.

use crate::common::*;
use crate::model::cli_info::CliInfo;

use async_trait::async_trait;


/// Trait defining the CLI interaction interface.
///
/// Implementors are responsible for reading available batch jobs from a configuration
/// source, displaying a numbered menu, and returning the user's selection.
///
/// # Implementors
///
/// - [`CliServiceImpl`] - Production implementation that reads from `cli_infos.toml`
#[async_trait]
pub trait CliService: Send + Sync {
    /// Displays the list of enabled batch jobs as a numbered menu and waits for
    /// the user to make a selection.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(cli_info))` - The [`CliInfo`] entry the user selected
    /// * `Ok(None)`           - The user chose to exit
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed,
    /// or if reading from stdin fails.
    async fn select_batch_job(&self) -> Result<Option<CliInfo>>;
}
