use std::env;

use actix::{Addr, SyncArbiter};
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;

use services::db_utils::{AppState, DbActor, get_db_pool};

mod schema;
mod actors;
mod types;
mod services;

fn init_db() -> Addr<DbActor> {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_db_pool(&db_url).unwrap();

    SyncArbiter::start(5, move || DbActor(pool.clone()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_addr = init_db();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: db_addr.clone() }))
            .service(services::home_page)
            .service(
                web::scope("/waiters")
                    .service(services::scope_waiters::fetch_waiters)
            )
            .service(
                web::scope("/test")
                    .service(services::scope_test::healthcheck)
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}