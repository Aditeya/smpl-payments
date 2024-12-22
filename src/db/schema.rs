// @generated automatically by Diesel CLI.

diesel::table! {
    transaction (id) {
        id -> Int4,
        from_wallet -> Int4,
        to_wallet -> Int4,
        amount -> Numeric,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 120]
        username -> Varchar,
        #[max_length = 120]
        email -> Varchar,
        #[max_length = 100]
        password -> Varchar,
        status -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    wallet (id) {
        id -> Int4,
        user_id -> Int4,
        balance -> Numeric,
        status -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(wallet -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(transaction, users, wallet,);
