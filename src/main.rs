#![allow(unused)]

use std::env;

use actix::{Addr, SyncArbiter};
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{http, web, App, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use futures::StreamExt;

use crate::services::redis_handling::RedisHandler;
use services::db_utils::{get_db_pool, AppState, PgActor};

mod schema;
mod services;
mod types;

fn init_pg_db() -> Addr<PgActor> {
    let db_url = env::var("PG_DATABASE_URL").expect("PG_DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_db_pool(&db_url).unwrap();

    SyncArbiter::start(5, move || PgActor(pool.clone()))
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

    let addr = env::var("ADDRESS").unwrap_or("127.0.0.1:8080".to_owned());
    let frontend_origin = env::var("FRONT_ORIGIN").unwrap_or("http://localhost:5173".to_owned());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(Data::new(AppState {
                pg_db: pg_db.clone(),
                redis_handler: RedisHandler::new(redis_db.clone()),
            }))
            .service(services::home_page)
            .service(
                web::scope("/waiters")
                    .service(services::waiters_route::fetch_waiters)
                    .service(services::waiters_route::add_waiter),
            )
            .service(
                web::scope("/menu")
                    .service(services::menu_route::create_menu)
                    .service(services::menu_route::set_active_menu)
                    .service(services::menu_route::view_menu)
                    .service(services::menu_route::delete_menu)
                    .service(services::menu_route::get_dish)
                    .service(services::menu_route::get_dishes),
            )
            .service(
                web::scope("/order")
                    .service(services::order_route::create_blank_order)
                    .service(services::order_route::add_dish_to_order)
                    .service(services::order_route::decrement_dishes_from_order)
                    .service(services::order_route::delete_dish_from_order)
                    .service(services::order_route::confirm_order)
                    .service(services::order_route::pay_for_order),
            )
            .service(web::scope("/dishes").service(services::dishes_route::create_dish))
            .service(
                web::scope("/test")
                    .service(services::test_route::healthcheck)
                    .service(services::test_route::create_mock_menu),
            )
            .wrap(cors)
    })
    .bind(addr)?
    .run()
    .await
}
