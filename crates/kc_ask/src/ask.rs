use kc_core::app_error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AskRequest {
    pub vault_path: std::path::PathBuf,
    pub question: String,
    pub now_ms: i64,
}

#[derive(Debug, Clone)]
pub struct AskResponse {
    pub answer_text: String,
}

pub trait AskService: Send + Sync {
    fn ask(&self, req: AskRequest) -> AppResult<AskResponse>;
}

pub struct UnavailableAsk;

impl AskService for UnavailableAsk {
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
