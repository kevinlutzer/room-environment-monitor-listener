-- Your SQL goes here
CREATE TABLE rem_data (
    id VARCHAR(36) PRIMARY KEY,
    device_id VARCHAR NOT NULL,
    temperature REAL,
    pressure REAL,
    pm2_5  REAL,
    pm1_0  REAL,
    pm10   REAL,
    humidity REAL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);