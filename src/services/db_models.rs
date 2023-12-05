#![allow(unused)]
#![allow(clippy::all)]

use std::fmt::{Debug, Formatter};
use chrono::{DateTime, serde::ts_seconds_option, Utc};
use diesel::{AsExpression, FromSqlRow, Queryable, Selectable};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Timestamp};
use serde::Serialize;
use crate::schema::dishes::dsl::dishes;
use crate::types::DishType;

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

#[derive(Queryable, Debug, Serialize)]
pub struct Dish {
    pub id: i64,
    pub name: String,
    pub type_: DishType,
    pub portion_weight_g: i32,
    pub cost: i32,
    pub approx_cook_time: i32,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Order {
    pub id: i64,
    pub table_id: Option<i64>,
    pub total_cost: i32,
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
