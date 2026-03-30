use derive_new::new;

use crate::common::*;

use crate::service::batch_service::BatchService;
use crate::service::cli_service::CliService;
use crate::service::{batch_service, cli_service};


#[derive(Debug, new)]
pub struct MainController<B,C>
where 
    B: BatchService,
    C: CliService
{
    batch_service: B,
    cli_service: C
}

impl<B,C> MainController<B,C>
where 
    B: BatchService,
    C: CliService
{
    
    pub async fn main_task(&self) -> anyhow::Result<()> {



        Ok(())
    }
}