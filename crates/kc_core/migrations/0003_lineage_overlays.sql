CREATE TABLE IF NOT EXISTS lineage_overlays (
  overlay_id TEXT PRIMARY KEY,
  doc_id TEXT NOT NULL REFERENCES docs(doc_id),
  from_node_id TEXT NOT NULL,
  to_node_id TEXT NOT NULL,
  relation TEXT NOT NULL,
  evidence TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  created_by TEXT NOT NULL,
  UNIQUE (doc_id, from_node_id, to_node_id, relation, evidence)
);

CREATE INDEX IF NOT EXISTS idx_lineage_overlays_doc_id
  ON lineage_overlays(doc_id, from_node_id, to_node_id, relation, evidence, overlay_id);
