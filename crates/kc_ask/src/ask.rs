use crate::trace::{write_trace_log, TraceLogV1};
use kc_core::app_error::{AppError, AppResult};
use kc_core::index_traits::LexicalCandidate;
use kc_core::locator::LocatorV1;
use kc_core::object_store::ObjectStore;
use kc_core::retrieval::{merge_candidates, RecencyConfigV1, RetrievalConfigV1};
use kc_core::types::ChunkId;
use kc_core::vault::vault_open;
use kc_core::{db::open_db, locator::resolve_locator_strict, vault::vault_paths};
use rusqlite::Connection;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AskRequest {
    pub vault_path: std::path::PathBuf,
    pub question: String,
    pub now_ms: i64,
}

#[derive(Debug, Clone)]
pub struct AskResponse {
    pub answer_text: String,
    pub citations: Vec<(i64, Vec<LocatorV1>)>,
    pub trace_path: std::path::PathBuf,
}

pub trait AskService: Send + Sync {
    fn ask(&self, req: AskRequest) -> AppResult<AskResponse>;
}

#[derive(Debug, Clone)]
pub struct RetrievedContext {
    pub chunk_id: ChunkId,
    pub ordinal: i64,
    pub final_score: f64,
    pub locator: LocatorV1,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct ProviderAnswer {
    pub answer_text: String,
    pub citations: Vec<(i64, Vec<LocatorV1>)>,
}

pub trait AskProvider: Send + Sync {
    fn answer(&self, question: &str, contexts: &[RetrievedContext]) -> AppResult<ProviderAnswer>;
}

#[derive(Debug, Default)]
pub struct DeterministicAskProvider;

impl AskProvider for DeterministicAskProvider {
    fn answer(&self, question: &str, contexts: &[RetrievedContext]) -> AppResult<ProviderAnswer> {
        let first = contexts.first().ok_or_else(|| {
            AppError::new(
                "KC_ASK_PROVIDER_UNAVAILABLE",
                "ask",
                "no retrieved context was available",
                true,
                serde_json::json!({}),
            )
        })?;
        let first_line = first
            .snippet
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(ProviderAnswer {
            answer_text: format!("Q: {}\nA: {}", question.trim(), first_line),
            citations: vec![(0, vec![first.locator.clone()])],
        })
    }
}

pub struct RetrievedOnlyAskService {
    pub trace_dir_name: String,
    pub provider: Arc<dyn AskProvider>,
}

impl Default for RetrievedOnlyAskService {
    fn default() -> Self {
        Self {
            trace_dir_name: "trace".to_string(),
            provider: Arc::new(DeterministicAskProvider),
        }
    }
}

fn validate_citations(citations: &[(i64, Vec<LocatorV1>)]) -> AppResult<()> {
    if citations.is_empty() {
        return Err(AppError::new(
            "KC_ASK_MISSING_CITATIONS",
            "ask",
            "answer must include citations for each paragraph",
            false,
            serde_json::json!({}),
        ));
    }

    for (paragraph_idx, locators) in citations {
        if *paragraph_idx < 0 {
            return Err(AppError::new(
                "KC_ASK_INVALID_CITATIONS",
                "ask",
                "paragraph_index must be non-negative",
                false,
                serde_json::json!({ "paragraph_index": paragraph_idx }),
            ));
        }
        if locators.is_empty() {
            return Err(AppError::new(
                "KC_ASK_INVALID_CITATIONS",
                "ask",
                "each paragraph citation entry must include at least one locator",
                false,
                serde_json::json!({ "paragraph_index": paragraph_idx }),
            ));
        }
        for locator in locators {
            if locator.v != 1 {
                return Err(AppError::new(
                    "KC_ASK_INVALID_CITATIONS",
                    "ask",
                    "locator version is invalid",
                    false,
                    serde_json::json!({ "paragraph_index": paragraph_idx, "locator_version": locator.v }),
                ));
            }
        }
    }
    Ok(())
}

fn sort_locator_list(locators: &mut [LocatorV1]) {
    locators.sort_by(|a, b| {
        a.doc_id
            .0
            .cmp(&b.doc_id.0)
            .then(a.range.start.cmp(&b.range.start))
            .then(a.range.end.cmp(&b.range.end))
    });
}

fn normalize_citations(citations: &[(i64, Vec<LocatorV1>)]) -> Vec<(i64, Vec<LocatorV1>)> {
    let mut normalized: Vec<(i64, Vec<LocatorV1>)> = citations
        .iter()
        .map(|(paragraph, locators)| {
            let mut sorted_locators = locators.clone();
            sort_locator_list(&mut sorted_locators);
            (*paragraph, sorted_locators)
        })
        .collect();
    normalized.sort_by(|a, b| {
        let af = a.1.first();
        let bf = b.1.first();
        a.0.cmp(&b.0).then(
            af.map(|x| (&x.doc_id.0, x.range.start, x.range.end))
                .cmp(&bf.map(|x| (&x.doc_id.0, x.range.start, x.range.end))),
        )
    });
    normalized
}

impl RetrievedOnlyAskService {
    fn table_exists(conn: &Connection, table: &str) -> AppResult<bool> {
        let mut stmt = conn
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 LIMIT 1")
            .map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "failed preparing sqlite master query",
                    true,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        let mut rows = stmt.query([table]).map_err(|e| {
            AppError::new(
                "KC_ASK_PROVIDER_UNAVAILABLE",
                "ask",
                "failed querying sqlite master",
                true,
                serde_json::json!({ "error": e.to_string(), "table": table }),
            )
        })?;
        Ok(rows
            .next()
            .map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "failed reading sqlite master query result",
                    true,
                    serde_json::json!({ "error": e.to_string(), "table": table }),
                )
            })?
            .is_some())
    }

    fn lexical_candidates(
        conn: &Connection,
        question: &str,
        limit: usize,
    ) -> AppResult<Vec<LexicalCandidate>> {
        let mut candidates = Vec::new();
        let q = question.trim();
        if !q.is_empty() && Self::table_exists(conn, "chunks_fts")? {
            let mut stmt = conn
                .prepare("SELECT chunk_id FROM chunks_fts WHERE chunks_fts MATCH ?1 ORDER BY rank LIMIT ?2")
                .map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed preparing lexical query",
                        true,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
            let rows_result = stmt.query_map(rusqlite::params![q, limit as i64], |row| {
                row.get::<_, String>(0)
            });
            if let Ok(rows) = rows_result {
                for (idx, row) in rows.enumerate() {
                    if let Ok(chunk_id) = row {
                        candidates.push(LexicalCandidate {
                            chunk_id: ChunkId(chunk_id),
                            rank: idx as i64 + 1,
                        });
                    }
                }
            }
        }

        if candidates.is_empty() {
            let mut stmt = conn
                .prepare(
                    "SELECT c.chunk_id
                     FROM chunks c
                     JOIN docs d ON d.doc_id=c.doc_id
                     ORDER BY d.effective_ts_ms DESC, c.doc_id ASC, c.ordinal ASC, c.chunk_id ASC
                     LIMIT ?1",
                )
                .map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed preparing fallback retrieval query",
                        true,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
            let rows = stmt
                .query_map([limit as i64], |row| row.get::<_, String>(0))
                .map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed executing fallback retrieval query",
                        true,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
            for (idx, row) in rows.enumerate() {
                let chunk_id = row.map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed loading fallback retrieval row",
                        true,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
                candidates.push(LexicalCandidate {
                    chunk_id: ChunkId(chunk_id),
                    rank: idx as i64 + 1,
                });
            }
        }

        Ok(candidates)
    }

    fn load_contexts(
        &self,
        conn: &Connection,
        object_store: &ObjectStore,
        question: &str,
        now_ms: i64,
        recency_enabled: bool,
    ) -> AppResult<Vec<RetrievedContext>> {
        let lexical = Self::lexical_candidates(conn, question, 32)?;
        if lexical.is_empty() {
            return Ok(Vec::new());
        }

        let retrieval_cfg = RetrievalConfigV1 {
            rrf_k: 60,
            w_lex: 1.0,
            w_vec: 1.0,
            recency: RecencyConfigV1 {
                enabled: recency_enabled,
                window_days: 365,
                max_boost: 0.20,
            },
        };
        let merged = merge_candidates(
            &lexical,
            &[],
            |chunk_id| {
                conn.query_row(
                    "SELECT c.doc_id, c.ordinal, d.source_kind, d.effective_ts_ms
                     FROM chunks c
                     JOIN docs d ON d.doc_id=c.doc_id
                     WHERE c.chunk_id=?1",
                    [chunk_id.0.clone()],
                    |row| {
                        Ok((
                            kc_core::types::DocId(row.get::<_, String>(0)?),
                            row.get::<_, i64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    },
                )
                .map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed loading chunk metadata for retrieval",
                        true,
                        serde_json::json!({ "error": e.to_string(), "chunk_id": chunk_id.0 }),
                    )
                })
            },
            &retrieval_cfg,
            now_ms,
        )?;

        let mut contexts = Vec::new();
        for merged_hit in merged.into_iter().take(5) {
            let (doc_id, start_char, end_char, canonical_hash, canonical_object_hash) = conn
                .query_row(
                    "SELECT c.doc_id, c.start_char, c.end_char, ct.canonical_hash, ct.canonical_object_hash
                     FROM chunks c
                     JOIN canonical_text ct ON ct.doc_id=c.doc_id
                     WHERE c.chunk_id=?1",
                    [merged_hit.chunk_id.0.clone()],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                        ))
                    },
                )
                .map_err(|e| {
                    AppError::new(
                        "KC_ASK_PROVIDER_UNAVAILABLE",
                        "ask",
                        "failed loading canonical metadata for merged chunk",
                        true,
                        serde_json::json!({ "error": e.to_string(), "chunk_id": merged_hit.chunk_id.0 }),
                    )
                })?;
            let bytes =
                object_store.get_bytes(&kc_core::types::ObjectHash(canonical_object_hash))?;
            let text = String::from_utf8(bytes).map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "canonical text is not utf8",
                    false,
                    serde_json::json!({ "error": e.to_string(), "doc_id": doc_id }),
                )
            })?;
            let total = text.chars().count() as i64;
            let clamped_start = start_char.clamp(0, total);
            let clamped_end = end_char.clamp(clamped_start, total);
            let locator = LocatorV1 {
                v: 1,
                doc_id: kc_core::types::DocId(doc_id),
                canonical_hash: kc_core::types::CanonicalHash(canonical_hash),
                range: kc_core::locator::LocatorRange {
                    start: clamped_start,
                    end: clamped_end,
                },
                hints: None,
            };

            let snippet = resolve_locator_strict(conn, object_store, &locator)?;
            contexts.push(RetrievedContext {
                chunk_id: merged_hit.chunk_id,
                ordinal: merged_hit.ordinal,
                final_score: merged_hit.final_score,
                locator,
                snippet,
            });
        }
        Ok(contexts)
    }

    fn finalize_answer_with_retrieval(
        &self,
        req: &AskRequest,
        answer_text: String,
        citations: Vec<(i64, Vec<LocatorV1>)>,
        retrieval_json: serde_json::Value,
    ) -> AppResult<AskResponse> {
        let normalized_citations = normalize_citations(&citations);
        validate_citations(&normalized_citations)?;

        let vault = vault_open(&req.vault_path)?;
        let trace = TraceLogV1 {
            schema_version: 1,
            trace_id: uuid::Uuid::new_v4().to_string(),
            ts_ms: req.now_ms,
            vault_id: vault.vault_id,
            question: req.question.clone(),
            retrieval: retrieval_json,
            model: serde_json::json!({ "mode": "retrieved-only" }),
            answer: serde_json::json!({ "text": answer_text }),
            redaction: serde_json::json!({ "enabled": true }),
        };

        let trace_path = write_trace_log(
            &req.vault_path.join(&self.trace_dir_name),
            &trace,
            &normalized_citations,
        )?;

        Ok(AskResponse {
            answer_text,
            citations: normalized_citations,
            trace_path,
        })
    }

    pub fn finalize_answer(
        &self,
        req: &AskRequest,
        answer_text: String,
        citations: Vec<(i64, Vec<LocatorV1>)>,
    ) -> AppResult<AskResponse> {
        self.finalize_answer_with_retrieval(req, answer_text, citations, serde_json::json!({}))
    }
}

