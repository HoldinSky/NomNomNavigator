use actix_web::{get, HttpResponse, Responder};

pub mod db_models;
pub mod db_utils;
pub mod insertable;
pub mod messages;
pub mod pg_handling;
pub mod redis_handling;

#[get("/")]
pub async fn home_page() -> impl Responder {
    HttpResponse::Ok().body("Rust service prototype")
}

// sub-route "/waiters"
pub mod waiters_route {
    use actix::Addr;
    use actix_web::web::{Data, Json};
    use actix_web::{get, post, HttpResponse, Responder};
    use serde::Deserialize;

    use crate::services::db_utils::{AppState, PgActor};
    use crate::services::messages::{AddWaiter, FetchWaiters};

    #[get("/all")]
    pub async fn fetch_waiters(state: Data<AppState>) -> impl Responder {
        match state.pg_db.send(FetchWaiters).await {
            Ok(Ok(waiters)) => HttpResponse::Ok().json(waiters),
            Ok(Err(_)) => HttpResponse::NotFound().json("Waiters not found"),
            _ => HttpResponse::InternalServerError().json("Unable to retrieve waiters"),
        }
    }

    #[derive(Deserialize)]
    struct AddWaiterBody {
        first_name: String,
        last_name: String,
    }

    #[post("/add")]
    pub async fn add_waiter(state: Data<AppState>, body: Json<AddWaiterBody>) -> impl Responder {
        match state
            .pg_db
            .send(AddWaiter {
                first_name: body.first_name.clone(),
                last_name: body.last_name.clone(),
                is_admin: false,
            })
            .await
        {
            Ok(Ok(_)) => {
                HttpResponse::Ok().json("New waiter is successfully added to the database")
            }
            Ok(Err(err)) => HttpResponse::InternalServerError().json(err.to_string()),
            _ => HttpResponse::InternalServerError().json("Unable to insert new waiter"),
        }
    }
}

// sub-route "/dishes"
pub mod dishes_route {
    use crate::services::db_utils::AppState;
    use crate::services::messages::{AddWaiter, CreateDish};
    use crate::types::{DishType, Ingredient};
    use actix_web::web::{Data, Json};
    use actix_web::{post, HttpResponse, Responder};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct CreateDishBody {
        dish_name: String,
        dish_type: DishType,
        price: i32,
        approx_cook_time_s: i32,
        portion_weight_g: i32,
        ingredients: Vec<Ingredient>,
    }

    #[post("/add")]
    pub async fn create_dish(state: Data<AppState>, body: Json<CreateDishBody>) -> impl Responder {
        let body = body.into_inner();

        match state
            .pg_db
            .send(CreateDish {
                dish_name: body.dish_name,
                dish_type: body.dish_type,
                price: body.price,
                approx_cook_time_s: body.approx_cook_time_s,
                portion_weight_g: body.portion_weight_g,
                ingredients: body.ingredients,
            })
            .await
        {
            Ok(Ok(dish)) => HttpResponse::Ok().json(dish),
            Ok(Err(err)) => HttpResponse::InternalServerError().json(err.to_string()),
            _ => HttpResponse::InternalServerError().json("Unable to insert new waiter"),
        }
    }
}

// sub-route "/menu"
pub mod menu_route {
    use crate::services::db_models::Dish;
    use crate::services::db_utils::AppState;
    use crate::services::messages::{CreateOrder, FetchDish, FetchDishes, FetchSpecificDishes};
    use actix_web::http::header::{HeaderName, HeaderValue, CONTENT_TYPE};
    use actix_web::web::{Bytes, Data, Json, Path};
    use actix_web::{delete, get, post, put, HttpResponse, Responder};
    use chrono::NaiveDate;
    use redis::FromRedisValue;
    use serde::Deserialize;
    use std::collections::HashSet;

    #[get("")]
    pub async fn view_menu(state: Data<AppState>) -> impl Responder {
        match state.redis_handler.get_menu() {
            Ok(menu_json) => HttpResponse::Ok()
                .append_header(("Content-Type", "application/json"))
                .body(menu_json),
            Err(err) => HttpResponse::InternalServerError().json(err),
        }
    }

    #[get("/dish/{id}")]
    pub async fn get_dish(state: Data<AppState>, path: Path<(i64)>) -> impl Responder {
        match state.redis_handler.get_dish(path.into_inner()) {
            Ok(redis_dish_json) => HttpResponse::Ok()
                .append_header(("Content-Type", "application/json"))
                .body(redis_dish_json),
            Err(err) => HttpResponse::NotFound().json(err),
        }
    }

