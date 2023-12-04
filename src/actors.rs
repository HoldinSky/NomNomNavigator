use actix::Handler;
use diesel::{PgConnection, QueryResult, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::schema::waiter::dsl::waiter;
use crate::services::db_models::Waiter;
use crate::services::db_utils::DbActor;
use crate::services::messages::FetchWaiters;

impl Handler<FetchWaiters> for DbActor {
    type Result = QueryResult<Vec<Waiter>>;

    fn handle(&mut self, _msg: FetchWaiters, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> = self.0.get().expect("Fetch waiter:: Unable to establish connection");

        waiter.get_results::<Waiter>(&mut conn)
    }
}