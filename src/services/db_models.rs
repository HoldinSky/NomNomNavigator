#![allow(unused)]
#![allow(clippy::all)]

use std::fmt::Debug;

use chrono::{DateTime, NaiveDate, serde::ts_seconds_option, Utc};
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::types::DishType;

#[derive(Queryable, Debug, Serialize)]
pub struct DishToOrder {
    pub id: i64,
    pub dish_id: i64,
    pub order_id: i64,
    pub dish_price: i32,
    pub count: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct DishToProduct {
    pub id: i64,
    pub dish_id: i64,
    pub product_id: i64,
    pub weight_g: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct Dish {
    pub id: i64,
    pub name: String,
    pub type_: DishType,
    pub portion_weight_g: i32,
    pub price: i32,
    pub approx_cook_time_s: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Order {
    pub id: i64,
    pub table_id: i64,
    pub total_cost: i32,
    pub is_confirmed: bool,
    pub is_paid: bool
}

#[derive(Queryable, Debug, Serialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub in_stock_g: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Table {
    pub id: i64,
    pub seat_count: i32,
    pub is_occupied: bool,
    #[serde(with = "ts_seconds_option")]
    pub reserved_at: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,
    pub waiter_id: Option<i64>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Waiter {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Stats {
    pub id: i64,
    pub day: NaiveDate,
    pub income: i32,
}
