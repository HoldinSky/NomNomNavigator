use crate::schema::dish_to_product::dsl::dish_to_product;
use crate::schema::dishes::approx_cook_time_s;
use actix::Handler;
use diesel::connection::SimpleConnection;
use diesel::expression::AsExpression;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    result::{DatabaseErrorKind, Error},
    EqAll, ExpressionMethods, Insertable, PgConnection, QueryDsl, QueryResult, RunQueryDsl,
};

use crate::services::db_models::{Dish, Waiter};
use crate::services::db_utils::PgActor;
use crate::services::insertable::DishProductMapping;
use crate::services::messages::{
    AddDishToOrder, AddWaiter, ConfirmOrder, CreateDish, CreateOrder, DecrementDishInOrder,
    DeleteDishFromOrder, FetchDish, FetchDishIngredients, FetchDishes, FetchSpecificDishes,
    FetchWaiters, PayForOrder,
};

use super::messages::GetOrder;

fn establish_connection(
    pool: &Pool<ConnectionManager<PgConnection>>,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, Error> {
    match pool.get() {
        Ok(val) => Ok(val),
        Err(_) => Err(connection_err()),
    }
}

fn connection_err() -> Error {
    Error::DatabaseError(
        DatabaseErrorKind::ClosedConnection,
        Box::new("Failed to establish connection".to_owned()),
    )
}

fn get_dish_price(mut conn: &mut PgConnection, dish_id: i64) -> Result<i32, Error> {
    use crate::schema::dishes::dsl::dishes;
    use crate::schema::dishes::price;

    match dishes.select(price).find(dish_id).first::<i32>(conn) {
        Ok(val) => Ok(val),
        Err(_) => Err(get_db_err("Failed to get dish's price")),
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
    type Result = QueryResult<()>;

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
            .returning((id, first_name, last_name, is_admin))
            .execute(&mut conn)?;

        Ok(())
    }
}

impl Handler<CreateDish> for PgActor {
    type Result = QueryResult<Dish>;

    fn handle(&mut self, msg: CreateDish, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dishes::{
            approx_cook_time_s, dsl::dishes, id, name, portion_weight_g, price, type_,
        };
        use crate::services::insertable::DishProductMapping;
        use crate::services::insertable::NewDish;

        let mut conn = establish_connection(&self.0)?;

        conn.build_transaction().run(move |trx_conn| {
            let new_dish = diesel::insert_into(dishes)
                .values(NewDish {
                    name: msg.dish_name,
                    type_: msg.dish_type.to_string(),
                    approx_cook_time_s: msg.approx_cook_time_s,
                    portion_weight_g: msg.portion_weight_g,
                    price: msg.price,
                })
                .returning((id, name, type_, portion_weight_g, price, approx_cook_time_s))
                .get_result::<Dish>(trx_conn)?;

            for ing in msg.ingredients {
                diesel::insert_into(dish_to_product)
                    .values(DishProductMapping {
                        dish_id: new_dish.id,
                        product_id: ing.id,
                        weight_g: ing.used_g,
                    })
                    .execute(trx_conn)?;
            }

            Ok(new_dish)
        })
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

impl Handler<FetchSpecificDishes> for PgActor {
    type Result = QueryResult<Vec<Dish>>;

    fn handle(&mut self, msg: FetchSpecificDishes, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dishes::{dsl::dishes, id};

        let mut conn = establish_connection(&self.0)?;

        dishes.filter(id.eq_any(msg.0)).get_results(&mut conn)
    }
}

impl Handler<FetchDishIngredients> for PgActor {
    type Result = QueryResult<Vec<(String, i32)>>;

    fn handle(&mut self, msg: FetchDishIngredients, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_product::{dish_id, dsl::dish_to_product, weight_g};
        use crate::schema::products::{dsl::products, name};

        let mut conn = establish_connection(&self.0)?;

        dish_to_product
            .inner_join(products)
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
        use crate::services::insertable::{NewOrder, OrderDish};

        let mut conn = establish_connection(&self.0)?;

        diesel::insert_into(orders)
            .values(NewOrder {
                table_id: msg.0,
                total_cost: 0,
            })
            .returning(id)
            .get_result::<i64>(&mut conn)
    }
}

impl Handler<GetOrder> for PgActor {
    type Result = QueryResult<Vec<(Dish, i32)>>;

    fn handle(&mut self, msg: GetOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{count, dish_id, dsl::dish_to_order, order_id};
        use crate::schema::dishes::{
            dsl::dishes, id as dish_pk, name, portion_weight_g, price, type_,
        };
        use crate::schema::orders::{dsl::orders, id as order_pk};

        let mut conn = establish_connection(&self.0)?;

        let query_res: Vec<Dish> = dish_to_order
            .filter(order_id.eq(msg.0))
            .inner_join(dishes)
            .select((
                dish_pk,
                name,
                type_,
                portion_weight_g,
                price,
                approx_cook_time_s,
            ))
            .filter(dish_pk.eq(dish_id))
            .get_results::<Dish>(&mut conn)?;

        conn.build_transaction().run(|trx_conn| {
            let mut to_return = vec![];
            for ele in query_res {
                let dish_count = dish_to_order
                    .filter(dish_id.eq(ele.id))
                    .filter(order_id.eq(msg.0))
                    .select(count)
                    .first::<i32>(trx_conn)?;

                to_return.push((ele, dish_count));
            }

            Ok(to_return)
        })
    }
}

impl Handler<AddDishToOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: AddDishToOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{count, dish_id, dsl::dish_to_order, id, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};
        use crate::services::insertable::OrderDish;

        let mut conn = establish_connection(&self.0)?;

        if orders
            .find(msg.order_id)
            .select(is_confirmed)
            .first::<bool>(&mut conn)?
        {
            return Err(get_db_err("The order is already confirmed"));
        };

        if let Ok((mapping_id, dish_count)) = dish_to_order
            .select((id, count))
            .filter(order_id.eq(msg.order_id))
            .filter(dish_id.eq(msg.dish_id))
            .first::<(i64, i32)>(&mut conn)
        {
            diesel::update(dish_to_order.find(mapping_id))
                .set(count.eq(dish_count + 1))
                .execute(&mut conn);

            return Ok(msg.order_id);
        };

        conn.build_transaction().run(|trx_conn| {
            let dish_price = get_dish_price(trx_conn, msg.dish_id)?;

            diesel::insert_into(dish_to_order)
                .values(OrderDish {
                    dish_id: msg.dish_id,
                    order_id: msg.order_id,
                    count: 1,
                    dish_price,
                })
                .execute(trx_conn)?;

            Ok(msg.order_id)
        })
    }
}

impl Handler<DecrementDishInOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DecrementDishInOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{count, dish_id, dsl::dish_to_order, id, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};

        let mut conn = establish_connection(&self.0)?;

        match orders
            .find(msg.order_id)
            .select(is_confirmed)
            .first::<bool>(&mut conn)
        {
            Ok(val) => {
                if val {
                    return Err(get_db_err("The order is already confirmed"));
                }
            }
            Err(err) => return Err(err),
        };

        conn.build_transaction().run(|trx_conn| {
            let (mapping_id, dish_count) = dish_to_order
                .select((id, count))
                .filter(order_id.eq(msg.order_id))
                .filter(dish_id.eq(msg.dish_id))
                .first::<(i64, i32)>(trx_conn)?;

            if dish_count == 1 {
                diesel::delete(dish_to_order.find(mapping_id)).execute(trx_conn);
            } else {
                diesel::update(dish_to_order.find(mapping_id))
                    .set(count.eq(dish_count - 1))
                    .execute(trx_conn);
            }

            Ok(msg.order_id)
        })
    }
}

