use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppError {
    pub schema_version: u32,
    pub code: String,
    pub category: String,
    pub message: String,
    pub retryable: bool,
    pub details: Value,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(code: &str, category: &str, message: &str, retryable: bool, details: Value) -> Self {
        Self {
            schema_version: 1,
            code: code.to_string(),
            category: category.to_string(),
            message: message.to_string(),
            retryable,
            details,
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::new("KC_INTERNAL_ERROR", "internal", message, false, json!({}))
    }
}
