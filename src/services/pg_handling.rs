use actix::Handler;
use diesel::{PgConnection, QueryResult, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use crate::schema::dishes::dsl::dishes;

use crate::schema::waiters::{first_name, id as w_id, is_admin, last_name};
use crate::schema::waiters::dsl::waiters;
use crate::services::db_models::{Dish, Waiter};
use crate::services::db_utils::PgActor;
use crate::services::insertable::NewWaiter;
use crate::services::messages::{AddWaiter, FetchWaiters, GetAllDishes};

impl Handler<FetchWaiters> for PgActor {
    type Result = QueryResult<Vec<Waiter>>;

    fn handle(&mut self, _msg: FetchWaiters, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> = self.0.get().expect("Fetch waiter:: Unable to establish connection");

        waiters.get_results::<Waiter>(&mut conn)
    }
}

impl Handler<AddWaiter> for PgActor {
    type Result = QueryResult<Waiter>;

    fn handle(&mut self, msg: AddWaiter, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> = self.0.get().expect("Add waiter:: Unable to establish connection");

        let new_waiter = NewWaiter {
            first_name: msg.first_name,
            last_name: msg.last_name,
            is_admin: msg.is_admin
        };

        diesel::insert_into(waiters)
            .values(new_waiter)
            .returning((
                w_id,
                first_name,
                last_name,
                is_admin
            )).get_result::<Waiter>(&mut conn)
    }
}

impl Handler<GetAllDishes> for PgActor {
    type Result = QueryResult<Vec<Dish>>;

    fn handle(&mut self, msg: GetAllDishes, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Get all dishes:: Unable to establish connection");

        dishes.get_results::<Dish>(&mut conn)
    }
}