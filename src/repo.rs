use std::sync::Arc;

use chrono::NaiveDateTime;
use diesel::{insert_into, pg::PgConnection, prelude::*};
use tokio::sync::Mutex;

use crate::{
    model::{RemData, RemStatus},
    schema::{
        rem_data::dsl::{
            device_id as rem_data_device_id, humidity as rem_data_humidity, id as rem_data_id,
            pm10 as rem_data_pm10, pm1_0 as rem_data_pm1_0, pm2_5 as rem_data_pm2_5,
            pressure as rem_data_pressure, rem_data, temperature as rem_data_temperature,
            voc_index as rem_data_voc_index,
        },
        rem_status::dsl::{
            device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
        },
    },
};

use diesel::prelude::{Queryable, QueryableByName, Selectable};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemRepoError {
    #[error("Database entry already exists for key: {}", .0)]
    DataEntryExists(String),
    #[error("Database error: {}", .0)]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Invalid message")]
    InvalidMessage,
}

/// REMStatus is the structure of the status that we receive from the REM device.
#[derive(Queryable, QueryableByName, Selectable, Debug)]
#[diesel(table_name = crate::schema::rem_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct REMStatusDB {
    pub id: String,
    pub device_id: String,
    pub created_at: NaiveDateTime,
}

/// RemData is the structure of the data that we receive from the REM device.
#[derive(Queryable, QueryableByName, Selectable, Debug)]
#[diesel(table_name = crate::schema::rem_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RemDataDB {
    pub id: String,
    pub device_id: String,

    pub pm2_5: f32,
    pub pm1_0: f32,
    pub pm10: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub voc_index: f32,

    pub created_at: NaiveDateTime,
}

fn repo_error_from_database(e: diesel::result::Error, key: String) -> RemRepoError {
    // Only error type for a duplicate key violation is violation error
    if matches!(
        e,
        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
    ) {
        return RemRepoError::DataEntryExists(key);
    }

    RemRepoError::DatabaseError(e)
}

pub struct RemRepo {
    db: Arc<Mutex<PgConnection>>,
}

impl RemRepo {
    /// RemRepo constructor, this creates a new instance of the
    /// struct with the passed db connection instance
    pub fn new(db: Arc<Mutex<PgConnection>>) -> Self {
        RemRepo { db }
    }

    pub async fn list_data(&self) -> Result<Vec<RemDataDB>, RemRepoError> {
        let mut mut_conn = self.db.lock().await;
        let data = rem_data.load::<RemDataDB>(&mut *mut_conn)?;

        Ok(data)
    }

    /// This function is used to insert a new record into the database
    /// it takes a reference to the REM struct and returns a Result
    /// with the inserted REM struct or an error
    pub async fn insert_rem_data(&self, data: RemData) -> Result<(), RemRepoError> {
        let a = (
            rem_data_id.eq(data.id.clone()),
            rem_data_device_id.eq(data.device_id),
            rem_data_temperature.eq(data.temperature),
            rem_data_pressure.eq(data.pressure),
            rem_data_pm2_5.eq(data.pm2_5),
            rem_data_pm1_0.eq(data.pm1_0),
            rem_data_pm10.eq(data.pm10),
            rem_data_humidity.eq(data.humidity),
            rem_data_voc_index.eq(data.voc_index),
        );

        // Lock on the Database
        let mut mut_conn = self.db.lock().await;
        insert_into(rem_data)
            .values(a)
            .execute(&mut *mut_conn)
            .map_err(|e| repo_error_from_database(e, data.id.clone()))?;

        Ok(())
    }

    /// Handle the status message from the REM device and insert it into the database. If there
    /// is any issue with insertion it will return a RemRepoError type
    pub async fn insert_rem_status(&self, status: RemStatus) -> Result<(), RemRepoError> {
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
            .map(|_| ())
            .map_err(|e| repo_error_from_database(e, status.id.clone()))
    }
}
