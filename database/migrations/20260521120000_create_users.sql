CREATE TABLE users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT    NOT NULL UNIQUE,
    email           TEXT    NOT NULL UNIQUE,
    password        TEXT    NOT NULL,
    is_system       INTEGER NOT NULL DEFAULT 0,        -- 1 = system user, cannot be deleted
    last_login_at   TEXT,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_users_email ON users(email);
