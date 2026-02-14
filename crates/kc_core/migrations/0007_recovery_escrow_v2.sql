CREATE TABLE IF NOT EXISTS recovery_escrow_configs (
  provider_id TEXT PRIMARY KEY,
  enabled INTEGER NOT NULL,
  descriptor_json TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS recovery_escrow_events (
  event_id INTEGER PRIMARY KEY AUTOINCREMENT,
  provider_id TEXT NOT NULL,
  action TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  details_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recovery_escrow_events_provider_ts
  ON recovery_escrow_events(provider_id, ts_ms DESC, event_id DESC);
