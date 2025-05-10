-- Add migration script here
ALTER TABLE users ADD COLUMN ws_id bigint REFERENCES workspaces(id) NOT NULL;