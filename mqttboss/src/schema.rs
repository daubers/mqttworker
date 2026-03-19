// @generated automatically by Diesel CLI.

diesel::table! {
    workers (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        last_seen -> Nullable<Timestamp>,
        cpus -> Nullable<Int4>,
        ram -> Nullable<Int4>,
        disk -> Nullable<Float8>,
        gpu -> Nullable<Int4>,
        tags -> Nullable<Jsonb>,
        available -> Bool,
    }
}
