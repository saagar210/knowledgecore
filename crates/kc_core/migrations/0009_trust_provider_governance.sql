CREATE TABLE IF NOT EXISTS trust_providers (
  provider_id TEXT PRIMARY KEY,
  issuer TEXT NOT NULL,
  audience TEXT NOT NULL,
  jwks_url TEXT NOT NULL,
  enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

INSERT OR IGNORE INTO trust_providers(
  provider_id, issuer, audience, jwks_url, enabled, created_at_ms, updated_at_ms
)
SELECT
  provider_id,
  issuer,
  audience,
  issuer || '/.well-known/jwks.json',
  enabled,
  created_at_ms,
  created_at_ms
FROM identity_providers;

CREATE TABLE IF NOT EXISTS trust_provider_policies (
  provider_id TEXT PRIMARY KEY REFERENCES trust_providers(provider_id) ON DELETE CASCADE,
  max_clock_skew_ms INTEGER NOT NULL CHECK (max_clock_skew_ms >= 0),
  require_claims_json TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS trust_session_revocations (
  session_id TEXT PRIMARY KEY REFERENCES identity_sessions(session_id) ON DELETE CASCADE,
  revoked_by TEXT NOT NULL,
  revoked_at_ms INTEGER NOT NULL,
  details_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_trust_providers_enabled_provider
  ON trust_providers(enabled, provider_id);

CREATE INDEX IF NOT EXISTS idx_trust_session_revocations_revoked_at
  ON trust_session_revocations(revoked_at_ms DESC, session_id DESC);
