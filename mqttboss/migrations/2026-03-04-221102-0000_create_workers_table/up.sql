-- Your SQL goes here
CREATE TABLE workers(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    capabilities jsonb NULL,
    last_seen TIMESTAMP DEFAULT NOW()
);