// @generated automatically by Diesel CLI.

diesel::table! {
    workers (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        capabilities -> Jsonb,
        last_seen -> Timestamp,
    }
}
