#![allow(unused)]
#![allow(clippy::all)]

use crate::types::DishType;
use chrono::{serde::ts_seconds_option, DateTime, NaiveDate, NaiveDateTime, Utc};
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Queryable, Debug, Serialize)]
pub struct DishToOrder {
    pub id: i64,
    pub dish_id: i64,
    pub order_id: i64,
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

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub table_id: i64,
    pub total_cost: i32,
    pub is_confirmed: bool,
    pub is_paid: bool,
    pub is_cooked: bool,
    pub created_at: NaiveDateTime,
    pub cooked_at: Option<NaiveDateTime>,
    pub confirmed_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub in_stock_g: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Stats {
    pub id: i64,
    pub day: NaiveDate,
    pub income: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Table {
    pub id: i64,
    pub seat_count: i32,
    pub is_occupied: bool,
    pub reserved_at: Option<NaiveDateTime>,
    pub reserved_by: Option<String>,
    pub waiter_id: Option<i64>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Waiter {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Worker {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub role_id: i32,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct WorkerAuth {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub token: Option<String>,
    pub worker_id: i32,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct WorkerRole {
    pub id: i32,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}
