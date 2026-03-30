//! SQL Server repository trait definition.
//!
//! This module defines the data access layer interface for SQL Server database operations.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     SqlServerRepository                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │              mssql_client::Client                        │   │
//! │  │            (Connection per Operation)                    │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! │                            │                                    │
//! │              ┌─────────────┴─────────────┐                      │
//! │              ▼                           ▼                      │
//! │     ┌─────────────────┐       ┌─────────────────┐               │
//! │     │  Query / Fetch  │       │  Test / Health  │               │
//! │     └─────────────────┘       └─────────────────┘               │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Environment Variables
//!
//! | Variable                        | Description                          | Example         |
//! |---------------------------------|--------------------------------------|-----------------|
//! | `MSSQL_SERVER`                  | SQL Server host                      | `localhost`     |
//! | `MSSQL_PORT`                    | SQL Server port                      | `1433`          |
//! | `MSSQL_DATABASE`                | Target database name                 | `your_database` |
//! | `MSSQL_USERNAME`                | Login username                       | `sa`            |
//! | `MSSQL_PASSWORD`                | Login password                       | `your_password` |
//! | `MSSQL_TRUST_SERVER_CERTIFICATE`| Whether to trust the server cert     | `true`          |

use anyhow::Result;
use async_trait::async_trait;

use crate::dtos::sample_proc_dto::{SampleProcRequestDto, SampleProcResponseDto};


/// Trait defining SQL Server repository operations.
///
/// This trait abstracts the SQL Server data access layer using `mssql-client`,
/// providing async methods for database interaction.
///
/// # Implementors
///
/// - [`SqlServerRepositoryImpl`] - Production implementation with real DB connection
#[async_trait]
pub trait SqlServerRepository: Send + Sync {
    /// Verifies that a connection to the SQL Server can be established.
    ///
    /// Creates a new client using the configured connection settings
    /// and immediately drops it, confirming connectivity.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection succeeds.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment variables are missing or invalid
    /// - The SQL Server is unreachable
    /// - Authentication fails
    async fn test_connection(&self) -> Result<()>;

    /// Calls `usp_sample_proc` with the given input parameters and returns all result rows.
    ///
    /// # Arguments
    ///
    /// * `req` — input parameters for the stored procedure
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<SampleProcResponseDto>)` with one element per returned row.
    /// Returns an empty `Vec` if the procedure produces no rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or the procedure raises an exception.
    async fn call_proc_sample(
        &self,
        req: &SampleProcRequestDto,
    ) -> Result<Vec<SampleProcResponseDto>>;
}
