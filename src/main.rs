use std::env;

use actix::{Addr, SyncArbiter};
use actix_web::{App, get, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;

use crate::db_utils::{AppState, DbActor, get_db_pool};

mod db_models;
mod db_utils;
mod types;
mod messages;
mod schema;

fn init() -> Addr<DbActor> {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_db_pool(&db_url).unwrap();

    SyncArbiter::start(5, move || DbActor(pool.clone()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_addr = init();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: db_addr.clone() }))
            .service(index)
            .service(healthcheck)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Rust service prototype")
}

#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().body("I'm alive!")
}