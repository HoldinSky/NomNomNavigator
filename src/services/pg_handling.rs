use actix::Handler;
use diesel::{
    Insertable,
    PgConnection,
    ExpressionMethods,
    QueryDsl, RunQueryDsl, QueryResult,
    result::{DatabaseErrorKind, Error},
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use diesel::connection::SimpleConnection;

use crate::services::db_models::{Waiter, Dish};
use crate::services::db_utils::PgActor;
use crate::services::messages::{
    AddWaiter,
    FetchWaiters,
    FetchDish,
    FetchDishes,
    FetchDishIngredients,
    AddDishToOrder,
    DecrementDishInOrder,
    DeleteDishFromOrder,
    ConfirmOrder,
    PayForOrder,
    CreateOrder,
};

fn establish_connection(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<PooledConnection<ConnectionManager<PgConnection>>, Error> {
    match pool.get() {
        Ok(val) => Ok(val),
        Err(_) => Err(connection_err())
    }
}

fn connection_err() -> Error { Error::DatabaseError(DatabaseErrorKind::ClosedConnection, Box::new("Failed to establish connection".to_owned())) }

fn get_dish_price(mut conn: &mut PooledConnection<ConnectionManager<PgConnection>>, dish_id: i64) -> Result<i32, Error> {
    use crate::schema::dishes::dsl::dishes;
    use crate::schema::dishes::price;

    match dishes.select(price).find(dish_id).first::<i32>(conn) {
        Ok(val) => Ok(val),
        Err(_) => Err(get_db_err("Failed to get dish's price"))
    }
}

fn get_db_err(msg: &str) -> Error {
    Error::DatabaseError(
        DatabaseErrorKind::UnableToSendCommand,
        Box::new(msg.to_owned()),
    )
}


impl Handler<FetchWaiters> for PgActor {
    type Result = QueryResult<Vec<Waiter>>;

    fn handle(&mut self, _msg: FetchWaiters, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::waiters::dsl::waiters;

        let mut conn = establish_connection(&self.0)?;

        waiters.get_results::<Waiter>(&mut conn)
    }
}

impl Handler<AddWaiter> for PgActor {
    type Result = QueryResult<Waiter>;

    fn handle(&mut self, msg: AddWaiter, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::waiters::dsl::waiters;
        use crate::schema::waiters::{first_name, id, is_admin, last_name};
        use crate::services::insertable::NewWaiter;

        let mut conn = establish_connection(&self.0)?;

        diesel::insert_into(waiters)
            .values(NewWaiter {
                first_name: msg.first_name,
                last_name: msg.last_name,
                is_admin: msg.is_admin,
            })
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

        let mut conn = establish_connection(&self.0)?;

        dishes.find(msg.0).first(&mut conn)
    }
}

impl Handler<FetchDishes> for PgActor {
    type Result = QueryResult<Vec<Dish>>;

    fn handle(&mut self, msg: FetchDishes, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dishes::dsl::dishes;

        let mut conn = establish_connection(&self.0)?;

        dishes.get_results::<Dish>(&mut conn)
    }
}

impl Handler<FetchDishIngredients> for PgActor {
    type Result = QueryResult<Vec<(String, i32)>>;

    fn handle(&mut self, msg: FetchDishIngredients, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_product::{dish_id, dsl::dish_to_product, weight_g};
        use crate::schema::products::{dsl::products, name};

        let mut conn = establish_connection(&self.0)?;

        dish_to_product.inner_join(products)
            .select((name, weight_g))
            .filter(dish_id.eq(msg.0))
            .get_results(&mut conn)
    }
}

impl Handler<CreateOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: CreateOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::dsl::dish_to_order;
        use crate::schema::orders::{dsl::orders, id};
        use crate::services::insertable::{OrderDish, NewOrder};

        let mut conn = establish_connection(&self.0)?;

        diesel::insert_into(orders)
            .values(NewOrder { table_id: msg.0, total_cost: 0 })
            .returning(id)
            .get_result::<i64>(&mut conn)
    }
}

impl Handler<AddDishToOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: AddDishToOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{count, dish_id, dsl::dish_to_order, id, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};
        use crate::services::insertable::OrderDish;

        let mut conn = establish_connection(&self.0)?;

        if orders.find(msg.order_id).select(is_confirmed).first::<bool>(&mut conn)? {
            return Err(get_db_err("The order is already confirmed"));
        };

        if let Ok((mapping_id, dish_count)) = dish_to_order
            .select((id, count))
            .filter(order_id.eq(msg.order_id))
            .filter(dish_id.eq(msg.dish_id)).first::<(i64, i32)>(&mut conn) {
            diesel::update(dish_to_order.find(mapping_id)).set(count.eq(dish_count + 1)).execute(&mut conn);

            return Ok(msg.order_id);
        };

        let dish_price = get_dish_price(&mut conn, msg.dish_id)?;

        diesel::insert_into(dish_to_order)
            .values(OrderDish {
                dish_id: msg.dish_id,
                order_id: msg.order_id,
                count: 1,
                dish_price,
            }).execute(&mut conn)?;

        Ok(msg.order_id)
    }
}

