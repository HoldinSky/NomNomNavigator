use serde::Serialize;
use crate::services::db_models::Dish;

#[derive(Debug, Clone, Serialize)]
pub struct RedisDish {
    id: i64,
    name: String,
}

pub fn put_menu_to_db(db: &redis::Client, dishes: &mut Vec<Dish>) -> Result<(), String> {
    dishes.truncate(dishes.len() / 2);

    let mut dishes: Vec<RedisDish> = dishes.iter_mut().map(|dish| {
        RedisDish { id: dish.id, name: dish.name.clone() }
    }).collect();

    let menu_json = match serde_json::to_string(&dishes) {
        Ok(menu) => menu,
        Err(_) => return Err("Failed to compose JSON object of menu".into())
    };

    let mut conn = match db.get_connection() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to establish connection with redis".into())
    };

    if let Err(_) = redis::cmd("SET").arg("menu").arg(menu_json).query::<()>(&mut conn) {
        return Err("Failed to insert menu into redis".into());
    }

    Ok(())
}