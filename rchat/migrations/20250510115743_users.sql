-- Add migration script here
CREATE TABLE IF NOT EXISTS users(
    id bigserial PRIMARY KEY,
    fullname varchar(64) NOT NULL,
    email varchar(64) NOT NULL,
    -- hashed argon2 password
    password_hash VARCHAR(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);