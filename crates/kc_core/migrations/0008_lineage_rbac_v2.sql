CREATE TABLE IF NOT EXISTS lineage_roles (
  role_name TEXT PRIMARY KEY,
  role_rank INTEGER NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lineage_permissions (
  role_name TEXT NOT NULL REFERENCES lineage_roles(role_name) ON DELETE CASCADE,
  action TEXT NOT NULL,
  allowed INTEGER NOT NULL CHECK (allowed IN (0, 1)),
  PRIMARY KEY (role_name, action)
);

CREATE TABLE IF NOT EXISTS lineage_role_bindings (
  subject_id TEXT NOT NULL,
  role_name TEXT NOT NULL REFERENCES lineage_roles(role_name) ON DELETE CASCADE,
  granted_by TEXT NOT NULL,
  granted_at_ms INTEGER NOT NULL,
  PRIMARY KEY (subject_id, role_name)
);

CREATE INDEX IF NOT EXISTS idx_lineage_role_bindings_role_subject
  ON lineage_role_bindings(role_name, subject_id);

CREATE TABLE IF NOT EXISTS lineage_lock_scopes (
  scope_kind TEXT NOT NULL,
  scope_value TEXT NOT NULL,
  owner TEXT NOT NULL,
  token TEXT NOT NULL,
  acquired_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  PRIMARY KEY (scope_kind, scope_value)
);

CREATE INDEX IF NOT EXISTS idx_lineage_lock_scopes_expires
  ON lineage_lock_scopes(expires_at_ms, scope_kind, scope_value);

INSERT OR IGNORE INTO lineage_roles(role_name, role_rank, description) VALUES
  ('admin', 10, 'Full lineage governance including role management'),
  ('editor', 20, 'Lineage overlay mutation and lock scope management'),
  ('viewer', 30, 'Read-only lineage access');

INSERT OR IGNORE INTO lineage_permissions(role_name, action, allowed) VALUES
  ('admin', 'lineage.read', 1),
  ('admin', 'lineage.overlay.write', 1),
  ('admin', 'lineage.role.manage', 1),
  ('admin', 'lineage.lock.scope.manage', 1),
  ('editor', 'lineage.read', 1),
  ('editor', 'lineage.overlay.write', 1),
  ('editor', 'lineage.lock.scope.manage', 1),
  ('viewer', 'lineage.read', 1);
