use std::collections::HashSet;
use actix::Addr;
use chrono::Datelike;
use redis::Commands;
use serde::Serialize;
use crate::services::db_models::Dish;
use crate::services::db_utils::PgActor;
use crate::services::messages::{FetchDish, FetchDishIngredients};
use crate::types::MENU_KEY;

#[derive(Debug, Clone, Serialize)]
struct RedisDish {
    dish: Dish,
    ingredients: Vec<(String, i32)>,
}

pub async fn put_menu_to_db(pg_db: Addr<PgActor>, redis_db: &redis::Client, dishes: &mut Vec<Dish>) -> Result<String, String> {
    let mut unique_dish_types = HashSet::new();
    dishes.retain(|dish| unique_dish_types.insert(dish.type_.clone()));

    let menu_json = match serde_json::to_string(&dishes) {
        Ok(menu) => menu,
        Err(_) => return Err("Failed to compose JSON object of menu".into())
    };

    let mut conn = match redis_db.get_connection() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    if let Err(_) = redis::cmd("SETEX")
        .arg(MENU_KEY)
        .arg(format!("{}", chrono::Duration::days(1).num_seconds()))
        .arg(menu_json)
        .query::<()>(&mut conn) {
        return Err("Failed to insert menu into redis".into());
    }

    for dish in dishes {
        match pg_db.send(FetchDishIngredients(dish.id)).await {
            Ok(Ok(resp)) => {
                let redis_dish = RedisDish { dish: dish.clone(), ingredients: resp };

                if let Ok(dish_entry) = serde_json::to_string(&redis_dish) {
                    redis::cmd("SETEX")
                        .arg(format!("{}-dish-{}", MENU_KEY, dish.id))
                        .arg(format!("{}", chrono::Duration::days(1).num_seconds()))
                        .arg(dish_entry)
                        .query::<()>(&mut conn);
                };
            }
            Err(_) => return Err("There is no dish_to_product records for this dish id".into()),
            _ => return Err("Unable to get dish ingredients".into())
        }
    }

    Ok(MENU_KEY.into())
}

pub fn get_menu(db: &redis::Client) -> Result<String, String> {
    let mut conn = match db.get_connection() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    match redis::cmd("GET").arg(MENU_KEY).query::<String>(&mut conn) {
        Ok(resp) => Ok(resp),
        Err(err) => Err("Failed to get JSON object of menu from redis db".into())
    }
}