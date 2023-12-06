// @generated automatically by Diesel CLI.

diesel::table! {
    dish_to_order (id) {
        id -> Int8,
        dish_id -> Int8,
        order_id -> Int8,
        dish_price -> Int4,
        count -> Int4,
    }
}

diesel::table! {
    dish_to_product (id) {
        id -> Int8,
        dish_id -> Int8,
        product_id -> Int8,
        weight_g -> Int4,
    }
}

diesel::table! {
    dishes (id) {
        id -> Int8,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Text,
        portion_weight_g -> Int4,
        price -> Int4,
        approx_cook_time_s -> Int4,
    }
}

diesel::table! {
    orders (id) {
        id -> Int8,
        table_id -> Int8,
        total_cost -> Int4,
    }
}

diesel::table! {
    products (id) {
        id -> Int8,
        #[max_length = 50]
        name -> Varchar,
        in_stock_g -> Int4,
    }
}

diesel::table! {
    tables (id) {
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
    waiters (id) {
        id -> Int8,
        #[max_length = 40]
        first_name -> Varchar,
        #[max_length = 40]
        last_name -> Varchar,
        is_admin -> Bool,
    }
}

diesel::joinable!(dish_to_order -> dishes (dish_id));
diesel::joinable!(dish_to_order -> orders (order_id));
diesel::joinable!(dish_to_product -> dishes (dish_id));
diesel::joinable!(dish_to_product -> products (product_id));
diesel::joinable!(orders -> tables (table_id));
diesel::joinable!(tables -> waiters (waiter_id));

diesel::allow_tables_to_appear_in_same_query!(
    dish_to_order,
    dish_to_product,
    dishes,
    orders,
    products,
    tables,
    waiters,
);
