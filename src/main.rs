#![allow(unused)]

use std::env;

use actix::{Addr, SyncArbiter};
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use futures::StreamExt;

use services::db_utils::{AppState, get_db_pool, PgActor};

mod schema;
mod types;
mod services;
mod test;

fn init_pg_db() -> Addr<PgActor> {
    let db_url = env::var("PG_DATABASE_URL").expect("PG_DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_db_pool(&db_url).unwrap();

    SyncArbiter::start(5, move || PgActor(pool.clone()))
}

async fn init_mongo_db() -> mongodb::Client {
    let db_uri = env::var("MONGO_DATABASE_URI").expect("MONGO_DATABASE_URI must be set");

    mongodb::Client::with_uri_str(db_uri.clone()).await
        .expect(&format!("Couldn't establish connection with MongoDb by url: {}", db_uri))
}

fn init_redis_db() -> redis::Client {
    let db_uri = env::var("REDIS_DATABASE_URI").expect("REDIS_DATABASE_URI must be set");

    redis::Client::open(db_uri).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pg_db = init_pg_db();
    let redis_db = init_redis_db();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { pg_db: pg_db.clone(), redis_db: redis_db.clone() }))
            .service(services::home_page)
            .service(
                web::scope("/waiters")
                    .service(services::scope_waiters::fetch_waiters)
                    .service(services::scope_waiters::add_waiter)
            )
            .service(
                web::scope("/test")
                    .service(services::scope_test::healthcheck)
                    .service(services::scope_test::create_mock_menu)
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}