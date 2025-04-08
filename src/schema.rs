// @generated automatically by Diesel CLI.

diesel::table! {
    rem_data (id) {
        #[max_length = 36]
        id -> Varchar,
        device_id -> Varchar,
        pm2_5 -> Float4,
        pm10 -> Float4,
        pm1_0 -> Float4,
        temperature -> Float4,
        pressure -> Float4,
        humidity -> Float4,
        voc_index -> Float4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    rem_status (id) {
        #[max_length = 36]
        id -> Varchar,
        device_id -> Varchar,
        up_time -> Int4,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    rem_data,
    rem_status,
);
