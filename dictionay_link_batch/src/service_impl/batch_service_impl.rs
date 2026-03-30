use async_trait::async_trait;
use getset::{Getters};

use crate::{common::*, repository::sqlserver_repository};

use crate::service::batch_service::*;

use crate::repository::sqlserver_repository::*;

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct BatchServiceImpl<S>
where 
    S: SqlServerRepository
{
    sql_server_repo: Arc<S>
}

impl<S> BatchServiceImpl<S>
where 
    S: SqlServerRepository
{
    
}



#[async_trait]
impl<S> BatchService for BatchServiceImpl<S>
where
    S: SqlServerRepository + Send + Sync + 'static
{

}