    #[get("/all-dishes")]
    pub async fn get_dishes(state: Data<AppState>) -> impl Responder {
        match state.pg_db.send(FetchDishes).await {
            Ok(Ok(dishes)) => HttpResponse::Ok().json(dishes),
            Ok(Err(_)) => HttpResponse::NotFound().json("No dishes found"),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to fetch dishes: {err}"))
            }
        }
    }

    #[derive(Deserialize)]
    struct CreateMenuBody {
        dishes: Vec<i64>,
        date: NaiveDate,
    }

    #[post("/create-new")]
    pub async fn create_menu(state: Data<AppState>, body: Bytes) -> impl Responder {
        let json_input = match String::from_utf8(Vec::from(body.as_ref())) {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::BadRequest()
                    .json("Failed to parse request. Non utf-8 characters")
            }
        };

        let body: CreateMenuBody = match serde_json::from_str(json_input.as_str()) {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::BadRequest()
                    .json("Failed to parse request. Body is not a desired structure")
            }
        };

        let mut unique_ids = HashSet::new();
        let mut dish_ids = body.dishes;

        dish_ids.retain(|id| unique_ids.insert(id.clone()));

        match state.pg_db.send(FetchSpecificDishes(dish_ids)).await {
            Ok(Ok(dishes)) => {
                match state
                    .redis_handler
                    .save_new_menu(state.pg_db.clone(), dishes, &body.date)
                    .await
                {
                    Ok(menu_key) => HttpResponse::Ok().json(menu_key),
                    Err(err) => HttpResponse::InternalServerError()
                        .json(format!("Unable to perform action: {err}")),
                }
            }
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Dishes were not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[put("/set-active/{date}")]
    pub async fn set_active_menu(state: Data<AppState>, path: Path<NaiveDate>) -> impl Responder {
        let date = path.into_inner();

        match state.redis_handler.set_active_menu(&date) {
            Ok(_) => HttpResponse::Ok().json(format!("Successfully set active menu to {date}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[delete("/{date}")]
    pub async fn delete_menu(state: Data<AppState>, path: Path<NaiveDate>) -> impl Responder {
        let date = path.into_inner();

        match state.redis_handler.delete_menu(&date) {
            Ok(_) => HttpResponse::Ok().json(format!("Successfully deleted menu for {date}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }
}

// sub-route "/order"
pub mod order_route {
    use crate::services::db_utils::AppState;
    use crate::services::messages::{
        AddDishToOrder, ConfirmOrder, CreateOrder, DecrementDishInOrder, DeleteDishFromOrder,
        FetchDish, GetOrder, PayForOrder,
    };
    use actix_web::web::{Data, Path};
    use actix_web::{delete, get, post, put, HttpResponse, Responder};
    use serde::de::IntoDeserializer;

    #[get("/{order_id}")]
    pub async fn get_ordered_dishes(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let order_id = path.into_inner();

        match state.pg_db.send(GetOrder(order_id)).await {
            Ok(Ok(dishes)) => HttpResponse::Ok().json(dishes),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order was not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[post("/create-for-table/{table_id}")]
    pub async fn create_blank_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match state.pg_db.send(CreateOrder(table_id)).await {
            Ok(Ok(id)) => HttpResponse::Ok().json(id),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Table was not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[post("/{order_id}/add/{dish_id}")]
    pub async fn add_dish_to_order(
        state: Data<AppState>,
        path: Path<(i64, i64)>,
    ) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state.pg_db.send(AddDishToOrder { order_id, dish_id }).await {
            Ok(Ok(id)) => HttpResponse::Ok().json(id),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order was not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[put("/{order_id}/decrement/{dish_id}")]
    pub async fn decrement_dishes_from_order(
        state: Data<AppState>,
        path: Path<(i64, i64)>,
    ) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state
            .pg_db
            .send(DecrementDishInOrder { order_id, dish_id })
            .await
        {
            Ok(Ok(id)) => HttpResponse::Ok().json(id),
            Ok(Err(err)) => {
                HttpResponse::NotFound().json(format!("Order or dish were not found: {err}"))
            }
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[delete("/{order_id}/{dish_id}")]
    pub async fn delete_dish_from_order(
        state: Data<AppState>,
        path: Path<(i64, i64)>,
    ) -> impl Responder {
        let (order_id, dish_id) = path.into_inner();

        match state
            .pg_db
            .send(DeleteDishFromOrder { order_id, dish_id })
            .await
        {
            Ok(Ok(id)) => HttpResponse::Ok().json(id),
            Ok(Err(err)) => {
                HttpResponse::NotFound().json(format!("Order or dish were not found: {err}"))
            }
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[post("/{order_id}/confirm")]
    pub async fn confirm_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let order_id = path.into_inner();

        match state.pg_db.send(ConfirmOrder(order_id)).await {
            Ok(Ok(_)) => HttpResponse::Ok().json(format!(
                "Order with id {order_id} is successfully confirmed"
            )),
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }

    #[post("/{order_id}/pay")]
    pub async fn pay_for_order(state: Data<AppState>, path: Path<i64>) -> impl Responder {
        let order_id = path.into_inner();

        match state.pg_db.send(PayForOrder(order_id)).await {
            Ok(Ok(_)) => {
                HttpResponse::Ok().json(format!("Order with id {order_id} is successfully paid"))
            }
            Ok(Err(err)) => HttpResponse::NotFound().json(format!("Order not found: {err}")),
            Err(err) => {
                HttpResponse::InternalServerError().json(format!("Unable to perform action: {err}"))
            }
        }
    }
}

// sub-route "/test"
pub mod test_route {
    use crate::services::db_utils::AppState;
    use crate::services::messages::{AddWaiter, FetchDishes};
    use actix_web::web::Data;
    use actix_web::{get, post, HttpResponse, Responder};
    use redis::Commands;
    use std::collections::HashSet;

    #[get("/healthcheck")]
    pub async fn healthcheck() -> impl Responder {
        HttpResponse::Ok().body("I'm alive!")
    }

    #[post("/create-mock-menu")]
    pub async fn create_mock_menu(state: Data<AppState>) -> impl Responder {
        let mut dishes = match state.pg_db.send(FetchDishes).await {
            Ok(Ok(resp)) => resp,
            _ => {
                return HttpResponse::InternalServerError()
                    .json("Unable to get dishes from sql database")
            }
        };

        let mut unique_dish_types = HashSet::new();
        dishes.retain(|dish| unique_dish_types.insert(dish.type_.clone()));

        match state
            .redis_handler
            .save_new_menu(
                state.pg_db.clone(),
                dishes,
                &chrono::Local::now().date_naive(),
            )
            .await
        {
            Ok(key) => HttpResponse::Ok().json(format!(
                "Menu is successfully composed and placed into redis db by the key '{key}'"
            )),
            Err(err) => HttpResponse::InternalServerError().json(err),
        }
    }
}
