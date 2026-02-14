use kc_ask::{AskRequest, AskService, RetrievedOnlyAskService};
use kc_cli::verifier::verify_bundle;
use kc_core::app_error::AppError;
#[cfg(feature = "phase_l_preview")]
use kc_core::deferred;
use kc_core::locator::LocatorV1;
use kc_core::rpc_service;
use serde::de::Error as DeError;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
#[serde(deny_unknown_fields)]
pub struct VaultInitReq {
    pub vault_path: String,
    pub vault_slug: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInitRes {
    pub vault_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultOpenReq {
    pub vault_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultOpenRes {
    pub vault_id: String,
    pub vault_slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultEncryptionStatusReq {
    pub vault_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEncryptionStatusRes {
    pub enabled: bool,
    pub mode: String,
    pub key_reference: Option<String>,
    pub kdf_algorithm: String,
    pub objects_total: i64,
    pub objects_encrypted: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultEncryptionEnableReq {
    pub vault_path: String,
    pub passphrase: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultEncryptionEnableRes {
    pub status: VaultEncryptionStatusRes,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultEncryptionMigrateReq {
    pub vault_path: String,
    pub passphrase: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultEncryptionMigrateRes {
    pub status: VaultEncryptionStatusRes,
    pub migrated_objects: i64,
    pub already_encrypted_objects: i64,
    pub event_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestScanFolderReq {
    pub vault_path: String,
    pub scan_root: String,
    pub source_kind: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestScanFolderRes {
    pub ingested: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestInboxStartReq {
    pub vault_path: String,
    pub file_path: String,
    pub source_kind: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestInboxStartRes {
    pub job_id: String,
    pub doc_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestInboxStopReq {
    pub vault_path: String,
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestInboxStopRes {
    pub stopped: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchQueryReq {
    pub vault_path: String,
    pub query: String,
    pub now_ms: i64,
    pub limit: Option<usize>,
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
#[serde(deny_unknown_fields)]
pub struct LocatorResolveReq {
    pub vault_path: String,
    pub locator: LocatorV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatorResolveRes {
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBundleReq {
    pub vault_path: String,
    pub export_dir: String,
    pub include_vectors: bool,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBundleRes {
    pub bundle_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyBundleReq {
    pub bundle_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyBundleRes {
    pub exit_code: i64,
    pub report: kc_cli::verifier::VerifyReportV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskQuestionReq {
    pub vault_path: String,
    pub question: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskQuestionRes {
    pub answer_text: String,
    pub trace_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct JobsListReq {
    pub vault_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobsListRes {
    pub jobs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncStatusReq {
    pub vault_path: String,
    pub target_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncHeadRes {
    pub schema_version: i64,
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatusRes {
    pub target_path: String,
    pub remote_head: Option<SyncHeadRes>,
    pub seen_remote_snapshot_id: Option<String>,
    pub last_applied_manifest_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncPushReq {
    pub vault_path: String,
    pub target_path: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushRes {
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub remote_head: SyncHeadRes,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncPullReq {
    pub vault_path: String,
    pub target_path: String,
    pub now_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPullRes {
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub remote_head: SyncHeadRes,
}

#[cfg(feature = "phase_l_preview")]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewStatusReq {}

#[cfg(feature = "phase_l_preview")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewStatusRes {
    pub schema_version: i64,
    pub status: String,
    pub capabilities: Vec<deferred::DraftCapabilityStatusV1>,
}

#[cfg(feature = "phase_l_preview")]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewCapabilityReq {
    pub name: String,
}

#[cfg(feature = "phase_l_preview")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewCapabilityRes {
    pub capability: String,
    pub status: String,
}

pub fn vault_init_rpc(req: VaultInitReq) -> RpcResponse<VaultInitRes> {
    match rpc_service::vault_init_service(std::path::Path::new(&req.vault_path), &req.vault_slug, req.now_ms) {
        Ok(vault_id) => RpcResponse::ok(VaultInitRes { vault_id }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn vault_open_rpc(req: VaultOpenReq) -> RpcResponse<VaultOpenRes> {
    match rpc_service::vault_open_service(std::path::Path::new(&req.vault_path)) {
        Ok(vault) => RpcResponse::ok(VaultOpenRes {
            vault_id: vault.vault_id,
            vault_slug: vault.vault_slug,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

fn map_encryption_status(status: kc_core::rpc_service::VaultEncryptionStatus) -> VaultEncryptionStatusRes {
    VaultEncryptionStatusRes {
        enabled: status.enabled,
        mode: status.mode,
        key_reference: status.key_reference,
        kdf_algorithm: status.kdf_algorithm,
        objects_total: status.objects_total,
        objects_encrypted: status.objects_encrypted,
    }
}

pub fn vault_encryption_status_rpc(req: VaultEncryptionStatusReq) -> RpcResponse<VaultEncryptionStatusRes> {
    match rpc_service::vault_encryption_status_service(std::path::Path::new(&req.vault_path)) {
        Ok(status) => RpcResponse::ok(map_encryption_status(status)),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn vault_encryption_enable_rpc(req: VaultEncryptionEnableReq) -> RpcResponse<VaultEncryptionEnableRes> {
    match rpc_service::vault_encryption_enable_service(
        std::path::Path::new(&req.vault_path),
        &req.passphrase,
    ) {
        Ok(status) => RpcResponse::ok(VaultEncryptionEnableRes {
            status: map_encryption_status(status),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn vault_encryption_migrate_rpc(req: VaultEncryptionMigrateReq) -> RpcResponse<VaultEncryptionMigrateRes> {
    match rpc_service::vault_encryption_migrate_service(
        std::path::Path::new(&req.vault_path),
        &req.passphrase,
        req.now_ms,
    ) {
        Ok(out) => RpcResponse::ok(VaultEncryptionMigrateRes {
            status: map_encryption_status(out.status),
            migrated_objects: out.migrated_objects,
            already_encrypted_objects: out.already_encrypted_objects,
            event_id: out.event_id,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ingest_scan_folder_rpc(req: IngestScanFolderReq) -> RpcResponse<IngestScanFolderRes> {
    match rpc_service::ingest_scan_folder_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.scan_root),
        &req.source_kind,
        req.now_ms,
    ) {
        Ok(ingested) => RpcResponse::ok(IngestScanFolderRes { ingested }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ingest_inbox_start_rpc(req: IngestInboxStartReq) -> RpcResponse<IngestInboxStartRes> {
    match rpc_service::ingest_inbox_start_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.file_path),
        &req.source_kind,
        req.now_ms,
    ) {
        Ok(out) => RpcResponse::ok(IngestInboxStartRes {
            job_id: out.job_id,
            doc_id: out.doc_id,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn ingest_inbox_stop_rpc(req: IngestInboxStopReq) -> RpcResponse<IngestInboxStopRes> {
    let _ = req.vault_path;
    match rpc_service::ingest_inbox_stop_service(&req.job_id) {
        Ok(stopped) => RpcResponse::ok(IngestInboxStopRes { stopped }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn search_query_rpc(req: SearchQueryReq) -> RpcResponse<SearchQueryRes> {
    match rpc_service::search_query_service(
        std::path::Path::new(&req.vault_path),
        &req.query,
        req.now_ms,
        req.limit.unwrap_or(20),
    ) {
        Ok(hits) => RpcResponse::ok(SearchQueryRes {
            hits: hits
                .into_iter()
                .map(|h| SearchHit {
                    doc_id: h.doc_id,
                    score: h.score,
                    snippet: h.snippet,
                })
                .collect(),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn locator_resolve_rpc(req: LocatorResolveReq) -> RpcResponse<LocatorResolveRes> {
    match rpc_service::locator_resolve_service(std::path::Path::new(&req.vault_path), &req.locator) {
        Ok(text) => RpcResponse::ok(LocatorResolveRes { text }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn export_bundle_rpc(req: ExportBundleReq) -> RpcResponse<ExportBundleRes> {
    match rpc_service::export_bundle_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.export_dir),
        req.include_vectors,
        req.now_ms,
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

pub fn ask_question_rpc(req: AskQuestionReq) -> RpcResponse<AskQuestionRes> {
    let service = RetrievedOnlyAskService::default();
    match service.ask(AskRequest {
        vault_path: std::path::PathBuf::from(&req.vault_path),
        question: req.question,
        now_ms: req.now_ms,
    }) {
        Ok(out) => RpcResponse::ok(AskQuestionRes {
            answer_text: out.answer_text,
            trace_path: out.trace_path.display().to_string(),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn events_list_rpc(req: EventsListReq) -> RpcResponse<EventsListRes> {
    match rpc_service::events_list_service(std::path::Path::new(&req.vault_path), req.limit.unwrap_or(50)) {
        Ok(events) => RpcResponse::ok(EventsListRes {
            events: events
                .into_iter()
                .map(|e| EventItem {
                    event_id: e.event_id,
                    ts_ms: e.ts_ms,
                    event_type: e.event_type,
                })
                .collect(),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn jobs_list_rpc(req: JobsListReq) -> RpcResponse<JobsListRes> {
    match rpc_service::jobs_list_service(std::path::Path::new(&req.vault_path)) {
        Ok(jobs) => RpcResponse::ok(JobsListRes { jobs }),
        Err(error) => RpcResponse::err(error),
    }
}

fn map_sync_head(head: kc_core::sync::SyncHeadV1) -> SyncHeadRes {
    SyncHeadRes {
        schema_version: head.schema_version,
        snapshot_id: head.snapshot_id,
        manifest_hash: head.manifest_hash,
        created_at_ms: head.created_at_ms,
    }
}

pub fn sync_status_rpc(req: SyncStatusReq) -> RpcResponse<SyncStatusRes> {
    match rpc_service::sync_status_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.target_path),
    ) {
        Ok(status) => RpcResponse::ok(SyncStatusRes {
            target_path: status.target_path,
            remote_head: status.remote_head.map(map_sync_head),
            seen_remote_snapshot_id: status.seen_remote_snapshot_id,
            last_applied_manifest_hash: status.last_applied_manifest_hash,
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn sync_push_rpc(req: SyncPushReq) -> RpcResponse<SyncPushRes> {
    match rpc_service::sync_push_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.target_path),
        req.now_ms,
    ) {
        Ok(out) => RpcResponse::ok(SyncPushRes {
            snapshot_id: out.snapshot_id,
            manifest_hash: out.manifest_hash,
            remote_head: map_sync_head(out.remote_head),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn sync_pull_rpc(req: SyncPullReq) -> RpcResponse<SyncPullRes> {
    match rpc_service::sync_pull_service(
        std::path::Path::new(&req.vault_path),
        std::path::Path::new(&req.target_path),
        req.now_ms,
    ) {
        Ok(out) => RpcResponse::ok(SyncPullRes {
            snapshot_id: out.snapshot_id,
            manifest_hash: out.manifest_hash,
            remote_head: map_sync_head(out.remote_head),
        }),
        Err(error) => RpcResponse::err(error),
    }
}

#[cfg(feature = "phase_l_preview")]
pub fn preview_status_rpc(_req: PreviewStatusReq) -> RpcResponse<PreviewStatusRes> {
    RpcResponse::ok(PreviewStatusRes {
        schema_version: 1,
        status: "draft".to_string(),
        capabilities: deferred::preview_capability_statuses(),
    })
}

#[cfg(feature = "phase_l_preview")]
pub fn preview_capability_rpc(req: PreviewCapabilityReq) -> RpcResponse<PreviewCapabilityRes> {
    RpcResponse::err(deferred::scaffold_error_for_capability(&req.name))
}

#[allow(dead_code)]
pub fn rpc_health_snapshot(vault_path: &str) -> RpcResponse<serde_json::Value> {
    match rpc_service::rpc_health_snapshot_service(std::path::Path::new(vault_path)) {
        Ok(data) => RpcResponse::ok(data),
        Err(error) => RpcResponse::err(error),
    }
}

pub fn phase_l_preview_enabled() -> bool {
    cfg!(feature = "phase_l_preview")
}
