CREATE TABLE project_versions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    version_number  INTEGER NOT NULL,
    object_snapshot TEXT    NOT NULL,                    -- full JSON snapshot as TEXT
    created_by      INTEGER REFERENCES users(id),
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    UNIQUE (project_id, version_number)
);

CREATE INDEX idx_versions_project ON project_versions(project_id, version_number);
