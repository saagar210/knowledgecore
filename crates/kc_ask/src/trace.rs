use kc_core::app_error::{AppError, AppResult};
use kc_core::locator::LocatorV1;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLogV1 {
    pub schema_version: i64,
    pub trace_id: String,
    pub ts_ms: i64,
    pub vault_id: String,
    pub question: String,
    pub retrieval: serde_json::Value,
    pub model: serde_json::Value,
    pub answer: serde_json::Value,
    pub redaction: serde_json::Value,
}

pub fn write_trace_log(
    trace_dir: &Path,
    trace: &TraceLogV1,
    citations: &[(i64, Vec<LocatorV1>)],
) -> AppResult<PathBuf> {
    fs::create_dir_all(trace_dir).map_err(|e| {
        AppError::new(
            "KC_TRACE_WRITE_FAILED",
            "trace",
            "failed to create trace directory",
            false,
            serde_json::json!({ "error": e.to_string(), "path": trace_dir }),
        )
    })?;

    let mut sorted: Vec<(i64, Vec<LocatorV1>)> = citations
        .iter()
        .map(|(paragraph_idx, locators)| {
            let mut locators_sorted = locators.clone();
            locators_sorted.sort_by(|a, b| {
                a.doc_id
                    .0
                    .cmp(&b.doc_id.0)
                    .then(a.range.start.cmp(&b.range.start))
                    .then(a.range.end.cmp(&b.range.end))
            });
            (*paragraph_idx, locators_sorted)
        })
        .collect();
    sorted.sort_by(|a, b| {
        let la = a.1.first();
        let lb = b.1.first();
        a.0.cmp(&b.0).then(
            la.map(|x| (&x.doc_id.0, x.range.start, x.range.end))
                .cmp(&lb.map(|x| (&x.doc_id.0, x.range.start, x.range.end))),
        )
    });

    let mut value = serde_json::to_value(trace).map_err(|e| {
        AppError::new(
            "KC_TRACE_WRITE_FAILED",
            "trace",
            "failed to serialize trace log",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    value["retrieval"]["citations"] = serde_json::to_value(sorted).map_err(|e| {
        AppError::new(
            "KC_TRACE_WRITE_FAILED",
            "trace",
            "failed to serialize citations",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let path = trace_dir.join(format!("{}.json", trace.trace_id));
    fs::write(
        &path,
        serde_json::to_vec_pretty(&value).map_err(|e| {
            AppError::new(
                "KC_TRACE_WRITE_FAILED",
                "trace",
                "failed to serialize trace log JSON",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?,
    )
    .map_err(|e| {
        AppError::new(
            "KC_TRACE_WRITE_FAILED",
            "trace",
            "failed to write trace log",
            false,
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })?;

    Ok(path)
}
