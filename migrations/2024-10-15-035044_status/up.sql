-- Your SQL goes here
CREATE TABLE rem_status (
  id VARCHAR(36) PRIMARY KEY,
  device_id VARCHAR NOT NULL,
  up_time INTEGER NOT NULL
)