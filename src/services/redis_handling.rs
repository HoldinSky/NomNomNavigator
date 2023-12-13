use actix::Addr;
use chrono::NaiveDate;
use redis::RedisError;
use serde::Serialize;

use crate::services::db_models::Dish;
use crate::services::db_utils::PgActor;
use crate::services::messages::FetchDishIngredients;
use crate::types::ACTIVE_MENU_KEY;
use crate::types::{RedisDish, MENU_KEY};

pub struct RedisHandler {
    db: redis::Client,
}

impl RedisHandler {
    pub fn new(db: redis::Client) -> Self {
        Self { db }
    }

    pub async fn save_new_menu(
        &self,
        pg_db: Addr<PgActor>,
        dishes: Vec<Dish>,
        date: &NaiveDate,
    ) -> Result<String, String> {
        let menu_json = serde_json::to_string(&dishes)
            .map_err(|_| "Failed to compose JSON object of menu".to_owned())?;

        let mut conn = self
            .db
            .get_connection()
            .map_err(|_| "Failed to establish connection with redis".to_owned())?;

        let menu_key = format!("{MENU_KEY}_{date}");

        redis::cmd("SET")
            .arg(&menu_key)
            .arg(menu_json)
            .query::<()>(&mut conn)
            .map_err(|_| "Failed to set JSON object as menu".to_owned())?;

        for dish in dishes {
            match pg_db.send(FetchDishIngredients(dish.id)).await {
                Ok(Ok(resp)) => {
                    let redis_dish = RedisDish {
                        dish: dish.clone(),
                        ingredients: resp,
                    };

                    if let Ok(dish_entry) = serde_json::to_string(&redis_dish) {
                        redis::cmd("SET")
                            .arg(format!("{}_dish-{}", &menu_key, dish.id))
                            .arg(dish_entry)
                            .query::<()>(&mut conn)
                            .map_err(|_| "Failed to set dish for new menu".to_owned())?
                    };
                }
                Err(_) => {
                    return Err("There is no dish_to_product records for this dish id".to_owned())
                }
                _ => return Err("Unable to get dish ingredients".to_owned()),
            }
        }

        Ok(menu_key)
    }

    pub fn set_active_menu(&self, date: &NaiveDate) -> Result<(), String> {
        let mut conn = self
            .db
            .get_connection()
            .map_err(|_| "Failed to establish connection with redis".to_owned())?;

        redis::cmd("SET")
            .arg(ACTIVE_MENU_KEY)
            .arg(format!("{MENU_KEY}_{date}"))
            .query::<()>(&mut conn)
            .map_err(|_| "Failed to set active menu".to_owned())
    }

    pub fn get_menu(&self) -> Result<String, String> {
        let mut conn = self
            .db
            .get_connection()
            .map_err(|_| "Failed to establish connection with redis".to_owned())?;

        match redis::cmd("GET")
            .arg(ACTIVE_MENU_KEY)
            .query::<String>(&mut conn)
        {
            Ok(active_menu_key) => redis::cmd("GET")
                .arg(active_menu_key)
                .query::<String>(&mut conn)
                .map_err(|_| "Failed to get JSON object of menu from redis db".to_owned()),
            Err(_) => Err("Failed to get value of active menu".to_owned()),
        }
    }

    pub fn delete_menu(&self, date: &NaiveDate) -> Result<(), String> {
        let mut conn = self
            .db
            .get_connection()
            .map_err(|_| "Failed to establish connection with redis".to_owned())?;

        let menu_to_delete = format!("{MENU_KEY}_{date}");

        let dishes_to_delete: Vec<String> = redis::cmd("keys")
            .arg(format!("{menu_to_delete}_*"))
            .query(&mut conn)
            .map_err(|_| "Failed to get dishes from specified menu".to_owned())?;

        let mut pipeline = redis::pipe();
        let mut pipeline = pipeline.atomic();

        for dish in dishes_to_delete {
            pipeline.cmd("DEL").arg(dish).ignore();
        }

        redis::cmd("DEL")
            .arg(&menu_to_delete)
            .query::<()>(&mut conn)
            .map_err(|_| "Failed to delete specified menu".to_owned())?;

        pipeline
            .query(&mut conn)
            .map_err(|_| "Failed to delete dishes of specified menu".to_owned())?;

        if let Ok(active) = redis::cmd("GET")
            .arg(ACTIVE_MENU_KEY)
            .query::<String>(&mut conn)
        {
            if active == menu_to_delete {
                redis::cmd("DEL")
                    .arg(ACTIVE_MENU_KEY)
                    .query::<()>(&mut conn)
                    .map_err(|_| "Failed to delete active menu as long as it points to deleted")?;
            }
        }

        Ok(())
    }

    pub fn get_dish(&self, dish_id: i64) -> Result<String, String> {
        let mut conn = self
            .db
            .get_connection()
            .map_err(|_| "Failed to establish connection with redis".to_owned())?;

        let dish_prefix = redis::cmd("GET")
            .arg(ACTIVE_MENU_KEY)
            .query::<String>(&mut conn)
            .map_err(|_| "Failed to get currently active menu".to_owned())?;

        let dish_key = format!("{dish_prefix}_dish-{dish_id}");

        redis::cmd("GET")
            .arg(dish_key)
            .query::<String>(&mut conn)
            .map_err(|_| "Failed to get specified dish from active menu".to_owned())
    }
}
