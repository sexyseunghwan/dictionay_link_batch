//! Main controller.
//!
//! This module orchestrates the CLI interaction loop: it repeatedly asks the
//! user to select a batch job via [`CliService`] and delegates execution to
//! [`BatchService`] until the user chooses to exit.

use derive_new::new;

use crate::common::*;
use crate::service::batch_service::BatchService;
use crate::service::cli_service::CliService;

/// Top-level controller that drives the interactive batch job loop.
///
/// `MainController` owns both service dependencies and coordinates them:
/// the CLI service handles user input while the batch service handles execution.
/// Neither service knows about the other.
#[derive(Debug, new)]
pub struct MainController<B, C>
where
    B: BatchService,
    C: CliService,
{
    /// Service responsible for executing batch jobs by name.
    batch_service: B,
    /// Service responsible for presenting the menu and collecting user input.
    cli_service: C,
}

impl<B, C> MainController<B, C>
where
    B: BatchService,
    C: CliService,
{
    /// Runs the interactive batch selection loop until the user exits.
    ///
    /// On each iteration the user selects a batch job from the menu.
    /// Batch execution errors are logged but do not terminate the loop,
    /// allowing the user to retry or select a different job.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the user selects the exit option.
    ///
    /// # Errors
    ///
    /// Returns an error if the CLI service fails (e.g. unreadable config file
    /// or broken stdin).
    pub async fn main_task(&self) -> Result<()> {
        loop {
            match self.cli_service.select_batch_job().await? {
                None => {
                    // User selected Exit
                    info!("[MainController::main_task] Exiting program.");
                    break;
                }
                Some(cli_info) => {
                    if let Err(e) = self.batch_service.run_batch(&cli_info).await {
                        error!(
                            "[MainController::main_task] Batch job failed [{}]: {}",
                            cli_info.cli_name, e
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
