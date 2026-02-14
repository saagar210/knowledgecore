use crate::app_error::AppError;
use serde::{Deserialize, Serialize};

pub const PREVIEW_ERROR_CODE: &str = "KC_DRAFT_LINEAGE_NOT_IMPLEMENTED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageNodeDraftV1 {
    pub node_id: String,
    pub kind: String,
    pub label: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageEdgeDraftV1 {
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageQueryResDraftV1 {
    pub schema_version: i64,
    pub status: String,
    pub activation_phase: String,
    pub nodes: Vec<LineageNodeDraftV1>,
    pub edges: Vec<LineageEdgeDraftV1>,
}

pub fn placeholder_response() -> LineageQueryResDraftV1 {
    LineageQueryResDraftV1 {
        schema_version: 1,
        status: "draft".to_string(),
        activation_phase: "N3".to_string(),
        nodes: vec![
            LineageNodeDraftV1 {
                node_id: "chunk:1".to_string(),
                kind: "chunk".to_string(),
                label: "Chunk 1".to_string(),
                metadata: serde_json::json!({}),
            },
            LineageNodeDraftV1 {
                node_id: "doc:1".to_string(),
                kind: "doc".to_string(),
                label: "Doc 1".to_string(),
                metadata: serde_json::json!({}),
            },
        ],
        edges: vec![LineageEdgeDraftV1 {
            from_node_id: "doc:1".to_string(),
            to_node_id: "chunk:1".to_string(),
            relation: "contains".to_string(),
            evidence: "draft".to_string(),
        }],
    }
}

pub fn not_implemented_error() -> AppError {
    AppError::new(
        PREVIEW_ERROR_CODE,
        "draft",
        "advanced lineage UI is not implemented in Phase L",
        false,
        serde_json::json!({
            "activation_phase": "N3",
            "schema_status": "draft",
            "spec": "spec/25-advanced-lineage-ui-v1-design-lock.md"
        }),
    )
}
