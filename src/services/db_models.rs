
#![allow(unused)]
#![allow(clippy::all)]

use chrono::{DateTime, NaiveDateTime, serde::ts_seconds_option, Utc};
use diesel::Queryable;
use diesel::sql_types::Timestamp;
use serde::Serialize;

#[derive(Queryable, Debug, Serialize)]
pub struct RestaurantTable {
    pub id: i64,
    pub seat_count: i32,
    pub is_occupied: bool,
    #[serde(with = "ts_seconds_option")]
    pub reserved_at: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,
    pub waiter_id: Option<i64>
}

#[derive(Queryable, Debug, Serialize)]
pub struct Waiter {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool
}

#[derive(Queryable, Debug, Serialize)]
pub struct ClientOrder {
    pub id: i64,
    pub table_id: Option<i64>,
    pub total_cost: i32
}

#[derive(Queryable, Debug, Serialize)]
pub struct Dish {
    pub id: i64,
    pub name: String,
    pub type_: DishType,
    pub portion_weight: u32,
    pub cost: u32,
    pub approximate_cooking_time: i32
}

#[derive(Queryable, Debug)]
pub struct DishToOrder {
    pub id: i64,
    pub dish_id: Option<i64>,
    pub order_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub enum DishType {
    Main,
    Appetizer,
    Garnish,
    Cold,
    Salads,
    Drinks,
    Alcohol
}
