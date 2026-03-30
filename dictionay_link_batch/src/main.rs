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

mod controller;

mod repository;

mod repository_impl;

mod service;

mod service_impl;

#[tokio::main]
async fn main() {
    
    dotenv().ok();
    set_global_logger();

    info!("test");
}