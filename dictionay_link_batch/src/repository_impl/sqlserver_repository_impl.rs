//! SQL Server repository implementation.
//!
//! This module provides the data access layer for SQL Server database operations
//! using `mssql-client` as the underlying driver.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │                  SqlServerRepositoryImpl                         │
//! ├──────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────────────────────────────────────────────────┐    │
//! │  │         Connection Config (built from env vars)          │    │
//! │  │              host / port / database / auth               │    │
//! │  └──────────────────────────────────────────────────────────┘    │
//! │                            │                                     │
//! │              ┌─────────────┴─────────────┐                       │
//! │              ▼                           ▼                       │
//! │     ┌─────────────────┐       ┌─────────────────┐                │
//! │     │  get_client()   │       │test_connection() │               │
//! │     │ (per operation) │       │  (health check)  │               │
//! │     └─────────────────┘       └─────────────────┘                │
//! │                                                                  │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Connection Strategy
//!
//! A new `Client` is created per operation via `get_client()`.
//! Connection settings are cloned from [`APP_CONFIG`] in `new()` and stored as fields.
//!
//! # Environment Variables
//!
//! All variables are loaded through [`AppConfig`]. See [`app_config`] for the full list.

use crate::common::*;
use crate::config::app_config::*;
use crate::dtos::sample_proc_dto::{SampleProcRequestDto, SampleProcResponseDto};
use crate::repository::sqlserver_repository::SqlServerRepository;

use async_trait::async_trait;
use getset::Getters;
use mssql_client::{Client, Config, Credentials, state::Ready};

/// Concrete implementation of the SQL Server repository.
///
/// `SqlServerRepositoryImpl` clones connection settings from the global [`APP_CONFIG`]
/// at construction time and builds a fresh `Client` for each operation.
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
/// let repo = SqlServerRepositoryImpl::new();
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
    /// Creates a new `SqlServerRepositoryImpl` by cloning connection settings
    /// from the global [`APP_CONFIG`].
    ///
    /// `APP_CONFIG` is guaranteed to be fully initialized before this is called
    /// because `main()` accesses it before constructing the repository.
    pub fn new() -> Self {
        // Clone connection settings from the already-validated global config
        let cfg: &AppConfig = &*APP_CONFIG;

        Self {
            server:     cfg.mssql_server.clone(),
            port:       cfg.mssql_port,
            database:   cfg.mssql_database.clone(),
            username:   cfg.mssql_username.clone(),
            password:   cfg.mssql_password.clone(),
            trust_cert: cfg.mssql_trust_server_certificate,
        }
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
            .credentials(Credentials::sql_server(
                self.username.clone(),
                self.password.clone(),
            ))
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

        info!(
            "[SqlServerRepositoryImpl::test_connection] SQL Server connection established successfully"
        );

        Ok(())
    }

    /// Executes `usp_sample_proc` and maps each returned row to [`SampleProcResponseDto`].
    ///
    /// Parameters are passed as positional placeholders (`@p1`, `@p2`) via
    /// `sp_executesql` so the values are never interpolated into the SQL string,
    /// preventing SQL injection.
    ///
    /// Column mapping (by index):
    /// | Index | Column           | Type           |
    /// |-------|------------------|----------------|
    /// | 0     | `result_code`    | `INT`          |
    /// | 1     | `result_message` | `NVARCHAR`     |
    /// | 2     | `result_value`   | `FLOAT` / NULL |
    async fn call_proc_sample(
        &self,
        req: &SampleProcRequestDto,
    ) -> Result<Vec<SampleProcResponseDto>> {
        let mut client: Client<Ready> = self.get_client().await?;

        // Positional parameters: @p1 → input_id, @p2 → input_name
        let rows = client
            .query(
                "EXEC usp_sample_proc @input_id = @p1, @input_name = @p2",
                &[&req.input_id, &req.input_name.as_str()],
            )
            .await
            .context("[SqlServerRepositoryImpl::call_proc_sample] Failed to execute usp_sample_proc")?;

        let mut result: Vec<SampleProcResponseDto> = Vec::new();

        for row_result in rows {
            let row = row_result
                .context("[SqlServerRepositoryImpl::call_proc_sample] Failed to read row")?;

            // Non-nullable columns: use get() which returns Result<T, TypeError>
            let result_code: i32 = row
                .get(0)
                .context("[SqlServerRepositoryImpl::call_proc_sample] Failed to read result_code (index 0)")?;

            let result_message: String = row
                .get(1)
                .context("[SqlServerRepositoryImpl::call_proc_sample] Failed to read result_message (index 1)")?;

            // Nullable column: use try_get() which returns Option<T>
            let result_value: Option<f64> = row.try_get(2);

            result.push(SampleProcResponseDto {
                result_code,
                result_message,
                result_value,
            });
        }

        info!(
            "[SqlServerRepositoryImpl::call_proc_sample] usp_sample_proc returned {} row(s)",
            result.len()
        );

        Ok(result)
    }
}
