-- Add migration script here

CREATE TABLE IF NOT EXISTS workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    owner_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- add column ws_id for users and chats after their id
ALTER TABLE users ADD COLUMN ws_id BIGINT NOT NULL;
ALTER TABLE chats ADD COLUMN ws_id BIGINT NOT NULL;

INSERT INTO users (id, fullname, email, password_hash, ws_id) VALUES (0, 'nobody', '', '', 0);
INSERT INTO workspaces(id, name, owner_id) VALUES(0, 'none', 0);
