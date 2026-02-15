CREATE TABLE IF NOT EXISTS lineage_policies (
  policy_id TEXT PRIMARY KEY,
  policy_name TEXT NOT NULL UNIQUE,
  effect TEXT NOT NULL CHECK (effect IN ('allow', 'deny')),
  priority INTEGER NOT NULL,
  condition_json TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS lineage_policy_bindings (
  subject_id TEXT NOT NULL,
  policy_id TEXT NOT NULL REFERENCES lineage_policies(policy_id) ON DELETE CASCADE,
  bound_by TEXT NOT NULL,
  bound_at_ms INTEGER NOT NULL,
  PRIMARY KEY (subject_id, policy_id)
);

CREATE INDEX IF NOT EXISTS idx_lineage_policy_bindings_order
  ON lineage_policy_bindings(subject_id, policy_id);

CREATE TABLE IF NOT EXISTS lineage_policy_audit (
  audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts_ms INTEGER NOT NULL,
  subject_id TEXT NOT NULL,
  action TEXT NOT NULL,
  doc_id TEXT,
  allowed INTEGER NOT NULL CHECK (allowed IN (0, 1)),
  reason TEXT NOT NULL,
  matched_policy_id TEXT,
  details_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lineage_policy_audit_ts
  ON lineage_policy_audit(ts_ms, audit_id);
