//! Application-wide configuration.
//!
//! All environment variables are loaded once at startup via [`APP_CONFIG`].
//! Any module that needs a config value should import and dereference this static
//! rather than calling `env::var` directly.
//!
//! # Usage
//!
//! ```rust,no_run
//! use crate::config::app_config::APP_CONFIG;
//!
//! let path = &APP_CONFIG.cli_infos_toml;
//! ```
//!
//! # Initialization Order
//!
//! `dotenv().ok()` **must** be called before the first access to `APP_CONFIG`.
//! `main()` guarantees this by calling `dotenv` as its very first statement.

use std::env;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;


/// Global singleton that holds all application configuration values.
///
/// Initialized lazily on first access. If any required environment variable
/// is missing or unparseable the process panics immediately with a descriptive
/// message so misconfiguration is caught at startup.
pub static APP_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::load()
        .expect("[AppConfig] Failed to load application configuration from environment variables")
});


/// Strongly-typed container for all environment-based configuration.
///
/// Fields are grouped by concern:
/// - SQL Server connection settings (`mssql_*`)
/// - Application paths (`cli_infos_toml`)
#[derive(Debug)]
pub struct AppConfig {
    // ── SQL Server ─────────────────────────────────────────────────────────
    /// SQL Server hostname or IP address (`MSSQL_SERVER`).
    pub mssql_server: String,
    /// SQL Server port number (`MSSQL_PORT`, e.g. `1433`).
    pub mssql_port: u16,
    /// Target database name (`MSSQL_DATABASE`).
    pub mssql_database: String,
    /// Login username (`MSSQL_USERNAME`).
    pub mssql_username: String,
    /// Login password (`MSSQL_PASSWORD`).
    pub mssql_password: String,
    /// Whether to skip TLS certificate validation (`MSSQL_TRUST_SERVER_CERTIFICATE`, default `true`).
    pub mssql_trust_server_certificate: bool,

    // ── Application ────────────────────────────────────────────────────────
    /// Path to the TOML file that lists available batch jobs (`CLI_INFOS_TOML`).
    pub cli_infos_toml: String,
}

impl AppConfig {
    /// Reads all required environment variables and constructs an [`AppConfig`].
    ///
    /// # Errors
    ///
    /// Returns an error if any required variable is missing or if `MSSQL_PORT`
    /// cannot be parsed as a `u16`.
    fn load() -> Result<Self> {
        // ── SQL Server ──────────────────────────────────────────────────────
        let mssql_server = env::var("MSSQL_SERVER")
            .context("MSSQL_SERVER not set")?;

        let mssql_port = env::var("MSSQL_PORT")
            .context("MSSQL_PORT not set")?
            .parse::<u16>()
            .context("MSSQL_PORT must be a valid u16")?;

        let mssql_database = env::var("MSSQL_DATABASE")
            .context("MSSQL_DATABASE not set")?;

        let mssql_username = env::var("MSSQL_USERNAME")
            .context("MSSQL_USERNAME not set")?;

        let mssql_password = env::var("MSSQL_PASSWORD")
            .context("MSSQL_PASSWORD not set")?;

        // Optional — defaults to true (suitable for dev/test environments)
        let mssql_trust_server_certificate =
            env::var("MSSQL_TRUST_SERVER_CERTIFICATE")
                .unwrap_or_else(|_| "true".to_string())
                .to_lowercase()
                == "true";

        // ── Application ─────────────────────────────────────────────────────
        let cli_infos_toml: String = env::var("CLI_INFOS_TOML")
            .context("CLI_INFOS_TOML not set")?;
        
        Ok(Self {
            mssql_server,
            mssql_port,
            mssql_database,
            mssql_username,
            mssql_password,
            mssql_trust_server_certificate,
            cli_infos_toml,
        })
    }
}
