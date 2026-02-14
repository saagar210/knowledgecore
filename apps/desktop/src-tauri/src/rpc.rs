use kc_ask::{AskRequest, AskService, RetrievedOnlyAskService};
use kc_cli::verifier::verify_bundle;
use kc_core::app_error::AppError;
use kc_core::canonical::load_canonical_text;
use kc_core::db::open_db;
use kc_core::export::{export_bundle, ExportOptions};
use kc_core::hashing::blake3_hex_prefixed;
use kc_core::ingest::ingest_bytes;
use kc_core::locator::{resolve_locator_strict, LocatorV1};
use kc_core::object_store::ObjectStore;
use kc_core::types::DocId;
use kc_core::vault::{vault_open, vault_paths};
use serde::de::Error as DeError;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum RpcResponse<T> {
    Ok { data: T },
    Err { error: AppError },
}

impl<T> RpcResponse<T> {
    pub fn ok(data: T) -> Self {
        Self::Ok { data }
    }

    pub fn err(error: AppError) -> Self {
        Self::Err { error }
    }
}

impl<T: Serialize> Serialize for RpcResponse<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            RpcResponse::Ok { data } => {
                map.serialize_entry("ok", &true)?;
                map.serialize_entry("data", data)?;
            }
            RpcResponse::Err { error } => {
                map.serialize_entry("ok", &false)?;
                map.serialize_entry("error", error)?;
            }
        }
        map.end()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RpcOkWire<T> {
    ok: bool,
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RpcErrWire {
    ok: bool,
    error: AppError,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RpcWire<T> {
    Ok(RpcOkWire<T>),
    Err(RpcErrWire),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for RpcResponse<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match RpcWire::<T>::deserialize(deserializer)? {
            RpcWire::Ok(wire) => {
                if !wire.ok {
                    return Err(D::Error::custom("rpc ok response must set ok=true"));
                }
                Ok(RpcResponse::Ok { data: wire.data })
            }
            RpcWire::Err(wire) => {
                if wire.ok {
                    return Err(D::Error::custom("rpc error response must set ok=false"));
                }
                Ok(RpcResponse::Err { error: wire.error })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct VaultInitReq {
    pub vault_path: String,
    pub vault_slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInitRes {
    pub vault_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VaultOpenReq {
    pub vault_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultOpenRes {
    pub vault_id: String,
    pub vault_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestScanFolderReq {
    pub vault_path: String,
    pub scan_root: String,
    pub source_kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestScanFolderRes {
    pub ingested: i64,
}

#[derive(Debug, Deserialize)]
pub struct IngestInboxOnceReq {
    pub vault_path: String,
    pub file_path: String,
    pub source_kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestInboxOnceRes {
    pub doc_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryReq {
    pub vault_path: String,
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub doc_id: String,
    pub score: f64,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQueryRes {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
pub struct LocatorResolveReq {
    pub vault_path: String,
    pub locator: LocatorV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatorResolveRes {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportBundleReq {
    pub vault_path: String,
    pub export_dir: String,
    pub include_vectors: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBundleRes {
    pub bundle_path: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyBundleReq {
    pub bundle_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyBundleRes {
    pub exit_code: i64,
    pub report: kc_cli::verifier::VerifyReportV1,
}

#[derive(Debug, Deserialize)]
pub struct AskQuestionReq {
    pub vault_path: String,
    pub question: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskQuestionRes {
    pub answer_text: String,
    pub trace_path: String,
}

#[derive(Debug, Deserialize)]
pub struct EventsListReq {
    pub vault_path: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventItem {
    pub event_id: i64,
    pub ts_ms: i64,
    pub event_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventsListRes {
    pub events: Vec<EventItem>,
}

#[derive(Debug, Deserialize)]
pub struct JobsListReq {
    pub vault_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobsListRes {
    pub jobs: Vec<String>,
}

fn mime_for_path(path: &std::path::Path) -> String {
    match path.extension().and_then(|x| x.to_str()).unwrap_or_default() {
        "md" => "text/markdown".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "pdf" => "application/pdf".to_string(),
        "txt" => "text/plain".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

pub fn vault_init_rpc(req: VaultInitReq, now_ms: i64) -> RpcResponse<VaultInitRes> {
    match kc_core::vault::vault_init(std::path::Path::new(&req.vault_path), &req.vault_slug, now_ms) {
        Ok(vault) => RpcResponse::ok(VaultInitRes {
            vault_id: vault.vault_id,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn vault_open_rpc(req: VaultOpenReq) -> RpcResponse<VaultOpenRes> {
    match vault_open(std::path::Path::new(&req.vault_path)) {
        Ok(vault) => RpcResponse::ok(VaultOpenRes {
            vault_id: vault.vault_id,
            vault_slug: vault.vault_slug,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ingest_scan_folder_rpc(req: IngestScanFolderReq, now_ms: i64) -> RpcResponse<IngestScanFolderRes> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(&req.vault_path))?;
        let conn = open_db(&std::path::Path::new(&req.vault_path).join(vault.db.relative_path))?;
        let store = ObjectStore::new(vault_paths(std::path::Path::new(&req.vault_path)).objects_dir);
        let mut ingested = 0i64;
        for entry in walkdir::WalkDir::new(&req.scan_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let bytes = fs::read(path).map_err(|e| {
                AppError::new(
                    "KC_INGEST_FAILED",
                    "ingest",
                    "failed reading scan file",
                    true,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            ingest_bytes(
                &conn,
                &store,
                &bytes,
                &mime_for_path(path),
                &req.source_kind,
                now_ms,
                Some(&path.to_string_lossy()),
                now_ms,
            )?;
            ingested += 1;
        }
        Ok::<_, AppError>(IngestScanFolderRes { ingested })
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ingest_inbox_once_rpc(req: IngestInboxOnceReq, now_ms: i64) -> RpcResponse<IngestInboxOnceRes> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(&req.vault_path))?;
        let conn = open_db(&std::path::Path::new(&req.vault_path).join(vault.db.relative_path))?;
        let store = ObjectStore::new(vault_paths(std::path::Path::new(&req.vault_path)).objects_dir);
        let path = std::path::Path::new(&req.file_path);
        let bytes = fs::read(path).map_err(|e| {
            AppError::new(
                "KC_INGEST_FAILED",
                "ingest",
                "failed reading inbox file",
                true,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;
        let out = ingest_bytes(
            &conn,
            &store,
            &bytes,
            &mime_for_path(path),
            &req.source_kind,
            now_ms,
            Some(&path.to_string_lossy()),
            now_ms,
        )?;
        Ok::<_, AppError>(IngestInboxOnceRes { doc_id: out.doc_id.0 })
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn search_query_rpc(req: SearchQueryReq) -> RpcResponse<SearchQueryRes> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(&req.vault_path))?;
        let conn = open_db(&std::path::Path::new(&req.vault_path).join(vault.db.relative_path))?;
        let store = ObjectStore::new(vault_paths(std::path::Path::new(&req.vault_path)).objects_dir);
        let mut stmt = conn.prepare(
            "SELECT doc_id FROM canonical_text ORDER BY created_event_id DESC, doc_id ASC LIMIT 20",
        ).map_err(|e| AppError::new("KC_RETRIEVAL_FAILED", "search", "failed preparing search query", false, serde_json::json!({ "error": e.to_string() })))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::new("KC_RETRIEVAL_FAILED", "search", "failed running search query", false, serde_json::json!({ "error": e.to_string() })))?;

        let mut hits = Vec::new();
        for row in rows {
            let doc_id = row.map_err(|e| {
                AppError::new(
                    "KC_RETRIEVAL_FAILED",
                    "search",
                    "failed reading search row",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
            let text = String::from_utf8(load_canonical_text(&conn, &store, &DocId(doc_id.clone()))?).unwrap_or_default();
            if text.to_lowercase().contains(&req.query.to_lowercase()) {
                hits.push(SearchHit {
                    doc_id,
                    score: 1.0,
                    snippet: text.chars().take(120).collect(),
                });
            }
        }
        Ok::<_, AppError>(SearchQueryRes { hits })
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn locator_resolve_rpc(req: LocatorResolveReq) -> RpcResponse<LocatorResolveRes> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(&req.vault_path))?;
        let conn = open_db(&std::path::Path::new(&req.vault_path).join(vault.db.relative_path))?;
        let store = ObjectStore::new(vault_paths(std::path::Path::new(&req.vault_path)).objects_dir);
        let text = resolve_locator_strict(&conn, &store, &req.locator)?;
        Ok::<_, AppError>(LocatorResolveRes { text })
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn export_bundle_rpc(req: ExportBundleReq, now_ms: i64) -> RpcResponse<ExportBundleRes> {
    match export_bundle(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.export_dir),
        &ExportOptions {
            include_vectors: req.include_vectors,
        },
        now_ms,
    ) {
        Ok(path) => RpcResponse::ok(ExportBundleRes {
            bundle_path: path.display().to_string(),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn verify_bundle_rpc(req: VerifyBundleReq) -> RpcResponse<VerifyBundleRes> {
    match verify_bundle(std::path::Path::new(&req.bundle_path)) {
        Ok((exit_code, report)) => RpcResponse::ok(VerifyBundleRes { exit_code, report }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ask_question_rpc(req: AskQuestionReq, now_ms: i64) -> RpcResponse<AskQuestionRes> {
    let service = RetrievedOnlyAskService::default();
    match service.ask(AskRequest {
        vault_path: std::path::PathBuf::from(&req.vault_path),
        question: req.question,
        now_ms,
    }) {
        Ok(out) => RpcResponse::ok(AskQuestionRes {
            answer_text: out.answer_text,
            trace_path: out.trace_path.display().to_string(),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn events_list_rpc(req: EventsListReq) -> RpcResponse<EventsListRes> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(&req.vault_path))?;
        let conn = open_db(&std::path::Path::new(&req.vault_path).join(vault.db.relative_path))?;
        let limit = req.limit.unwrap_or(50).max(1);
        let mut stmt = conn
            .prepare("SELECT event_id, ts_ms, type FROM events ORDER BY event_id DESC LIMIT ?1")
            .map_err(|e| {
                AppError::new(
                    "KC_DB_INTEGRITY_FAILED",
                    "events",
                    "failed preparing events query",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok(EventItem {
                    event_id: row.get(0)?,
                    ts_ms: row.get(1)?,
                    event_type: row.get(2)?,
                })
            })
            .map_err(|e| {
                AppError::new(
                    "KC_DB_INTEGRITY_FAILED",
                    "events",
                    "failed querying events",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| {
                AppError::new(
                    "KC_DB_INTEGRITY_FAILED",
                    "events",
                    "failed decoding event row",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?);
        }
        Ok::<_, AppError>(EventsListRes { events })
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn jobs_list_rpc(_req: JobsListReq) -> RpcResponse<JobsListRes> {
    RpcResponse::ok(JobsListRes { jobs: vec![] })
}

pub fn rpc_health_snapshot(vault_path: &str) -> RpcResponse<serde_json::Value> {
    let result = (|| {
        let vault = vault_open(std::path::Path::new(vault_path))?;
        let conn = open_db(&std::path::Path::new(vault_path).join(vault.db.relative_path.clone()))?;
        let db_bytes = fs::read(std::path::Path::new(vault_path).join(vault.db.relative_path)).map_err(|e| {
            AppError::new("KC_RPC_FAILED", "rpc", "failed reading db for health snapshot", false, serde_json::json!({ "error": e.to_string() }))
        })?;
        let event_count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0)).unwrap_or(0);
        Ok::<_, AppError>(serde_json::json!({
            "vaultId": vault.vault_id,
            "dbHash": blake3_hex_prefixed(&db_bytes),
            "eventCount": event_count
        }))
    })();
    match result {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}
