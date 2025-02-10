use std::sync::Arc;

use diesel::{insert_into, pg::PgConnection, prelude::*};
use tokio::sync::Mutex;

use crate::model::{REMData, REMStatus};
use crate::schema::{
    rem_data::dsl::{
        device_id as rem_data_device_id, humidity, id as rem_data_id, pm10, pm1_0, pm2_5, pressure,
        rem_data, temperature,
    },
    rem_status::dsl::{
        device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
    },
};

use super::error::REMRepoError;

fn repo_error_from_database(e: diesel::result::Error, key: String) -> REMRepoError {
    // Only error type for a duplicate key violation is violation error
    if matches!(
        e,
        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
    ) {
        return REMRepoError::DataEntryExists(key);
    }

    REMRepoError::DatabaseError(e)
}

pub struct REMRepo {
    db: Arc<Mutex<PgConnection>>,
}

impl REMRepo {
    /// REMRepo constructor, this creates a new instance of the
    /// REMRepo struct with the passed db connection instance
    pub fn new(db: Arc<Mutex<PgConnection>>) -> Self {
        REMRepo { db }
    }

    /// This function is used to insert a new record into the database
    /// it takes a reference to the REM struct and returns a Result
    /// with the inserted REM struct or an error
    pub async fn insert_rem_data(&self, data: REMData) -> Result<(), REMRepoError> {
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
        let mut mut_conn = self.db.lock().await;
        insert_into(rem_data)
            .values(a)
            .execute(&mut *mut_conn)
            .map_err(|e| repo_error_from_database(e, data.id.clone()))?;

        Ok(())
    }

    /// Handle the status message from the REM device and insert it into the database
    pub async fn insert_rem_status(&self, status: REMStatus) -> Result<(), REMRepoError> {
        let r = (
            rem_status_id.eq(status.id.clone()),
            rem_status_device_id.eq(status.device_id),
            up_time.eq(status.up_time),
        );

        // Lock on the Database
        let mut mut_conn = self.db.lock().await;
        insert_into(rem_status)
            .values(r)
            .execute(&mut *mut_conn)
            .map(|op| ())
            .map_err(|e| repo_error_from_database(e, status.id.clone()))
    }
}
