use actix::Message;
use diesel::QueryResult;

use crate::services::db_models::Waiter;
use crate::services::db_models::Dish;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Waiter>>")]
pub struct FetchWaiters;

#[derive(Message)]
#[rtype(result = "QueryResult<Waiter>")]
pub struct AddWaiter {
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool
}

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Dish>>")]
pub struct GetAllDishes;