-- Add migration script here
-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces(
    id bigserial PRIMARY KEY,
    name varchar(32) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
)