-- Add migration script here
-- add superuser 0 and workspace 0
BEGIN;

INSERT INTO users(id, fullname, email, password_hash)
VALUES (0, 'superuser', 'super@none.org', '');

INSERT INTO workspaces(id, name, owner_id)
VALUES (0, 'none', 0);

UPDATE users SET ws_id = 0 where id = 0;
COMMIT;

-- alter user table to make ws_id not null
ALTER TABLE users ALTER COLUMN ws_id SET NOT NULL;