-- Add migration script here
ALTER TABLE users ADD COLUMN ws_id bigint REFERENCES workspaces(id);
ALTER TABLE chats ADD COLUMN ws_id bigint REFERENCES workspaces(id);