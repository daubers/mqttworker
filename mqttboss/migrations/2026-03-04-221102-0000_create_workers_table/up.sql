-- Your SQL goes here
CREATE TABLE workers(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    capabilities jsonb NOT NULL,
    last_seen TIMESTAMP NOT NULL
);