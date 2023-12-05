use crate::schema::waiters;
use diesel::Insertable;
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = waiters)]
pub struct NewWaiter {
    pub first_name: String,
    pub last_name: String,
    pub is_admin: bool
}