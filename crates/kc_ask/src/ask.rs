use crate::trace::{write_trace_log, TraceLogV1};
use kc_core::app_error::{AppError, AppResult};
use kc_core::locator::LocatorV1;
use kc_core::object_store::ObjectStore;
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

impl RetrievedOnlyAskService {
    fn load_contexts(
        &self,
        conn: &Connection,
        object_store: &ObjectStore,
    ) -> AppResult<Vec<RetrievedContext>> {
        let mut stmt = conn
            .prepare(
                "SELECT d.doc_id, c.canonical_hash, c.canonical_object_hash
                 FROM docs d
                 JOIN canonical_text c ON c.doc_id=d.doc_id
                 ORDER BY d.effective_ts_ms DESC, d.doc_id ASC
                 LIMIT 5",
            )
            .map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "failed preparing retrieval query",
                    true,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "failed executing retrieval query",
                    true,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        let mut contexts = Vec::new();
        for row in rows {
            let (doc_id, canonical_hash, canonical_object_hash) = row.map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "failed loading retrieval row",
                    true,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
            let bytes = object_store.get_bytes(&kc_core::types::ObjectHash(canonical_object_hash))?;
            let text = String::from_utf8(bytes).map_err(|e| {
                AppError::new(
                    "KC_ASK_PROVIDER_UNAVAILABLE",
                    "ask",
                    "canonical text is not utf8",
                    false,
                    serde_json::json!({ "error": e.to_string(), "doc_id": doc_id }),
                )
            })?;
            let end = text.chars().take(240).count() as i64;
            let locator = LocatorV1 {
                v: 1,
                doc_id: kc_core::types::DocId(doc_id),
                canonical_hash: kc_core::types::CanonicalHash(canonical_hash),
                range: kc_core::locator::LocatorRange { start: 0, end },
                hints: None,
            };

            let snippet = resolve_locator_strict(conn, object_store, &locator)?;
            contexts.push(RetrievedContext { locator, snippet });
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
        validate_citations(&citations)?;

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

        let trace_path = write_trace_log(&req.vault_path.join(&self.trace_dir_name), &trace, &citations)?;

        Ok(AskResponse {
            answer_text,
            citations,
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
        let contexts = self.load_contexts(&conn, &object_store)?;

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
                    "doc_id": ctx.locator.doc_id.0.clone(),
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
