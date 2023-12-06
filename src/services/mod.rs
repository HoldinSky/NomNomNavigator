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
        let db: Addr<PgActor> = state.pg_db.clone();

        match db.send(FetchWaiters).await {
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
        let db = state.pg_db.clone();

        match db.send(AddWaiter {
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
    use crate::services::db_models::Dish;
    use crate::services::db_utils::AppState;
    use crate::services::messages::FetchDish;
    use crate::services::redis_handling::get_menu;

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
        let db = state.pg_db.clone();

        match db.send(FetchDish(path.into_inner())).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(_)) => HttpResponse::NotFound().json("Dish with that id not found"),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to fetch dish: {err}"))
        }
    }
}

// sub-route "/order"
pub mod order_route {
    use actix_web::{delete, HttpResponse, post, put, Responder};
    use actix_web::web::{Data, Path};
    use crate::services::db_utils::AppState;
    use crate::services::messages::{FirstDish, FetchDish, AddDishToOrder, DecrementDishInOrder, DeleteDishFromOrder};

    #[post("/{table_id}/{dish_id}/first-dish")]
    pub async fn order_dish(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let db = state.pg_db.clone();

        let (table_id, dish_id) = path.into_inner();

        match db.send(FirstDish { table_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Table or order was not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[post("/{order_id}/add/{dish_id}")]
    pub async fn add_dish_to_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let db = state.pg_db.clone();

        let (order_id, dish_id) = path.into_inner();

        match db.send(AddDishToOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order was not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[put("/decrement/{order_id}/{dish_id}")]
    pub async fn decrement_dishes_from_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let db = state.pg_db.clone();

        let (order_id, dish_id) = path.into_inner();

        match db.send(DecrementDishInOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order or dish were not found: {err}")),
            Err(err) => HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
        }
    }

    #[delete("/{order_id}/{dish_id}")]
    pub async fn delete_dish_from_order(state: Data<AppState>, path: Path<(i64, i64)>) -> impl Responder {
        let db = state.pg_db.clone();

        let (order_id, dish_id) = path.into_inner();

        match db.send(DeleteDishFromOrder { order_id, dish_id }).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(format!("Order id: {}", resp)),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order or dish were not found: {err}")),
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
        let pg_db = state.pg_db.clone();
        let redis_db = state.redis_db.clone();

        let mut dishes = match pg_db.send(FetchDishes).await {
            Ok(Ok(resp)) => resp,
            _ => return HttpResponse::InternalServerError().json("Unable to get dishes from sql database")
        };

        match put_menu_to_db(pg_db, &redis_db, &mut dishes).await {
            Ok(key) => HttpResponse::Ok().json(format!("Menu is successfully composed and placed into redis db by the key '{key}'")),
            Err(err) => HttpResponse::InternalServerError().json(err),
        }
    }
}
