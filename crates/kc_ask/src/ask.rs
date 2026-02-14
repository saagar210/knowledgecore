use crate::trace::{write_trace_log, TraceLogV1};
use kc_core::app_error::{AppError, AppResult};
use kc_core::locator::LocatorV1;
use kc_core::vault::vault_open;

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

pub struct RetrievedOnlyAskService {
    pub trace_dir_name: String,
}

impl Default for RetrievedOnlyAskService {
    fn default() -> Self {
        Self {
            trace_dir_name: "trace".to_string(),
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
    pub fn finalize_answer(
        &self,
        req: &AskRequest,
        answer_text: String,
        citations: Vec<(i64, Vec<LocatorV1>)>,
    ) -> AppResult<AskResponse> {
        validate_citations(&citations)?;

        let vault = vault_open(&req.vault_path)?;
        let trace = TraceLogV1 {
            schema_version: 1,
            trace_id: uuid::Uuid::new_v4().to_string(),
            ts_ms: req.now_ms,
            vault_id: vault.vault_id,
            question: req.question.clone(),
            retrieval: serde_json::json!({}),
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
}

impl AskService for RetrievedOnlyAskService {
    fn ask(&self, _req: AskRequest) -> AppResult<AskResponse> {
        Err(AppError::new(
            "KC_ASK_PROVIDER_UNAVAILABLE",
            "ask",
            "ask provider unavailable",
            true,
            serde_json::json!({}),
        ))
    }
}
