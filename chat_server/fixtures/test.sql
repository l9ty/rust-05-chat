-- password 123456
insert into users (fullname, email, password_hash, ws_id)
values
    ('user-1', 'user-1@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 1),
    ('user-2', 'user-2@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 2),
    ('user-3', 'user-3@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 0),
    ('user-4', 'user-4@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 0),
    ('user-5', 'user-5@a.com', '$argon2id$v=19$m=19456,t=2,p=1$4O6qd5vvQqG2LkHKu6kWmQ$CXHGt3p/w3rn7zA27KXgoqQf3KedFlgdtDWodZIvLUM', 0)
;

insert into workspaces (name, owner_id)
values
    ('ws-1', 1),
    ('ws-2', 2)
;

insert into chats (ws_id, name, type, members)
values
    (1, null, 'single', '{1, 2}'),
    (0, 'chat-1', 'public_channel', '{3, 4}')
;
