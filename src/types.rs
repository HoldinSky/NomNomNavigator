use std::fmt::{Debug, Display, Formatter};

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::query_builder::QueryId;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow, SqlType};
use serde::ser::StdError;
use serde::{Deserialize, Serialize};

use crate::services::db_models::{Dish, Order};

// Constants

pub const ACTIVE_MENU_KEY: &str = "active-menu";
pub const MENU_KEY: &str = "menu";

// actual User Defined Types

#[derive(Debug)]
pub struct PoolInitializationError(pub String);

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[diesel(sql_type = Text)]
pub enum DishType {
    Main,
    Appetizer,
    Garnish,
    Cold,
    Salad,
    Drink,
    Alcohol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisDish {
    pub dish: Dish,
    pub ingredients: Vec<(String, i32)>,
}

#[derive(Deserialize)]
pub struct Ingredient {
    pub id: i64,
    pub used_g: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DishWithCount {
    pub dish: Dish,
    pub count: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderInfo {
    pub order: Order,
    pub dishes: Vec<DishWithCount>,
}

// additional code for types

impl Display for PoolInitializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct UnknownDishType(String);

impl Display for UnknownDishType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.0)
    }
}

impl StdError for UnknownDishType {}

impl ToSql<Text, Pg> for DishType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let value = match self {
            DishType::Main => "main",
            DishType::Appetizer => "appetizer",
            DishType::Garnish => "garnish",
            DishType::Cold => "cold",
            DishType::Salad => "salad",
            DishType::Drink => "drink",
            DishType::Alcohol => "alcohol",
        };

        ToSql::<Text, Pg>::to_sql(value, out)
    }
}

impl FromSql<Text, Pg> for DishType {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match String::from_utf8_lossy(bytes.as_bytes()).as_ref() {
            "main" => Ok(DishType::Main),
            "appetizer" => Ok(DishType::Appetizer),
            "garnish" => Ok(DishType::Garnish),
            "cold" => Ok(DishType::Cold),
            "salad" => Ok(DishType::Salad),
            "drink" => Ok(DishType::Drink),
            "alcohol" => Ok(DishType::Alcohol),
            _ => Err(Box::new(UnknownDishType(
                "Couldn't recognize dish type".into(),
            ))),
        }
    }
}

impl DishType {
    pub fn to_string(self) -> String {
        match self {
            DishType::Main => String::from("main"),
            DishType::Appetizer => String::from("appetizer"),
            DishType::Garnish => String::from("garnish"),
            DishType::Cold => String::from("cold"),
            DishType::Salad => String::from("salad"),
            DishType::Drink => String::from("drink"),
            DishType::Alcohol => String::from("alcohol"),
        }
    }

    pub fn from_string(input: &str) -> Result<Self, String> {
        match input {
            "main" => Ok(DishType::Main),
            "appetizer" => Ok(DishType::Appetizer),
            "garnish" => Ok(DishType::Garnish),
            "cold" => Ok(DishType::Cold),
            "salad" => Ok(DishType::Salad),
            "drink" => Ok(DishType::Drink),
            "alcohol" => Ok(DishType::Alcohol),
            _ => Err(format!("Couldn't recognize dish type: {}", input)),
        }
    }
}
