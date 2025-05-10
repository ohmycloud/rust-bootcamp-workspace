-- Add migration script here
-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM(
  'single',
  'group',
  'private_channel',
  'public_channel'
);

-- create chat table
CREATE TABLE IF NOT EXISTS chats(
    id bigserial PRIMARY KEY,
    name varchar(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    -- user id list
    members bigint[] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);