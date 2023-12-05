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

// under scope "/waiters"
pub mod scope_waiters {
    use actix::Addr;
    use actix_web::{get, HttpResponse, post, Responder};
    use actix_web::web::{Data, Json};
    use serde::Deserialize;

    use crate::services::db_utils::{AppState, PgActor};
    use crate::services::messages::{AddWaiter, FetchWaiters};

    #[get("/all")]
    pub async fn fetch_waiters(state: Data<AppState>) -> impl Responder {
        let db: Addr<PgActor> = state.as_ref().pg_db.clone();

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
        let db = state.as_ref().pg_db.clone();

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

// under scope "/users"
pub mod scope_users {
    use actix_web::{get, Responder};
    use actix_web::web::Data;
    use crate::services::db_utils::AppState;

    // #[get("/menu")]
    // pub async fn view_menu(state: Data<AppState>) -> impl Responder {
    //
    // }
}

// under scope "/test"
pub mod scope_test {
    use actix_web::{get, HttpResponse, post, Responder};
    use actix_web::web::Data;
    use redis::Commands;
    use crate::services::db_utils::AppState;
    use crate::services::messages::{AddWaiter, GetAllDishes};
    use crate::services::redis_handling::put_menu_to_db;

    #[get("/healthcheck")]
    pub async fn healthcheck() -> impl Responder {
        HttpResponse::Ok().body("I'm alive!")
    }

    #[post("/create-mock-menu")]
    pub async fn create_mock_menu(state: Data<AppState>) -> impl Responder {
        let state_ref = state.as_ref();
        let pg_db = state_ref.pg_db.clone();
        let redis_db = state_ref.redis_db.clone();

        let mut dishes = match pg_db.send(GetAllDishes).await {
            Ok(Ok(resp)) => resp,
            _ => return HttpResponse::InternalServerError().json("Unable to get dishes from database")
        };

        match put_menu_to_db(&redis_db, &mut dishes) {
            Ok(_) => HttpResponse::Ok().json("Menu is successfully composed and placed into redis db by the key 'menu'"),
            Err(err) => HttpResponse::InternalServerError().json(err),
        }
    }
}