impl Handler<DecrementDishInOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DecrementDishInOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dsl::dish_to_order, id, count, dish_id, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};

        let mut conn = establish_connection(&self.0)?;

        match orders.find(msg.order_id).select(is_confirmed).first::<bool>(&mut conn) {
            Ok(val) => if val { return Err(get_db_err("The order is already confirmed")); },
            Err(err) => return Err(err)
        };

        let (mapping_id, dish_count) = dish_to_order.select((id, count))
            .filter(order_id.eq(msg.order_id))
            .filter(dish_id.eq(msg.dish_id)).first::<(i64, i32)>(&mut conn)?;

        if dish_count == 1 {
            diesel::delete(dish_to_order.find(mapping_id)).execute(&mut conn);
        } else {
            diesel::update(dish_to_order.find(mapping_id)).set(count.eq(dish_count - 1)).execute(&mut conn);
        }

        Ok(msg.order_id)
    }
}

impl Handler<DeleteDishFromOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DeleteDishFromOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dsl::dish_to_order, dish_id, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};
        use crate::services::db_models::Order;

        let mut conn = establish_connection(&self.0)?;

        match orders.find(msg.order_id).select(is_confirmed).first::<bool>(&mut conn) {
            Ok(val) => if val { return Err(get_db_err("The order is already confirmed")); },
            Err(err) => return Err(err)
        };

        diesel::delete(dish_to_order.filter(dish_id.eq(msg.dish_id)).filter(order_id.eq(msg.order_id))).execute(&mut conn)?;

        Ok(msg.order_id)
    }
}

impl Handler<ConfirmOrder> for PgActor {
    type Result = QueryResult<()>;

    fn handle(&mut self, msg: ConfirmOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::orders::{dsl::orders, id as ord_pk, is_confirmed};
        use crate::schema::dish_to_order::{dsl::dish_to_order, dish_id as dto_dish_id, order_id, count};
        use crate::schema::dish_to_product::{dsl::dish_to_product, dish_id as dtp_dish_id, product_id, weight_g};
        use crate::schema::products::{dsl::products, id as prod_pk, in_stock_g};
        use std::collections::HashMap;

        let mut conn = establish_connection(&self.0)?;

        match orders.find(msg.0).select(is_confirmed).first::<bool>(&mut conn) {
            Ok(val) => if val { return Err(get_db_err("The order is already confirmed")); },
            Err(err) => return Err(err)
        };

        let ordered_dishes = orders.find(msg.0)
            .inner_join(dish_to_order).filter(ord_pk.eq(order_id))
            .select((dto_dish_id, count)).get_results::<(i64, i32)>(&mut conn)?;

        let mut products_to_weight: HashMap<i64, i32> = HashMap::new();
        for (id, dish_count) in ordered_dishes {
            dish_to_product.filter(dtp_dish_id.eq(id))
                .select((product_id, weight_g)).get_results::<(i64, i32)>(&mut conn)?
                .iter().for_each(|(p_id, weight)| {
                let already_used = products_to_weight.entry(p_id.clone()).or_insert(0);
                *already_used += weight * dish_count;
            });
        }

        for (p_id, weight_used) in products_to_weight {
            diesel::update(products.find(p_id)).set(in_stock_g.eq(in_stock_g - weight_used)).execute(&mut conn);
        }

        diesel::update(orders.find(msg.0)).set(is_confirmed.eq(true)).execute(&mut conn)?;

        Ok(())
    }
}

impl Handler<PayForOrder> for PgActor {
    type Result = QueryResult<()>;

    fn handle(&mut self, msg: PayForOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::orders::{dsl::orders, is_confirmed, is_paid, total_cost};
        use crate::schema::stats::{dsl::stats, day, income};
        use crate::services::db_models::Order;
        use crate::services::insertable::NewStats;
        use chrono::NaiveDate;

        let mut conn = establish_connection(&self.0)?;

        match orders.find(msg.0).select(is_confirmed).first::<bool>(&mut conn) {
            Ok(val) => if !val { return Err(get_db_err("The order is not confirmed yet")); },
            Err(err) => return Err(err)
        };

        let order_cost = match orders.find(msg.0).select((is_paid, total_cost)).first::<(bool, i32)>(&mut conn) {
            Ok(val) => if val.0 { return Err(get_db_err("The order is already paid")); } else { val.1 },
            Err(err) => return Err(err)
        };

        diesel::update(orders.find(msg.0)).set(is_paid.eq(true)).execute(&mut conn)?;

        let today = chrono::Local::now().date_naive();

        let is_first_record = stats.select(day).filter(day.eq(today)).first::<NaiveDate>(&mut conn).err() != None;

        if is_first_record {
            diesel::insert_into(stats).values(
                NewStats {
                    day: today,
                    income: order_cost,
                }).execute(&mut conn)
        } else {
            diesel::update(stats.filter(day.eq(today))).set(income.eq(income + order_cost)).execute(&mut conn)
        }?;

        Ok(())
    }
}