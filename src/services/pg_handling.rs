use actix::Handler;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    result::{DatabaseErrorKind, Error},
    ExpressionMethods, Insertable, PgConnection, QueryDsl, QueryResult, RunQueryDsl,
};

use crate::services::db_models::{Dish, DishToOrder, Waiter};
use crate::services::db_utils::PgActor;
use crate::services::insertable::{NewOrder, OrderDish};
use crate::services::messages::{AddDishToOrder, AddWaiter, DecrementDishInOrder, DeleteDishFromOrder, FetchDish, FetchDishes, FetchDishIngredients, FetchWaiters, FirstDish};

fn dish_price_err() -> Result<(), Error> { Err(Error::DatabaseError(DatabaseErrorKind::UnableToSendCommand, Box::new("Failed to get dish's price".to_owned()))) }

fn establish_connection(pool: &Pool<ConnectionManager<PgConnection>>) -> PooledConnection<ConnectionManager<PgConnection>> {
    pool.get().expect("Unable to establish connection with postgres db")
}

fn get_dish_price(mut conn: &mut PooledConnection<ConnectionManager<PgConnection>>, dish_id: i64) -> Result<i32, Error> {
    use crate::schema::dishes::dsl::dishes;
    use crate::schema::dishes::price;

    match dishes.select(price).find(dish_id).first::<i32>(conn) {
        Ok(val) => Ok(val),
        Err(_) => Err(dish_price_err().err().unwrap())
    }
}

impl Handler<FetchWaiters> for PgActor {
    type Result = QueryResult<Vec<Waiter>>;

    fn handle(&mut self, _msg: FetchWaiters, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::waiters::dsl::waiters;

        let mut conn = establish_connection(&self.0);

        waiters.get_results::<Waiter>(&mut conn)
    }
}

impl Handler<AddWaiter> for PgActor {
    type Result = QueryResult<Waiter>;

    fn handle(&mut self, msg: AddWaiter, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::waiters::dsl::waiters;
        use crate::schema::waiters::{first_name, id, is_admin, last_name};
        use crate::services::insertable::NewWaiter;

        let mut conn = establish_connection(&self.0);

        let new_waiter = NewWaiter {
            first_name: msg.first_name,
            last_name: msg.last_name,
            is_admin: msg.is_admin,
        };

        diesel::insert_into(waiters)
            .values(new_waiter)
            .returning((
                id,
                first_name,
                last_name,
                is_admin
            )).get_result::<Waiter>(&mut conn)
    }
}

impl Handler<FetchDish> for PgActor {
    type Result = QueryResult<Dish>;

    fn handle(&mut self, msg: FetchDish, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dishes::dsl::dishes;
        let mut conn = establish_connection(&self.0);

        dishes.find(msg.0).first(&mut conn)
    }
}

impl Handler<FetchDishes> for PgActor {
    type Result = QueryResult<Vec<Dish>>;

    fn handle(&mut self, msg: FetchDishes, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dishes::dsl::dishes;
        let mut conn = establish_connection(&self.0);

        dishes.get_results::<Dish>(&mut conn)
    }
}

impl Handler<FetchDishIngredients> for PgActor {
    type Result = QueryResult<Vec<(String, i32)>>;

    fn handle(&mut self, msg: FetchDishIngredients, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_product::{dish_id, dsl::dish_to_product, weight_g};
        use crate::schema::products::{dsl::products, name};

        let mut conn = establish_connection(&self.0);

        dish_to_product.inner_join(products)
            .select((name, weight_g))
            .filter(dish_id.eq(msg.0))
            .get_results(&mut conn)
    }
}

impl Handler<FirstDish> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: FirstDish, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::dsl::dish_to_order;
        use crate::schema::dishes::{dsl::dishes, price};
        use crate::schema::orders::{dsl::orders, id};

        let mut conn = establish_connection(&self.0);

        let order_id = match diesel::insert_into(orders)
            .values(NewOrder { table_id: msg.table_id, total_cost: 0 })
            .returning(id)
            .get_result::<i64>(&mut conn) {
            Ok(val) => val,
            _ => return Err(Error::DatabaseError(DatabaseErrorKind::UnableToSendCommand, Box::new("Failed to create new order record".to_owned())))
        };

        let dish_price = match get_dish_price(&mut conn, msg.dish_id) {
            Ok(val) => val,
            Err(err) => return Err(err)
        };

        diesel::insert_into(dish_to_order)
            .values(OrderDish {
                dish_id: msg.dish_id,
                count: 1,
                order_id,
                dish_price,
            }).execute(&mut conn);

        Ok(order_id)
    }
}

impl Handler<AddDishToOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: AddDishToOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dsl::dish_to_order, dish_id, order_id, count, id};

        let mut conn = establish_connection(&self.0);

        if let Ok((mapping_id, dish_count)) = dish_to_order
            .select((id, count))
            .filter(order_id.eq(msg.order_id))
            .filter(dish_id.eq(msg.dish_id)).first::<(i64, i32)>(&mut conn) {
            diesel::update(dish_to_order.find(mapping_id)).set(count.eq(dish_count + 1)).execute(&mut conn);

            return Ok(msg.order_id);
        };

        let dish_price = match get_dish_price(&mut conn, msg.dish_id) {
            Ok(val) => val,
            Err(err) => return Err(err)
        };

        match diesel::insert_into(dish_to_order)
            .values(OrderDish {
                dish_id: msg.dish_id,
                order_id: msg.order_id,
                count: 1,
                dish_price,
            }).execute(&mut conn) {
            Ok(_) => Ok(msg.order_id),
            Err(err) => Err(err)
        }
    }
}

impl Handler<DecrementDishInOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DecrementDishInOrder, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dsl::dish_to_order, dish_id, order_id, count, id};

        let mut conn = establish_connection(&self.0);

        match dish_to_order
            .select((id, count))
            .filter(order_id.eq(msg.order_id))
            .filter(dish_id.eq(msg.dish_id)).first::<(i64, i32)>(&mut conn) {
            Ok((mapping_id, dish_count)) => {
                if dish_count == 1 {
                    diesel::delete(dish_to_order.find(mapping_id)).execute(&mut conn);
                } else {
                    diesel::update(dish_to_order.find(mapping_id)).set(count.eq(dish_count - 1)).execute(&mut conn);
                }

                Ok(msg.order_id)
            }
            Err(err) => Err(err)
        }
    }
}

impl Handler<DeleteDishFromOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DeleteDishFromOrder, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dsl::dish_to_order, dish_id, order_id};

        let mut conn = establish_connection(&self.0);

        match diesel::delete(dish_to_order.filter(dish_id.eq(msg.dish_id)).filter(order_id.eq(msg.order_id)))
            .execute(&mut conn) {
            Ok(_) => Ok(msg.order_id),
            Err(err) => Err(err)
        }
    }
}