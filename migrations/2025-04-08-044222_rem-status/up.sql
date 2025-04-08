-- Your SQL goes here
DROP TABLE IF EXISTS rem_status;
CREATE TABLE rem_status (
    id VARCHAR(36) PRIMARY KEY,
    device_id VARCHAR NOT NULL,
    up_time INTEGER NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);