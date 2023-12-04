// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dish_type"))]
    pub struct DishType;
}

diesel::table! {
    client_order (id) {
        id -> Int8,
        table_id -> Nullable<Int8>,
        total_cost -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DishType;

    dish (id) {
        id -> Int8,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> DishType,
        portion_weight -> Int4,
        cost -> Int4,
        approx_cook_time -> Timestamp,
    }
}

diesel::table! {
    dish_to_order (id) {
        id -> Int8,
        dish_id -> Nullable<Int8>,
        order_id -> Nullable<Int8>,
    }
}

diesel::table! {
    restaurant_table (id) {
        id -> Int8,
        seat_count -> Int4,
        is_occupied -> Bool,
        reserved_at -> Nullable<Timestamp>,
        #[max_length = 50]
        reserved_by -> Nullable<Varchar>,
        waiter_id -> Nullable<Int8>,
    }
}

diesel::table! {
    waiter (id) {
        id -> Int8,
        #[max_length = 40]
        first_name -> Varchar,
        #[max_length = 40]
        last_name -> Varchar,
        is_admin -> Bool,
    }
}

diesel::joinable!(client_order -> restaurant_table (table_id));
diesel::joinable!(dish_to_order -> client_order (order_id));
diesel::joinable!(dish_to_order -> dish (dish_id));
diesel::joinable!(restaurant_table -> waiter (waiter_id));

diesel::allow_tables_to_appear_in_same_query!(
    client_order,
    dish,
    dish_to_order,
    restaurant_table,
    waiter,
);
