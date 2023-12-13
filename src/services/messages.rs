use actix::Message;
use diesel::QueryResult;

use crate::services::db_models::Dish;
use crate::services::db_models::Waiter;
use crate::types::{DishType, Ingredient, OrderInfo};

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Waiter>>")]
pub struct FetchWaiters;

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct AddWaiter {
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
}

#[derive(Message)]
#[rtype(result = "QueryResult<Dish>")]
pub struct FetchDish(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Dish>>")]
pub struct FetchDishes;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Dish>>")]
pub struct FetchSpecificDishes(pub Vec<i64>);

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<(String, i32)>>")]
pub struct FetchDishIngredients(pub i64);

/// returns id of newly created order
#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct CreateOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<OrderInfo>")]
pub struct FetchOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<OrderInfo>>")]
pub struct FetchOrders;

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
    pub dish_id: i64,
}

#[derive(Message)]
#[rtype(result = "QueryResult<i64>")]
pub struct DeleteDishFromOrder {
    pub order_id: i64,
    pub dish_id: i64,
}

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct ConfirmOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct CookOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<()>")]
pub struct PayForOrder(pub i64);

#[derive(Message)]
#[rtype(result = "QueryResult<Dish>")]
pub struct CreateDish {
    pub dish_name: String,
    pub dish_type: DishType,
    pub price: i32,
    pub approx_cook_time_s: i32,
    pub portion_weight_g: i32,
    pub ingredients: Vec<Ingredient>,
}
