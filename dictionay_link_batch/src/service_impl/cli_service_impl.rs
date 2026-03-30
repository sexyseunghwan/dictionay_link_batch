use getset::{Getters};
use async_trait::async_trait;

use crate::common::*;

use crate::service::cli_service::*;


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct CliServiceImpl {
    
}
