//! SQL Server repository implementation.
//!
//! This module provides the data access layer for SQL Server database operations
//! using `mssql-client` as the underlying driver.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  SqlServerRepositoryImpl                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │         Connection Config (built from env vars)          │   │
//! │  │              host / port / database / auth               │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! │                            │                                     │
//! │              ┌─────────────┴─────────────┐                      │
//! │              ▼                           ▼                      │
//! │     ┌─────────────────┐       ┌─────────────────┐              │
//! │     │  get_client()   │       │test_connection() │              │
//! │     │ (per operation) │       │  (health check)  │              │
//! │     └─────────────────┘       └─────────────────┘              │
//! │                                                                  │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Connection Strategy
//!
//! A new `Client` is created per operation via `get_client()`.
//! Connection settings are read once at startup in `new()` and stored as fields.
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

use crate::common::*;
use crate::repository::sqlserver_repository::SqlServerRepository;

use async_trait::async_trait;
use mssql_client::{Client, Config, Credentials, state::Ready};
use getset::Getters;

/// Concrete implementation of the SQL Server repository.
///
/// `SqlServerRepositoryImpl` reads connection settings from environment variables
/// once at construction time and builds a fresh `Client` for each operation.
///
/// # Thread Safety
///
/// All fields are plain `String` / `u16` / `bool` values, so the struct is
/// `Send + Sync` and can be shared across async tasks behind an `Arc`.
///
/// # Examples
///
/// ```rust,no_run
/// use crate::repository_impl::sqlserver_repository_impl::SqlServerRepositoryImpl;
/// use crate::repository::sqlserver_repository::SqlServerRepository;
///
/// // Ensure env vars are set (e.g. via .env):
/// // MSSQL_SERVER=localhost
/// // MSSQL_PORT=1433
/// // MSSQL_DATABASE=your_database
/// // MSSQL_USERNAME=sa
/// // MSSQL_PASSWORD=your_password
///
/// let repo = SqlServerRepositoryImpl::new()?;
/// repo.test_connection().await?;
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SqlServerRepositoryImpl {
    /// SQL Server hostname or IP address.
    server: String,
    /// SQL Server port number (default: 1433).
    port: u16,
    /// Target database name.
    database: String,
    /// Login username for SQL Server authentication.
    username: String,
    /// Login password for SQL Server authentication.
    password: String,
    /// Whether to skip TLS certificate validation.
    trust_cert: bool,
}

impl SqlServerRepositoryImpl {
    /// Creates a new `SqlServerRepositoryImpl` by reading connection settings
    /// from environment variables.
    ///
    /// # Environment Variables
    ///
    /// * `MSSQL_SERVER`                  - Required. Hostname or IP of the SQL Server
    /// * `MSSQL_PORT`                    - Required. Port number (e.g. `1433`)
    /// * `MSSQL_DATABASE`                - Required. Target database name
    /// * `MSSQL_USERNAME`                - Required. Login username
    /// * `MSSQL_PASSWORD`                - Required. Login password
    /// * `MSSQL_TRUST_SERVER_CERTIFICATE`- Optional. Defaults to `"true"`
    ///
    /// # Returns
    ///
    /// Returns `Ok(SqlServerRepositoryImpl)` when all required variables are present
    /// and `MSSQL_PORT` can be parsed as a valid `u16`.
    ///
    /// # Errors
    ///
    /// Returns an error if any required environment variable is missing
    /// or if `MSSQL_PORT` is not a valid number.
    pub fn new() -> Result<Self> {
        // Read all required connection parameters from the environment
        let server: String   = env::var("MSSQL_SERVER").context("MSSQL_SERVER not set")?;
        let port: u16 = env::var("MSSQL_PORT")
            .context("MSSQL_PORT not set")?
            .parse()
            .context("MSSQL_PORT must be a valid number")?;
        let database: String = env::var("MSSQL_DATABASE").context("MSSQL_DATABASE not set")?;
        let username: String = env::var("MSSQL_USERNAME").context("MSSQL_USERNAME not set")?;
        let password: String = env::var("MSSQL_PASSWORD").context("MSSQL_PASSWORD not set")?;

        // Default to trusting the server certificate when the variable is absent
        let trust_cert = env::var("MSSQL_TRUST_SERVER_CERTIFICATE")
            .unwrap_or("true".to_string())
            .to_lowercase() == "true";

        Ok(Self { server, port, database, username, password, trust_cert })
    }

    /// Builds a `mssql_client::Config` from the stored connection settings.
    ///
    /// # Returns
    ///
    /// A fully configured `Config` ready to be passed to `Client::connect`.
    fn build_config(&self) -> Config {
        // Build config using method chaining (each method returns Self)
        Config::new()
            // Set host, port and target database
            .host(&self.server)
            .port(self.port)
            .database(&self.database)
            // Use SQL Server login authentication
            .credentials(Credentials::sql_server(self.username.clone(), self.password.clone()))
            // Optionally disable TLS certificate validation (useful in dev/test environments)
            .trust_server_certificate(self.trust_cert)
    }

    /// Opens a new connection to the SQL Server and returns the `Client`.
    ///
    /// A fresh client is created on every call; the caller is responsible
    /// for dropping it when the operation completes.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Client<Ready>)` on a successful TCP handshake and login.
    ///
    /// # Errors
    ///
    /// Returns an error if the server is unreachable or authentication fails.
    async fn get_client(&self) -> Result<Client<Ready>> {
        let config: Config = self.build_config();

        // Establish a new connection using the built config
        let client: Client<Ready> = Client::connect(config)
            .await
            .context("[SqlServerRepositoryImpl::get_client] Failed to connect to SQL Server")?;

        Ok(client)
    }
}

#[async_trait]
impl SqlServerRepository for SqlServerRepositoryImpl {
    /// Verifies connectivity by opening and immediately dropping a client.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection is established successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    async fn test_connection(&self) -> Result<()> {
        // Attempt to open a connection; drop it right after to free the resource
        let _client: Client<Ready> = self.get_client().await?;

        info!("[SqlServerRepositoryImpl::test_connection] SQL Server connection established successfully");

        Ok(())
    }
}
