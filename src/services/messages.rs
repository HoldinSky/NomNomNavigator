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
#[rtype(result = "QueryResult<Dish>")]
pub struct FetchDish(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Dish>>")]
pub struct FetchDishes;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<(String, i32)>>")]
pub struct FetchDishIngredients(pub i64);

/// returns id of newly created order
#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct FirstDish {
    pub table_id: i64,
    pub dish_id: i64
}

#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct AddDishToOrder {
    pub order_id: i64,
    pub dish_id: i64,
}

#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct DecrementDishInOrder {
    pub order_id: i64,
    pub dish_id: i64
}

#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct DeleteDishFromOrder {
    pub order_id: i64,
    pub dish_id: i64
}

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct ConfirmOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct PayForOrder(pub i64);