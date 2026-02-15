CREATE TABLE IF NOT EXISTS recovery_escrow_provider_configs (
  provider_id TEXT PRIMARY KEY,
  provider_priority INTEGER NOT NULL,
  config_ref TEXT NOT NULL,
  enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
  updated_at_ms INTEGER NOT NULL
);

INSERT OR REPLACE INTO recovery_escrow_provider_configs(
  provider_id,
  provider_priority,
  config_ref,
  enabled,
  updated_at_ms
)
SELECT
  provider_id,
  CASE provider_id
    WHEN 'aws' THEN 0
    WHEN 'gcp' THEN 1
    WHEN 'azure' THEN 2
    ELSE 9
  END,
  descriptor_json,
  enabled,
  updated_at_ms
FROM recovery_escrow_configs
;

INSERT OR IGNORE INTO recovery_escrow_provider_configs(provider_id, provider_priority, config_ref, enabled, updated_at_ms)
VALUES
  ('aws', 0, '{}', 0, 0),
  ('gcp', 1, '{}', 0, 0),
  ('azure', 2, '{}', 0, 0);

CREATE INDEX IF NOT EXISTS idx_recovery_escrow_provider_configs_priority
  ON recovery_escrow_provider_configs(provider_priority ASC, provider_id ASC);
