-- insert workspaces
INSERT INTO workspaces (name, owner_id)
VALUES ('acme', 0),
       ('foo', 0),
       ('bar', 0);

-- insert users
INSERT INTO users(ws_id, email, fullname, password_hash)
VALUES (1, 'alice@acme.org', 'Alice Chen', '$argon2id$v=19$m=19456,t=2,p=1$O5mn2rjMn/kIEIkgbB//mw$bFFlTgTUtYh4KyffEdFBUgACoGVCwG0p/xtNpayNHLQ'),
       (1, 'bob@acme.org', 'Bob Wang', '$argon2id$v=19$m=19456,t=2,p=1$O5mn2rjMn/kIEIkgbB//mw$bFFlTgTUtYh4KyffEdFBUgACoGVCwG0p/xtNpayNHLQ'),
       (1, 'tom@acme.org', 'Tom Zhang', '$argon2id$v=19$m=19456,t=2,p=1$O5mn2rjMn/kIEIkgbB//mw$bFFlTgTUtYh4KyffEdFBUgACoGVCwG0p/xtNpayNHLQ'),
       (1, 'lilei@acme.org', 'Lei Li', '$argon2id$v=19$m=19456,t=2,p=1$O5mn2rjMn/kIEIkgbB//mw$bFFlTgTUtYh4KyffEdFBUgACoGVCwG0p/xtNpayNHLQ'),
       (1, 'lisa@acme.org', 'LiSa', '$argon2id$v=19$m=19456,t=2,p=1$O5mn2rjMn/kIEIkgbB//mw$bFFlTgTUtYh4KyffEdFBUgACoGVCwG0p/xtNpayNHLQ');

-- insert public/private channel
INSERT INTO chats(ws_id, name, type, members)
VALUES (1, 'general', 'public_channel', '{1,2,3,4,5}'),
       (1, 'private', 'private_channel', '{1,2,3}');

-- insert unnamed chat
INSERT INTO chats(ws_id, type, members)
VALUES (1, 'single', '{1,2}'),
       (1, 'group', '{1,3,4}');

-- insert some messages
INSERT INTO messages(chat_id, sender_id, content)
VALUES (1, 1, 'Hello, world'),
(1, 2, 'Hi, there!'),
(1, 3, 'How are you?'),
(1, 4, 'I am fine, thank you!'),
(1, 5, 'Good to hear that!'),
(1, 1, 'Hello, world!'),
(1, 2, 'Hi, there!'),
(1, 3, 'Hello, Raku'),
(1, 1, 'Hi, Rust!'),
(1, 1, 'Hi, Larry Walls!');