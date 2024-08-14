-- password 123456
insert into users (fullname, email, password_hash, ws_id)
values
    ('user-1', 'user-1@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 1),
    ('user-2', 'user-2@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 1),
    ('user-3', 'user-3@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 2),
    ('user-4', 'user-4@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 2),
    ('user-5', 'user-5@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 0)
;

insert into workspaces (name, owner_id)
values
    -- user 1 own the ws-1
    ('ws-1', 1),
    -- user 3 own the ws-2
    ('ws-2', 3)
;

insert into chats (ws_id, name, type, members)
values
    -- user 1 and 2 chat in single in ws-1
    (1, null, 'single', '{1, 2}'),
    -- user 3 and 4 chat in public_channel in ws-2
    (2, 'chat-u34', 'public_channel', '{3, 4}')
;

insert into messages (chat_id, sender_id, content)
values
    -- user 1 send hello
    (1, 1, 'hello'),
    -- user 2 send nice
    (1, 2, 'nice'),
    -- user 3 send hell
    (2, 3, 'hell'),
    -- user 4 send nc
    (2, 4, 'nc')
;
