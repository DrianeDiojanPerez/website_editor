CREATE TABLE project_members (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id     INTEGER NOT NULL REFERENCES users(id)    ON DELETE CASCADE,
    role        TEXT    NOT NULL DEFAULT 'viewer'
                    CHECK (role IN ('editor','viewer')),

    UNIQUE (project_id, user_id)
);

CREATE INDEX idx_members_project ON project_members(project_id);
CREATE INDEX idx_members_user    ON project_members(user_id);