impl AskService for RetrievedOnlyAskService {
    fn ask(&self, req: AskRequest) -> AppResult<AskResponse> {
        let vault = vault_open(&req.vault_path)?;
        let conn = open_db(&req.vault_path.join(vault.db.relative_path))?;
        let object_store = ObjectStore::new(vault_paths(&req.vault_path).objects_dir);
        let contexts = self.load_contexts(
            &conn,
            &object_store,
            &req.question,
            req.now_ms,
            vault.defaults.recency.enabled,
        )?;

        if contexts.is_empty() {
            return Err(AppError::new(
                "KC_ASK_PROVIDER_UNAVAILABLE",
                "ask",
                "no retrieved context available for ask",
                true,
                serde_json::json!({}),
            ));
        }

        let provider_answer = self.provider.answer(&req.question, &contexts)?;

        // Validate every cited locator can be resolved against canonical text.
        for (_paragraph, locators) in &provider_answer.citations {
            for locator in locators {
                let _ = resolve_locator_strict(&conn, &object_store, locator)?;
            }
        }

        let retrieval_json = serde_json::json!({
            "chunks": contexts
                .iter()
                .map(|ctx| serde_json::json!({
                    "chunk_id": ctx.chunk_id.0.clone(),
                    "doc_id": ctx.locator.doc_id.0.clone(),
                    "ordinal": ctx.ordinal,
                    "final_score": ctx.final_score,
                    "range": {
                        "start": ctx.locator.range.start,
                        "end": ctx.locator.range.end
                    },
                    "snippet": ctx.snippet.clone()
                }))
                .collect::<Vec<_>>(),
        });

        self.finalize_answer_with_retrieval(
            &req,
            provider_answer.answer_text,
            provider_answer.citations,
            retrieval_json,
        )
    }
}
