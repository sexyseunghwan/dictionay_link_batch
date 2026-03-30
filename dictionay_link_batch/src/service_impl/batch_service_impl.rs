//! Batch service implementation.
//!
//! This module provides the production implementation of [`BatchService`],
//! dispatching named batch jobs and executing their business logic against
//! the SQL Server repository.

use async_trait::async_trait;
use derive_new::new;
use getset::Getters;

use crate::common::*;
use crate::model::cli_info::CliInfo;
use crate::repository::sqlserver_repository::*;
use crate::service::batch_service::*;


/// Production implementation of [`BatchService`] backed by a SQL Server repository.
///
/// Each batch job is identified by its `cli_name` string and matched to a
/// concrete handler branch inside `run_batch`. To add a new job, declare it
/// in `cli_infos.toml` and add the corresponding `match` arm here.
#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct BatchServiceImpl<S>
where
    S: SqlServerRepository,
{
    /// Shared reference to the SQL Server repository used by all batch jobs.
    sql_server_repo: Arc<S>,
}


impl<S> BatchServiceImpl<S>
where
    S: SqlServerRepository,
{
    async fn migration_elastic_dict_type(&self, cli_info: &CliInfo) -> Result<()> {
        // TODO: implement migration_elastic_dict_type logic
        

        Ok(())
    }
}


#[async_trait]
impl<S> BatchService for BatchServiceImpl<S>
where
    S: SqlServerRepository + Send + Sync + 'static,
{
    /// Dispatches and executes the batch job described by `cli_info`.
    ///
    /// Each arm is responsible for its own error handling and logging.
    /// An unknown `cli_name` is treated as a hard error.
    async fn run_batch(&self, cli_info: &CliInfo) -> Result<()> {
        info!(
            "[BatchServiceImpl::run_batch] Starting batch job: {}",
            cli_info.cli_name
        );
        
        match cli_info.cli_name.as_str() {
            "migration_to_db" => {
                // TODO: implement migration_to_db batch logic
                info!("[BatchServiceImpl::run_batch] migration_to_db completed");
            }
            "ELASTIC_DICT_TYPE_TB_ADD" => {
                self.migration_elastic_dict_type(cli_info).await?;
                info!("[BatchServiceImpl::run_batch] ELASTIC_DICT_TYPE_TB_ADD completed");
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[BatchServiceImpl::run_batch] Unknown batch job name: {}",
                    cli_info.cli_name
                ));
            }
        }

        Ok(())
    }
}
