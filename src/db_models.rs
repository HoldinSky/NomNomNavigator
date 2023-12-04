
#![allow(unused)]
#![allow(clippy::all)]

use chrono::{DateTime, Utc};
use diesel::Queryable;
use diesel::sql_types::Timestamp;
use serde::Serialize;

#[derive(Queryable, Debug, Serialize)]
pub struct RestaurantTable {
    pub id: i64,
    pub seat_count: u32,
    pub is_occupied: bool,
    #[serde(skip)]
    pub reserved_at: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,
    pub waiter_id: i64
}

#[derive(Queryable, Debug, Serialize)]
pub struct Waiter {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool
}

#[derive(Queryable, Debug, Serialize)]
pub struct Order {
    pub id: i64,
    pub table_id: i64,
    pub total_cost: u64
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

#[derive(Queryable, Debug, Serialize)]
pub struct Dish {
    pub id: i64,
    pub name: String,
    pub dish_type: DishType,
    pub portion_weight: u32,
    pub cost: u32,
    #[serde(skip)]
    pub approximate_cooking_time: Timestamp
}