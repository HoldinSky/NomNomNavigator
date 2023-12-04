use actix::Message;
use diesel::QueryResult;

use crate::services::db_models::Waiter;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Waiter>>")]
pub struct FetchWaiters;