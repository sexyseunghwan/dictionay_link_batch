//! CLI service implementation.
//!
//! This module provides a terminal-based interactive menu that reads available
//! batch jobs from `cli_infos.toml` and waits for the user to select one.

use async_trait::async_trait;
use derive_new::new;
use getset::Getters;
use std::io::{self, Write};

use crate::common::*;
use crate::model::cli_info::{CliInfo, CliInfos};
use crate::service::cli_service::*;


/// Terminal-based implementation of [`CliService`].
///
/// Reads `cli_infos.toml` on every call to `select_batch_job` so that changes
/// to the configuration file are picked up without restarting the process.
#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct CliServiceImpl {
    /// Path to the TOML file that lists available batch jobs.
    toml_path: String,
}


#[async_trait]
impl CliService for CliServiceImpl {
    /// Reads enabled batch jobs from the TOML file, prints a numbered menu,
    /// and returns the [`CliInfo`] entry the user selected.
    ///
    /// Loops until the user enters a valid number. Selecting the last option
    /// (Exit) returns `Ok(None)`.
    async fn select_batch_job(&self) -> Result<Option<CliInfo>> {
        // Read and parse the configuration file
        let content: String = std::fs::read_to_string(&self.toml_path)
            .with_context(|| format!("[CliServiceImpl::select_batch_job] Failed to read toml file: {}", self.toml_path))?;

        let cli_infos: CliInfos = toml::from_str(&content)
            .with_context(|| format!("[CliServiceImpl::select_batch_job] Failed to parse toml file: {}", self.toml_path))?;

        // Collect only enabled entries for display
        let enabled: Vec<CliInfo> = cli_infos
            .cli_info
            .into_iter()
            .filter(|c| c.enabled)
            .collect();

        if enabled.is_empty() {
            return Err(anyhow::anyhow!(
                "[CliServiceImpl::select_batch_job] No runnable batch jobs found (no entries with enabled = true)"
            ));
        }
        
        loop {
            // Print the numbered menu
            println!("\n=== Dictionary Link Batch ===");
            for (i, info) in enabled.iter().enumerate() {
                println!("  {}. {}", i + 1, info.cli_name);
            }
            println!("  {}. Exit", enabled.len() + 1);
            print!("\nSelect a number: ");
            io::stdout().flush()?;

            let mut input: String = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(n) if n == enabled.len() + 1 => return Ok(None),
                Ok(n) if n >= 1 && n <= enabled.len() => {
                    return Ok(Some(enabled[n - 1].clone()));
                }
                _ => {
                    println!(
                        "Invalid input. Please enter a number between 1 and {}.",
                        enabled.len() + 1
                    );
                }
            }
        }
    }
}
