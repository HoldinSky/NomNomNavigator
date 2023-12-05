use actix::{Actor, Addr, SyncContext};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::types::PoolInitializationError;

pub struct PgActor(pub Pool<ConnectionManager<PgConnection>>);

pub struct AppState {
    pub pg_db: Addr<PgActor>,
    pub redis_db: redis::Client
}

impl Actor for PgActor {
    type Context = SyncContext<Self>;
}

pub fn get_db_pool(db_url: &str) -> Result<Pool<ConnectionManager<PgConnection>>, PoolInitializationError> {
    let manager: ConnectionManager<PgConnection> = ConnectionManager::<PgConnection>::new(db_url);
    match Pool::builder().build(manager) {
        Ok(val) => Ok(val),
        Err(err) => Err(PoolInitializationError(err.to_string()))
    }
}