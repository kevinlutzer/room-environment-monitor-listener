// @generated automatically by Diesel CLI.

diesel::table! {
    rem_data (id) {
        #[max_length = 36]
        id -> Varchar,
        device_id -> Varchar,
        temperature -> Nullable<Float4>,
        pressure -> Nullable<Float4>,
        pm2_5 -> Nullable<Float4>,
        pm1_0 -> Nullable<Float4>,
        pm10 -> Nullable<Float4>,
        humidity -> Nullable<Float4>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    rem_status (id) {
        #[max_length = 36]
        id -> Varchar,
        device_id -> Varchar,
        up_time -> Int4,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(rem_data, rem_status,);
