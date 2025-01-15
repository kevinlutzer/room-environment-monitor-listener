use std::sync::Arc;

use diesel::{insert_into, pg::PgConnection, prelude::*};
use tokio::sync::Mutex;

use crate::schema::rem_data::dsl::{
    device_id as rem_data_device_id, humidity, id as rem_data_id, pm10, pm1_0, pm2_5, pressure,
    rem_data, temperature,
};
use crate::schema::rem_status::dsl::{
    device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
};


pub struct REMRepo {
    db: Arc<Mutex<PgConnection>>,
}

impl REMRepo {
    /// REMRepo constructor, this creates a new instance of the 
    /// REMRepo struct with the passed db connection instance
    fn new(db: Arc<Mutex<PgConnection>>) -> Self {
        REMRepo { db }
    }

    /// This function is used to insert a new record into the database
    /// it takes a reference to the REM struct and returns a Result
    /// with the inserted REM struct or an error
    pub fn insert_rem_data(data: REMData) -> Result<> {
        let a = (
            rem_data_id.eq(data.id.clone()),
            rem_data_device_id.eq(data.device_id),
            temperature.eq(data.temperature),
            pressure.eq(data.pressure),
            pm2_5.eq(data.pm2_5),
            pm1_0.eq(data.pm1_0),
            pm10.eq(data.pm10),
            humidity.eq(data.humidity),
        );
    
        // Lock on the Database
        let mut mut_conn = conn.lock().await;
        insert_into(rem_data)
            .values(a)
            .execute(&mut *mut_conn)
            .map_err(|e| mqtt_error_from_database(e, data.id.clone()))?;
    }
}
