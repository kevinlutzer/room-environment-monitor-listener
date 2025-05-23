-- Your SQL goes here
DROP TABLE IF EXISTS rem_data;
CREATE TABLE rem_data (
    id VARCHAR(36) PRIMARY KEY,
    device_id VARCHAR NOT NULL,

    pm2_5  REAL NOT NULL,
    pm10 REAL NOT NULL,
    pm1_0 REAL NOT NULL,
    temperature REAL NOT NULL,
    pressure REAL NOT NULL,
    humidity REAL NOT NULL,
    voc_index REAL NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
