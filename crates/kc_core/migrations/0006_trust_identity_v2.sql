CREATE TABLE IF NOT EXISTS identity_providers (
  provider_id TEXT PRIMARY KEY,
  issuer TEXT NOT NULL,
  audience TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS identity_sessions (
  session_id TEXT PRIMARY KEY,
  provider_id TEXT NOT NULL,
  subject TEXT NOT NULL,
  claim_subset_json TEXT NOT NULL,
  issued_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(provider_id) REFERENCES identity_providers(provider_id)
);

CREATE INDEX IF NOT EXISTS idx_identity_sessions_provider_created
  ON identity_sessions(provider_id, created_at_ms DESC, session_id DESC);

CREATE TABLE IF NOT EXISTS device_certificates (
  cert_id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL,
  provider_id TEXT NOT NULL,
  subject TEXT NOT NULL,
  cert_chain_hash TEXT NOT NULL,
  issued_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  verified_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(device_id) REFERENCES trusted_devices(device_id),
  FOREIGN KEY(provider_id) REFERENCES identity_providers(provider_id)
);

CREATE INDEX IF NOT EXISTS idx_device_certificates_device_created
  ON device_certificates(device_id, created_at_ms DESC, cert_id DESC);

CREATE INDEX IF NOT EXISTS idx_device_certificates_provider_created
  ON device_certificates(provider_id, created_at_ms DESC, cert_id DESC);
