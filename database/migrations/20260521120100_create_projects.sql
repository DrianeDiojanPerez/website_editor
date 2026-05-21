CREATE TABLE projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    current_version INTEGER NOT NULL DEFAULT 1,
    object_data     TEXT    NOT NULL DEFAULT '{}',      -- JSON stored as TEXT
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);
