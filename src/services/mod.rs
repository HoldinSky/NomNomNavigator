use actix_web::{get, HttpResponse, Responder};

pub mod db_models;
pub mod db_utils;
pub mod messages;

#[get("/")]
pub async fn home_page() -> impl Responder {
    HttpResponse::Ok().body("Rust service prototype")
}

// under scope "/waiters"
pub mod scope_waiters {
    use actix::Addr;
    use actix_web::{get, HttpResponse, Responder};
    use actix_web::web::Data;

    use crate::services::db_utils::{AppState, DbActor};
    use crate::services::messages::FetchWaiters;

    #[get("/all")]
    pub async fn fetch_waiters(state: Data<AppState>) -> impl Responder {
        let db: Addr<DbActor> = state.as_ref().db.clone();

        match db.send(FetchWaiters).await {
            Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
            Ok(Err(_)) => HttpResponse::NotFound().json("Waiters not found"),
            _ => HttpResponse::InternalServerError().json("Unable to retrieve waiters"),
        }
    }
}

// under scope "/test"
pub mod scope_test {
    use actix_web::{get, HttpResponse, Responder};

    #[get("/healthcheck")]
    pub async fn healthcheck() -> impl Responder {
        HttpResponse::Ok().body("I'm alive!")
    }
}

