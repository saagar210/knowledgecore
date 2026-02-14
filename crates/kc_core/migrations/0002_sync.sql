CREATE TABLE IF NOT EXISTS sync_state (
  state_key TEXT PRIMARY KEY,
  state_value TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  direction TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  bundle_relpath TEXT NOT NULL,
  manifest_hash TEXT NOT NULL
);