impl Handler<DeleteDishFromOrder> for PgActor {
    type Result = QueryResult<i64>;

    fn handle(&mut self, msg: DeleteDishFromOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{dish_id, dsl::dish_to_order, order_id};
        use crate::schema::orders::{dsl::orders, is_confirmed};
        use crate::services::db_models::Order;

        let mut conn = establish_connection(&self.0)?;

        conn.build_transaction().run(|trx_conn| {
            match orders
                .find(msg.order_id)
                .select(is_confirmed)
                .first::<bool>(trx_conn)
            {
                Ok(val) => {
                    if val {
                        return Err(get_db_err("The order is already confirmed"));
                    }
                }
                Err(err) => return Err(err),
            };

            diesel::delete(
                dish_to_order
                    .filter(dish_id.eq(msg.dish_id))
                    .filter(order_id.eq(msg.order_id)),
            )
            .execute(trx_conn)?;

            Ok(msg.order_id)
        })
    }
}

impl Handler<ConfirmOrder> for PgActor {
    type Result = QueryResult<()>;

    fn handle(&mut self, msg: ConfirmOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::dish_to_order::{
            count, dish_id as dto_dish_id, dsl::dish_to_order, order_id,
        };
        use crate::schema::dish_to_product::{
            dish_id as dtp_dish_id, dsl::dish_to_product, product_id, weight_g,
        };
        use crate::schema::orders::{dsl::orders, id as ord_pk, is_confirmed};
        use crate::schema::products::{dsl::products, id as prod_pk, in_stock_g};
        use std::collections::HashMap;

        let mut conn = establish_connection(&self.0)?;

        match orders
            .find(msg.0)
            .select(is_confirmed)
            .first::<bool>(&mut conn)
        {
            Ok(val) => {
                if val {
                    return Err(get_db_err("The order is already confirmed"));
                }
            }
            Err(err) => return Err(err),
        };

        let ordered_dishes = orders
            .find(msg.0)
            .inner_join(dish_to_order)
            .filter(ord_pk.eq(order_id))
            .select((dto_dish_id, count))
            .get_results::<(i64, i32)>(&mut conn)?;

        let mut dishes_to_count = HashMap::new();
        let mut products_to_weight: HashMap<i64, i32> = HashMap::new();

        for (id, dish_count) in ordered_dishes {
            dishes_to_count.insert(id, dish_count);
        }

        conn.build_transaction().run(|trx_conn| {
            let dish_to_products_usage = dish_to_product
                .filter(dtp_dish_id.eq_any(dishes_to_count.keys()))
                .select((dtp_dish_id, product_id, weight_g))
                .get_results::<(i64, i64, i32)>(trx_conn)?;

            for (dish, product, weight) in dish_to_products_usage {
                let already_used = products_to_weight.entry(product.clone()).or_insert(0);
                *already_used += weight * dishes_to_count.get(&dish).unwrap();
            }

            for (p_id, weight_used) in products_to_weight {
                diesel::update(products.find(p_id))
                    .set(in_stock_g.eq(in_stock_g - weight_used))
                    .execute(trx_conn)?;
            }

            diesel::update(orders.find(msg.0))
                .set(is_confirmed.eq(true))
                .execute(trx_conn)?;

            Ok(())
        })
    }
}

