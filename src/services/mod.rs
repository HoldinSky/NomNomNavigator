use actix_web::{get, HttpResponse, Responder};

pub mod db_models;
pub mod db_utils;
pub mod messages;
pub mod insertable;
pub mod pg_handling;
pub mod redis_handling;

#[get("/")]
pub async fn home_page() -> impl Responder {
    HttpResponse::Ok().body("Rust service prototype")
}

// sub-route "/waiters"
pub mod waiters_route {
    use actix::Addr;
    use actix_web::{get, HttpResponse, post, Responder};
    use actix_web::web::{Data, Json};
    use serde::Deserialize;

    use crate::services::db_utils::{AppState, PgActor};
    use crate::services::messages::{AddWaiter, FetchWaiters};

    #[get("/all")]
    pub async fn fetch_waiters(state: Data<AppState>) -> impl Responder {
        match state.pg_db.send(FetchWaiters).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(_)) => HttpResponse::NotFound().json("Waiters not found"),
            _ => HttpResponse::InternalServerError().json("Unable to retrieve waiters"),
        }
    }

    #[derive(Deserialize)]
    pub struct AddWaiterBody {
        pub first_name: String,
        pub last_name: String,
        pub is_admin: bool,
    }

    #[post("/add")]
    pub async fn add_waiter(state: Data<AppState>, body: Json<AddWaiterBody>) -> impl Responder {
        match state.pg_db.send(AddWaiter {
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            is_admin: body.is_admin,
        }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(err)) => HttpResponse::InternalServerError().json(err.to_string()),
            _ => HttpResponse::InternalServerError().json("Unable to insert new waiter")
        }
    }
}

// sub-route "/menu"
pub mod menu_route {
    use actix_web::{get, HttpResponse, post, Responder};
    use actix_web::web::{Data, Path};
    use redis::FromRedisValue;
    use serde::Deserialize;
    use crate::services::db_models::Dish;
    use crate::services::db_utils::AppState;
    use crate::services::messages::{FetchSpecificDishes, CreateOrder, FetchDish, FetchDishes};
    use crate::services::redis_handling::{get_menu, put_menu_to_db};

    #[get("")]
    pub async fn view_menu(state: Data<AppState>) -> impl Responder {
        match get_menu(&state.redis_db) {
            Ok(menu_json) => {
                let menu = serde_json::from_str::<Vec<Dish>>(&menu_json).unwrap();
                HttpResponse::Ok().json(menu)
            }
            Err(err) => HttpResponse::InternalServerError().json(err)
        }
    }

    #[get("/dish/{id}")]
    pub async fn get_dish(state: Data<AppState>, path: Path<(i64)>) -> impl Responder {
        match state.pg_db.send(FetchDish(path.into_inner())).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(_)) => HttpResponse::NotFound().json("Dish with that id not found"),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to fetch dish: {err}"))
        }
    }

    #[get("/all-dishes")]
    pub async fn get_dishes(state: Data<AppState>) -> impl Responder {
        match state.pg_db.send(FetchDishes).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(_)) => HttpResponse::NotFound().json("No dishes found"),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to fetch dishes: {err}"))
        }
    }

    #[post("/create-new")]
    pub async fn create_menu(state: Data<AppState>, body: actix_web::web::Bytes) -> impl Responder {
        let json_input = match String::from_utf8(Vec::from(body.as_ref())) {
            Ok(val) => val.trim_matches('"').to_owned(),
            Err(err) => return HttpResponse::BadRequest().json("Failed to parse request. Non utf-8 characters")
        };

        let dish_ids: Vec<i64> = match serde_json::from_str(json_input.as_str()) {
            Ok(val) => val,
            Err(err) => return HttpResponse::BadRequest().json("Failed to parse request. Body is not an array of dish ids")
        };

        match state.pg_db.send(FetchSpecificDishes(dish_ids)).await {
            Ok(Ok(resp)) => {
                match put_menu_to_db(state.pg_db.clone(), &state.redis_db, resp).await {
                    Ok(key) => HttpResponse::Ok().json(format!("Successfully created menu with the key {key}")),
                    Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
                }
            },
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Dishes were not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }
}

// sub-route "/order"
pub mod order_route {
    use actix_web::{delete, HttpResponse, post, put, Responder};
    use actix_web::web::{Data, Path};
    use serde::de::IntoDeserializer;
    use crate::services::db_utils::AppState;
    use crate::services::messages::{FetchDish, AddDishToOrder, DecrementDishInOrder, DeleteDishFromOrder, ConfirmOrder, PayForOrder, CreateOrder};

    #[post("/create-for-table/{table_id}")]
    pub async fn create_blank_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match state.pg_db.send(CreateOrder(table_id)).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Id of newly created order: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Table was not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[post("/{order_id}/add/{dish_id}")]
    pub async fn add_dish_to_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state.pg_db.send(AddDishToOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order was not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[put("/{order_id}/decrement/{dish_id}")]
    pub async fn decrement_dishes_from_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state.pg_db.send(DecrementDishInOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order or dish were not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[delete("/{order_id}/{dish_id}")]
    pub async fn delete_dish_from_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state.pg_db.send(DeleteDishFromOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order or dish were not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[post("/{order_id}/confirm")]
    pub async fn confirm_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let order_id = path.into_inner();

        match state.pg_db.send(ConfirmOrder(order_id)).await {
            Ok(Ok(_)) => HttpResponse::Ok().json(format!("Order with id {order_id} is successfully confirmed")),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[post("/{order_id}/pay")]
    pub async fn pay_for_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let order_id = path.into_inner();

        match state.pg_db.send(PayForOrder(order_id)).await {
            Ok(Ok(_)) => HttpResponse::Ok().json(format!("Order with id {order_id} is successfully paid")),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }
}

// sub-route "/test"
pub mod test_route {
    use actix_web::{get, HttpResponse, post, Responder};
    use actix_web::web::Data;
    use redis::Commands;
    use crate::services::db_utils::AppState;
    use crate::services::messages::{AddWaiter, FetchDishes};
    use crate::services::redis_handling::put_menu_to_db;

    #[get("/healthcheck")]
    pub async fn healthcheck() -> impl Responder {
        HttpResponse::Ok().body("I'm alive!")
    }

    #[post("/create-mock-menu")]
    pub async fn create_mock_menu(state: Data<AppState>) -> impl Responder {
        let mut dishes = match state.pg_db.send(FetchDishes).await {
            Ok(Ok(resp)) => resp,
            _ => return HttpResponse::InternalServerError().json("Unable to get dishes from sql database")
        };

        match put_menu_to_db(state.pg_db.clone(), &state.redis_db, dishes).await {
            Ok(key) => HttpResponse::Ok().json(format!("Menu is successfully composed and placed into redis db by the key '{key}'")),
            Err(err) => HttpResponse::InternalServerError().json(err),
        }
    }
}
