use chrono::NaiveDate;
use chrono::NaiveDateTime;
use diesel::Insertable;
use serde::Serialize;

use crate::schema::dish_to_order;
use crate::schema::dish_to_product;
use crate::schema::dishes;
use crate::schema::orders;
use crate::schema::stats;
use crate::schema::waiters;
use crate::types::{DishType, Ingredient};

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = waiters)]
pub struct NewWaiter {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = dish_to_order)]
pub struct OrderDish {
    pub dish_id: i64,
    pub order_id: i64,
    pub count: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub table_id: i64,
    pub total_cost: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = stats)]
pub struct NewStats {
    pub day: NaiveDate,
    pub income: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = dishes)]
pub struct NewDish {
    pub name: String,
    pub type_: String,
    pub approx_cook_time_s: i32,
    pub portion_weight_g: i32,
    pub price: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = dish_to_product)]
pub struct DishProductMapping {
    pub dish_id: i64,
    pub product_id: i64,
    pub weight_g: i32,
}
