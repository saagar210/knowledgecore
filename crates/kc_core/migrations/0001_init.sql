CREATE TABLE IF NOT EXISTS objects (
  object_hash TEXT PRIMARY KEY,
  bytes INTEGER NOT NULL,
  relpath TEXT NOT NULL,
  created_event_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS docs (
  doc_id TEXT PRIMARY KEY,
  original_object_hash TEXT NOT NULL REFERENCES objects(object_hash),
  bytes INTEGER NOT NULL,
  mime TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  effective_ts_ms INTEGER NOT NULL,
  ingested_event_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS doc_sources (
  doc_id TEXT NOT NULL REFERENCES docs(doc_id),
  source_path TEXT NOT NULL,
  PRIMARY KEY (doc_id, source_path)
);

CREATE TABLE IF NOT EXISTS canonical_text (
  doc_id TEXT PRIMARY KEY REFERENCES docs(doc_id),
  canonical_object_hash TEXT NOT NULL REFERENCES objects(object_hash),
  canonical_hash TEXT NOT NULL,
  extractor_name TEXT NOT NULL,
  extractor_version TEXT NOT NULL,
  extractor_flags_json TEXT NOT NULL,
  normalization_version INTEGER NOT NULL,
  toolchain_json TEXT NOT NULL,
  created_event_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
  chunk_id TEXT PRIMARY KEY,
  doc_id TEXT NOT NULL REFERENCES docs(doc_id),
  ordinal INTEGER NOT NULL,
  start_char INTEGER NOT NULL,
  end_char INTEGER NOT NULL,
  chunking_config_hash TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  UNIQUE(doc_id, chunking_config_hash, ordinal)
);

CREATE TABLE IF NOT EXISTS events (
  event_id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts_ms INTEGER NOT NULL,
  type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  prev_event_hash TEXT,
  event_hash TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_docs_effective_ts ON docs(effective_ts_ms);
