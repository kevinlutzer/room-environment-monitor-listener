// @generated automatically by Diesel CLI.

diesel::table! {
    rem_status (id) {
        #[max_length = 36]
        id -> Varchar,
        device_id -> Varchar,
        up_time -> Int4,
    }
}