impl Handler<PayForOrder> for PgActor {
    type Result = QueryResult<()>;

    fn handle(&mut self, msg: PayForOrder, _ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::orders::{dsl::orders, is_confirmed, is_paid, total_cost};
        use crate::schema::stats::{day, dsl::stats, income};
        use crate::services::db_models::Order;
        use crate::services::insertable::NewStats;
        use chrono::NaiveDate;

        let mut conn = establish_connection(&self.0)?;

        match orders
            .find(msg.0)
            .select(is_confirmed)
            .first::<bool>(&mut conn)
        {
            Ok(val) => {
                if !val {
                    return Err(get_db_err("The order is not confirmed yet"));
                }
            }
            Err(err) => return Err(err),
        };

        conn.build_transaction().run(|trx_conn| {
            let order_cost = match orders
                .find(msg.0)
                .select((is_paid, total_cost))
                .first::<(bool, i32)>(trx_conn)
            {
                Ok(val) => {
                    if val.0 {
                        return Err(get_db_err("The order is already paid"));
                    } else {
                        val.1
                    }
                }
                Err(err) => return Err(err),
            };

            diesel::update(orders.find(msg.0))
                .set(is_paid.eq(true))
                .execute(trx_conn)?;

            let today = chrono::Local::now().date_naive();

            let is_first_record = stats
                .select(day)
                .filter(day.eq(today))
                .first::<NaiveDate>(trx_conn)
                .err()
                != None;

            if is_first_record {
                diesel::insert_into(stats)
                    .values(NewStats {
                        day: today,
                        income: order_cost,
                    })
                    .execute(trx_conn)
            } else {
                diesel::update(stats.filter(day.eq(today)))
                    .set(income.eq(income + order_cost))
                    .execute(trx_conn)
            }?;

            Ok(())
        })
    }
}
