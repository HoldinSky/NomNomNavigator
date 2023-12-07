use chrono::NaiveDate;
use diesel::Insertable;
use serde::Serialize;

use crate::schema::dish_to_order;
use crate::schema::waiters;
use crate::schema::orders;
use crate::schema::stats;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = waiters)]
pub struct NewWaiter {
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = dish_to_order)]
pub struct OrderDish {
    pub dish_id: i64,
    pub order_id: i64,
    pub dish_price: i32,
    pub count: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub table_id: i64,
    pub total_cost: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = stats)]
pub struct NewStats {
    pub day: NaiveDate,
    pub income: i32,
}