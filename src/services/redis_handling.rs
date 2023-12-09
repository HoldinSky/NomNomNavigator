use actix::Addr;
use chrono::NaiveDate;
use redis::RedisError;
use serde::Serialize;

use crate::services::db_models::Dish;
use crate::services::db_utils::PgActor;
use crate::services::messages::FetchDishIngredients;
use crate::types::MENU_KEY;
use crate::types::ACTIVE_MENU_KEY;

#[derive(Debug, Clone, Serialize)]
struct RedisDish {
    dish: Dish,
    ingredients: Vec<(String, i32)>,
}

pub async fn put_menu_to_db(pg_db: Addr<PgActor>, redis_db: &redis::Client, dishes: Vec<Dish>, date: NaiveDate) -> Result<String, String> {
    let menu_json = match serde_json::to_string(&dishes) {
        Ok(menu) => menu,
        Err(_) => return Err("Failed to compose JSON object of menu".into())
    };

    let mut conn = match redis_db.get_connection() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    let menu_key = format!("{MENU_KEY}_{date}");

    redis::cmd("SET")
        .arg(&menu_key)
        .arg(menu_json)
        .execute(&mut conn);

    for dish in dishes {
        match pg_db.send(FetchDishIngredients(dish.id)).await {
            Ok(Ok(resp)) => {
                let redis_dish = RedisDish { dish: dish.clone(), ingredients: resp };

                if let Ok(dish_entry) = serde_json::to_string(&redis_dish) {
                    redis::cmd("SET")
                        .arg(format!("{}_dish-{}", &menu_key, dish.id))
                        .arg(dish_entry)
                        .execute(&mut conn);
                };
            }
            Err(_) => return Err("There is no dish_to_product records for this dish id".into()),
            _ => return Err("Unable to get dish ingredients".into())
        }
    }

    Ok(menu_key)
}

pub fn set_active_menu(db: &redis::Client, date: NaiveDate) -> Result<(), String> {
    let mut conn = match db.get_connection(){
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    redis::cmd("SET").arg(ACTIVE_MENU_KEY).arg(format!("{MENU_KEY}_{date}")).execute(&mut conn);

    Ok(())
}

pub fn get_menu(db: &redis::Client) -> Result<String, String> {
    let mut conn = match db.get_connection() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    match redis::cmd("GET").arg(ACTIVE_MENU_KEY).query::<String>(&mut conn) {
        Ok(active_menu_key) => {
            match redis::cmd("GET").arg(active_menu_key).query::<String>(&mut conn) {
                Ok(menu_json) => Ok(menu_json),
                Err(_) => Err("Failed to get JSON object of menu from redis db".into())
            }
        },
        Err(_) => Err("Failed to get value of active menu".into())
    }
}