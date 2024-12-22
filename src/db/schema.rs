// @generated automatically by Diesel CLI.

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
