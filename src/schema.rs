// @generated automatically by Diesel CLI.

diesel::table! {
    attachments (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        product_id -> Uuid,
        url -> Text,
        is_primary -> Bool,
    }
}

diesel::table! {
    order_product (order_id, product_id) {
        order_id -> Uuid,
        product_id -> Uuid,
        quantity -> Int4,
    }
}

diesel::table! {
    orders (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        user_id -> Uuid,
        payment_mode -> Text,
        mollie_payment_id -> Nullable<Text>,
        mollie_payment_url -> Nullable<Text>,
        status -> Text,
    }
}

diesel::table! {
    product_categories (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        order -> Nullable<Int4>,
    }
}

diesel::table! {
    product_category_translations (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        product_category_id -> Uuid,
        name -> Text,
        locale -> Text,
    }
}

diesel::table! {
    product_product_category (product_id, product_category_id) {
        product_id -> Uuid,
        product_category_id -> Uuid,
    }
}

diesel::table! {
    product_translations (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        product_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        locale -> Text,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        price -> Nullable<Float8>,
        is_active -> Bool,
        code -> Nullable<Text>,
        slug -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        name -> Text,
        email -> Text,
        email_verified_at -> Nullable<Timestamp>,
        password -> Text,
        salt -> Text,
        remember_token -> Nullable<Text>,
    }
}

diesel::joinable!(attachments -> products (product_id));
diesel::joinable!(order_product -> orders (order_id));
diesel::joinable!(order_product -> products (product_id));
diesel::joinable!(product_category_translations -> product_categories (product_category_id));
diesel::joinable!(product_product_category -> product_categories (product_category_id));
diesel::joinable!(product_product_category -> products (product_id));
diesel::joinable!(product_translations -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    attachments,
    order_product,
    orders,
    product_categories,
    product_category_translations,
    product_product_category,
    product_translations,
    products,
    users,
);
