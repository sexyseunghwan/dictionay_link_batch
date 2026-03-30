/*
Author      : Seunghwan Shin
Create date : 2026-00-00
Description :

History     : 2026-00-00 Seunghwan Shin       # [v.1.0.0] first create.
*/
mod common;
use common::*;

mod util_modules;
use util_modules::logger_utils::*;

mod config;
use config::app_config::APP_CONFIG;

mod model;

mod dtos;

mod controller;
use controller::main_controller::MainController;

mod repository;

mod repository_impl;
use repository_impl::sqlserver_repository_impl::SqlServerRepositoryImpl;

mod service;

mod service_impl;
use service_impl::batch_service_impl::BatchServiceImpl;
use service_impl::cli_service_impl::CliServiceImpl;


#[tokio::main]
async fn main() {

    dotenv().ok();
    set_global_logger();

    // Trigger APP_CONFIG initialization early so misconfiguration is caught at startup
    let _ = &*APP_CONFIG;

    let sql_repo: Arc<SqlServerRepositoryImpl> = Arc::new(SqlServerRepositoryImpl::new());

    let batch_service: BatchServiceImpl<SqlServerRepositoryImpl> = BatchServiceImpl::new(sql_repo);
    let cli_service: CliServiceImpl = CliServiceImpl::new(APP_CONFIG.cli_infos_toml.clone());
    let controller: MainController<BatchServiceImpl<SqlServerRepositoryImpl>, CliServiceImpl> =
        MainController::new(batch_service, cli_service);

    if let Err(e) = controller.main_task().await {
        error!("[main] Unexpected error during execution: {}", e);
    }
}
