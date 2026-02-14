CREATE TABLE IF NOT EXISTS lineage_edit_locks (
  doc_id TEXT PRIMARY KEY REFERENCES docs(doc_id),
  owner TEXT NOT NULL,
  token TEXT NOT NULL,
  acquired_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lineage_edit_locks_expires
  ON lineage_edit_locks(expires_at_ms, doc_id);
