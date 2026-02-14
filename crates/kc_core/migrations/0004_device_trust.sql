CREATE TABLE IF NOT EXISTS trusted_devices (
  device_id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  pubkey TEXT NOT NULL,
  fingerprint TEXT NOT NULL,
  verified_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_trusted_devices_fingerprint
  ON trusted_devices(fingerprint);

CREATE TABLE IF NOT EXISTS trust_events (
  event_id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT NOT NULL,
  action TEXT NOT NULL,
  actor TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  details_json TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES trusted_devices(device_id)
);

CREATE INDEX IF NOT EXISTS idx_trust_events_device_id_event_id
  ON trust_events(device_id, event_id);
