# plan/01-step-by-step-implementation-plan.md

## Purpose
Execution-ready, fully ordered implementation plan optimized for agentic coding. Each task includes goal, dependencies, files, exact code stubs (signatures/types), commands, done criteria, and failure diagnosis.

## Invariants
- All invariants from AGENTS.md apply.
- CLI parity before UI.
- No business logic in UI or Tauri.
- Tier 1 algorithms and ordering rules implemented only in core.

## Acceptance Tests
- Each milestone gate passes and is reproducible.
- Golden corpus tests exist and remain stable.

---

## PART 1: Bootstrap (Phase 0 + A)

### Task 0.1: Create workspace skeleton
**Goal**
- Create repo structure and minimal crates with compiling stubs.

**Preconditions**
- None.

**Files to create/modify**
- `Cargo.toml` (workspace)
- `crates/kc_core/Cargo.toml`, `src/lib.rs`
- `crates/kc_extract/Cargo.toml`, `src/lib.rs`
- `crates/kc_index/Cargo.toml`, `src/lib.rs`
- `crates/kc_ask/Cargo.toml`, `src/lib.rs`
- `crates/kc_cli/Cargo.toml`, `src/main.rs`
- `apps/desktop/src-tauri/Cargo.toml`, `src/main.rs`
- `apps/desktop/ui/package.json`, `tsconfig.json`, basic app scaffold

**Code stubs (exact)**
`crates/kc_core/src/lib.rs`
```rust
pub mod app_error;
pub mod canon_json;
pub mod hashing;
pub mod types;
pub mod vault;
pub mod db;
pub mod object_store;
pub mod locator;
pub mod chunking;
pub mod export;

pub use app_error::AppError;
crates/kc_core/src/app_error.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub schema_version: u32, // must be 1
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
}
crates/kc_extract/src/lib.rs
pub mod extractor;
pub use extractor::{Extractor, ExtractInput, ExtractOutput, ToolchainIdentity};
crates/kc_index/src/lib.rs
pub mod indexer;
pub mod vector;
pub mod fts;

pub use indexer::{IndexService, LexicalCandidates, VectorCandidates};
crates/kc_ask/src/lib.rs
pub mod ask;
pub mod trace;

pub use ask::{AskService, AskRequest, AskResponse};
crates/kc_cli/src/main.rs
fn main() {
    // placeholder; real CLI wiring added in later tasks
    println!("kc_cli stub");
}
Commands
cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli
Done criteria
Workspace builds and tests run (even if trivial).
Failure modes + diagnosis
Dependency cycles: ensure kc_core has no deps on other kc_* crates.
Missing serde features: add serde and serde_json where needed.
Task A.1: Implement canonical JSON v1 encoder and hashing (kc_core)
Goal
Implement spec/00-canonical-json.md in code with golden vectors tests.
Preconditions
Task 0.1 completed.
Files to create/modify
crates/kc_core/src/canon_json.rs
crates/kc_core/tests/canonical_json.rs
crates/kc_core/tests/canonical_json_vectors.json (fixture)
Code stubs (exact)
crates/kc_core/src/canon_json.rs
use crate::app_error::{AppError, AppResult};
use serde_json::Value;

pub fn to_canonical_bytes(value: &Value) -> AppResult<Vec<u8>>;

pub fn hash_canonical(value: &Value) -> AppResult<String>; // returns "blake3:<hex>"
Commands
cargo test -p kc_core -- canonical_json
Done criteria
Golden vectors test passes.
Float rejection test passes.
Failure diagnosis
Sorting keys: ensure stable lexicographic ordering.
JSON escapes: ensure no whitespace and stable escaping.
Task A.2: Implement hashing primitives and typed IDs (kc_core)
Goal
Implement BLAKE3 hashing and newtypes for DocId/ObjectHash/etc per spec/01.
Preconditions
Task A.1.
Files
crates/kc_core/src/hashing.rs
crates/kc_core/src/types.rs
crates/kc_core/tests/hashing.rs
Code stubs (exact)
crates/kc_core/src/hashing.rs
use crate::app_error::{AppError, AppResult};

pub fn blake3_hex_prefixed(bytes: &[u8]) -> String; // "blake3:<hex>"

pub fn validate_blake3_prefixed(s: &str) -> AppResult<()>;
crates/kc_core/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigHash(pub String);
Commands
cargo test -p kc_core -- hashing
Done criteria
Tests cover format validation and sample digest correctness.
Failure diagnosis
Ensure blake3 crate is included; ensure hex encoding is lowercase.
PART 2: Vault substrate (Phase B)
Task B.1: Implement vault.json schema and vault init/open (kc_core)
Goal
Create vault folder structure, write/read vault.json, validate schema.
Preconditions
Task A.2.
Files
crates/kc_core/src/vault.rs
crates/kc_core/tests/vault.rs
Code stubs (exact)
crates/kc_core/src/vault.rs
use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultJsonV1 {
    pub schema_version: u32,
    pub vault_id: String, // uuid
    pub vault_slug: String,
    pub created_at_ms: i64,
    pub db: VaultDbConfig,
    pub defaults: VaultDefaults,
    pub toolchain: VaultToolchain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDbConfig { pub relative_path: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDefaults {
    pub chunking_config_id: String,
    pub embedding_model_id: String,
    pub recency: VaultRecencyDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRecencyDefaults { pub enabled: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultToolchain {
    pub pdfium: ToolIdentity,
    pub tesseract: ToolIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIdentity { pub identity: String }

pub fn vault_init(vault_path: &Path, vault_slug: &str, now_ms: i64) -> AppResult<VaultJsonV1>;

pub fn vault_open(vault_path: &Path) -> AppResult<VaultJsonV1>;

pub fn vault_paths(vault_path: &Path) -> VaultPaths;

#[derive(Debug, Clone)]
pub struct VaultPaths {
    pub root: PathBuf,
    pub db: PathBuf,
    pub objects_dir: PathBuf,
    pub inbox_dir: PathBuf,
    pub inbox_processed_dir: PathBuf,
    pub vectors_dir: PathBuf,
}
Commands
cargo test -p kc_core -- vault
Done criteria
init creates directories and writes vault.json
open validates schema_version == 1
Failure diagnosis
Path permissions issues: surface as KC_VAULT_INIT_FAILED with details.
Task B.2: Implement SQLite open, migrations, and schema v1 (kc_core)
Goal
Implement migration runner applying SQL files and setting PRAGMA user_version.
Preconditions
Task B.1.
Files
crates/kc_core/src/db.rs
crates/kc_core/migrations/0001_init.sql
crates/kc_core/tests/migrations.rs
Code stubs (exact)
crates/kc_core/src/db.rs
use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;

pub fn open_db(db_path: &Path) -> AppResult<Connection>;

pub fn apply_migrations(conn: &Connection) -> AppResult<()>;

pub fn schema_version(conn: &Connection) -> AppResult<i64>;
Commands
cargo test -p kc_core -- migrations
Done criteria
empty DB migrated to version 1 and tables exist.
Failure diagnosis
SQLite features missing: ensure rusqlite uses bundled sqlite if needed; see packaging spec later.
Task B.3: Implement object store read/write/verify (kc_core)
Goal
Store blobs content-addressed under store/objects/ and record in DB.
Preconditions
Task B.2.
Files
crates/kc_core/src/object_store.rs
crates/kc_core/tests/object_store.rs
Code stubs (exact)
crates/kc_core/src/object_store.rs
use crate::app_error::{AppError, AppResult};
use crate::types::ObjectHash;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub struct ObjectStore {
    pub objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf) -> Self;

    pub fn put_bytes(&self, conn: &Connection, bytes: &[u8], created_event_id: i64) -> AppResult<ObjectHash>;

    pub fn get_bytes(&self, object_hash: &ObjectHash) -> AppResult<Vec<u8>>;

    pub fn exists(&self, object_hash: &ObjectHash) -> AppResult<bool>;
}
Commands
cargo test -p kc_core -- object_store
Done criteria
dedupe works: storing same bytes returns same hash and does not duplicate file.
PART 3: Ingest jobs and events (Phase C)
Task C.1: Implement events append with hash-chain (kc_core)
Goal
Append events with deterministic payload canonical JSON string and hash chain.
Preconditions
Task B.2.
Files
crates/kc_core/src/events.rs (new)
crates/kc_core/src/lib.rs (export module)
crates/kc_core/tests/events.rs
Code stubs (exact)
crates/kc_core/src/events.rs
use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event_id: i64,
    pub ts_ms: i64,
    pub event_type: String,
    pub payload_json: String,
    pub prev_event_hash: Option<String>,
    pub event_hash: String,
}

pub fn append_event(conn: &Connection, ts_ms: i64, event_type: &str, payload: &Value) -> AppResult<EventRecord>;
Commands
cargo test -p kc_core -- events
Done criteria
events insert and chain hashes correctly.
Task C.2: Implement ingest registration (bytes -> objects -> docs) (kc_core)
Goal
Ingest bytes, register doc row and source metadata, store effective_ts_ms.
Preconditions
Tasks B.3 and C.1.
Files
crates/kc_core/src/ingest.rs (new)
crates/kc_core/src/lib.rs (export)
crates/kc_core/tests/ingest.rs
Code stubs (exact)
crates/kc_core/src/ingest.rs
use crate::app_error::{AppError, AppResult};
use crate::types::{DocId, ObjectHash};
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct IngestedDoc {
    pub doc_id: DocId,
    pub original_object_hash: ObjectHash,
    pub bytes: i64,
    pub mime: String,
    pub source_kind: String,
    pub effective_ts_ms: i64,
}

pub fn ingest_bytes(
    conn: &Connection,
    object_store: &crate::object_store::ObjectStore,
    bytes: &[u8],
    mime: &str,
    source_kind: &str,
    effective_ts_ms: i64,
    source_path: Option<&str>,
    now_ms: i64,
) -> AppResult<IngestedDoc>;
Commands
cargo test -p kc_core -- ingest
Done criteria
ingest is idempotent for identical bytes.
doc_sources inserted when provided.
Task C.3: Implement scan job and inbox processed move policy (kc_cli + kc_core)
Goal
Implement CLI commands for scan and inbox ingest behavior using kc_core APIs.
Preconditions
Task C.2.
Files
crates/kc_cli/src/main.rs (replace stub with real CLI)
crates/kc_cli/src/cli.rs (new)
crates/kc_cli/src/commands/ingest.rs (new)
Code stubs (exact)
crates/kc_cli/src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
  Vault { #[command(subcommand)] cmd: VaultCmd },
  Ingest { #[command(subcommand)] cmd: IngestCmd },
}

#[derive(Subcommand)]
pub enum VaultCmd { Init { vault_path: String, vault_slug: String }, Open { vault_path: String } }

#[derive(Subcommand)]
pub enum IngestCmd {
  ScanFolder { vault_path: String, scan_root: String, source_kind: String },
  InboxOnce { vault_path: String, file_path: String, source_kind: String },
}
Commands
cargo test -p kc_cli -p kc_core
Done criteria
CLI can init vault and ingest a file.
Failure diagnosis
MIME detection: use infer crate or file extension mapping; if unknown use application/octet-stream (assumption).
PART 4: Extraction and canonicalization (Phase D)
Task D.1: Define extractor traits in kc_core (no cycles)
Goal
Define interfaces in kc_core so kc_extract can implement them and core orchestration can be reused by CLI and Tauri without duplicating logic.
Preconditions
Task C.2.
Files
crates/kc_core/src/services.rs (new)
crates/kc_core/src/lib.rs (export services)
Code stubs (exact)
crates/kc_core/src/services.rs
use crate::app_error::{AppError, AppResult};
use crate::types::{CanonicalHash, DocId, ObjectHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainIdentity {
    pub pdfium_identity: String,
    pub tesseract_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTextArtifact {
    pub doc_id: DocId,
    pub canonical_bytes: Vec<u8>,
    pub canonical_hash: CanonicalHash,
    pub canonical_object_hash: ObjectHash,
    pub extractor_name: String,
    pub extractor_version: String,
    pub extractor_flags_json: String, // canonical JSON string
    pub normalization_version: i64,
    pub toolchain_json: String, // canonical JSON string
}

#[derive(Debug, Clone)]
pub struct ExtractInput<'a> {
    pub doc_id: &'a DocId,
    pub bytes: &'a [u8],
    pub mime: &'a str,
    pub source_kind: &'a str,
}

pub trait ExtractService: Send + Sync {
    fn extract_canonical(&self, input: ExtractInput) -> AppResult<CanonicalTextArtifact>;
}
Commands
cargo test -p kc_core
Done criteria
kc_core compiles without depending on kc_extract.
Task D.2: Implement MD and HTML canonicalization (kc_extract)
Goal
Produce canonical text with heading markers and normalization v1.
Preconditions
Task D.1.
Files
crates/kc_extract/src/extractor.rs (implement ExtractService)
crates/kc_extract/src/md.rs, html.rs, normalize.rs, markers.rs
crates/kc_extract/tests/golden_md_html.rs
Code stubs (exact)
crates/kc_extract/src/extractor.rs
use kc_core::app_error::{AppError, AppResult};
use kc_core::services::{CanonicalTextArtifact, ExtractInput, ExtractService};
use kc_core::types::{CanonicalHash, ObjectHash};
use kc_core::hashing::blake3_hex_prefixed;

pub struct DefaultExtractor {
    pub toolchain: kc_core::services::ToolchainIdentity,
}

impl DefaultExtractor {
    pub fn new(toolchain: kc_core::services::ToolchainIdentity) -> Self { Self { toolchain } }
}

impl ExtractService for DefaultExtractor {
    fn extract_canonical(&self, input: ExtractInput) -> AppResult<CanonicalTextArtifact> {
        // dispatch by mime; call md/html/pdf handlers
        todo!()
    }
}
Commands
cargo test -p kc_extract
Done criteria
Golden MD/HTML tests pass and include heading marker lines.
Task D.3: Implement PDFium extraction + OCR trigger + Tesseract pipeline (kc_extract)
Goal
Extract canonical text from PDF, add [[PAGE:NNNN]] markers, and run OCR when quality threshold triggers.
Preconditions
Task D.2.
Files
crates/kc_extract/src/pdf.rs
crates/kc_extract/src/ocr.rs
crates/kc_extract/tests/golden_pdf.rs
Code stubs (exact)
crates/kc_extract/src/pdf.rs
use kc_core::app_error::{AppError, AppResult};

pub struct PdfiumConfig {
    pub library_path: Option<String>, // dev override
}

pub struct PdfExtractOutput {
    pub text_with_page_markers: String,
    pub extracted_len: usize,
    pub extracted_alnum_ratio: f64,
}

pub fn extract_pdf_text(pdf_bytes: &[u8], cfg: &PdfiumConfig) -> AppResult<PdfExtractOutput>;
crates/kc_extract/src/ocr.rs
use kc_core::app_error::{AppError, AppResult};

pub struct OcrConfig {
    pub tesseract_cmd: Option<String>,
    pub language: String, // default "eng"
}

pub fn should_run_ocr(extracted_len: usize, alnum_ratio: f64) -> bool;

pub fn ocr_pdf_via_images(pdf_bytes: &[u8], ocr_cfg: &OcrConfig) -> AppResult<String>;
Commands
cargo test -p kc_extract -- golden_pdf
Done criteria
PDF fixtures produce canonical text with page markers.
Scanned PDF triggers OCR and records toolchain identity.
Task D.4: Persist canonical text artifact (kc_core orchestration)
Goal
Store canonical bytes as object, upsert canonical_text row, enforce canonical_hash invariant.
Preconditions
Tasks B.3 and D.1.
Files
crates/kc_core/src/canonical.rs (new)
crates/kc_core/src/lib.rs (export)
crates/kc_core/tests/canonical_persist.rs
Code stubs (exact)
crates/kc_core/src/canonical.rs
use crate::app_error::{AppError, AppResult};
use crate::services::CanonicalTextArtifact;
use crate::types::{DocId, ObjectHash};
use rusqlite::Connection;

pub fn persist_canonical_text(
    conn: &Connection,
    object_store: &crate::object_store::ObjectStore,
    artifact: &CanonicalTextArtifact,
    created_event_id: i64,
) -> AppResult<()>;

pub fn load_canonical_text(conn: &Connection, object_store: &crate::object_store::ObjectStore, doc_id: &DocId) -> AppResult<Vec<u8>>;
Commands
cargo test -p kc_core -- canonical_persist
Done criteria
canonical_text row exists and canonical_hash equals object_hash.
PART 5: Chunking + Indexing + Retrieval (Phase E/F)
Task E.1: Implement chunking engine and chunk persistence (kc_core)
Goal
Produce deterministic chunks and insert into DB.
Preconditions
Task D.4.
Files
crates/kc_core/src/chunking.rs (implement from spec/06)
crates/kc_core/tests/chunking_golden.rs
Code stubs (exact)
crates/kc_core/src/chunking.rs
use crate::app_error::{AppError, AppResult};
use crate::types::{ChunkId, ConfigHash, DocId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfigV1 {
    pub v: i64,
    pub md_html: MdHtmlChunkCfg,
    pub pdf: PdfChunkCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdHtmlChunkCfg { pub max_chars: usize, pub min_chars: usize }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfChunkCfg { pub window_chars: usize, pub overlap_chars: usize, pub respect_markers: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub ordinal: i64,
    pub start_char: i64,
    pub end_char: i64,
    pub chunking_config_hash: ConfigHash,
}

pub fn hash_chunking_config(cfg: &ChunkingConfigV1) -> AppResult<ConfigHash>;

pub fn chunk_document(doc_id: &DocId, canonical_text: &str, mime: &str, cfg: &ChunkingConfigV1) -> AppResult<Vec<ChunkRecord>>;
Commands
cargo test -p kc_core -- chunking_golden
Done criteria
chunk lists match golden fixtures.
Task E.2: Define index service traits in kc_core and implement FTS5 in kc_index
Goal
Provide indexing interfaces and implement FTS rebuild/query in kc_index.
Preconditions
Task E.1.
Files
crates/kc_core/src/index_traits.rs (new)
crates/kc_index/src/fts.rs
crates/kc_index/tests/fts.rs
Code stubs (exact)
crates/kc_core/src/index_traits.rs
use crate::app_error::{AppError, AppResult};
use crate::types::{ChunkId, DocId};

#[derive(Debug, Clone)]
pub struct LexicalCandidate { pub chunk_id: ChunkId, pub rank: i64 }

#[derive(Debug, Clone)]
pub struct VectorCandidate { pub chunk_id: ChunkId, pub rank: i64 }

pub trait LexicalIndex: Send + Sync {
    fn rebuild_for_doc(&self, doc_id: &DocId) -> AppResult<()>;
    fn query(&self, query: &str, limit: usize) -> AppResult<Vec<LexicalCandidate>>;
}

pub trait VectorIndex: Send + Sync {
    fn rebuild_for_doc(&self, doc_id: &DocId) -> AppResult<()>;
    fn query(&self, query: &str, limit: usize) -> AppResult<Vec<VectorCandidate>>;
}
Commands
cargo test -p kc_index
Done criteria
FTS table created and query returns candidates.
Task F.1: Implement LanceDB vector index + embedding provider identity (kc_index)
Goal
Implement vector index schema, embedding provider interface, and query method returning ranked candidates.
Preconditions
Task E.2.
Files
crates/kc_index/src/vector.rs
crates/kc_index/src/embedding.rs (new)
crates/kc_index/tests/vector.rs
Code stubs (exact)
crates/kc_index/src/embedding.rs
use kc_core::app_error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct EmbeddingIdentity {
    pub model_id: String,
    pub model_hash: String,
    pub dims: usize,
    pub provider: String,
    pub provider_version: String,
    pub flags_json: String,
}

pub trait Embedder: Send + Sync {
    fn identity(&self) -> EmbeddingIdentity;
    fn embed(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>>;
}
Commands
cargo test -p kc_index -- vector
Done criteria
vector index rebuild and query works for fixtures.
Task F.2: Implement hybrid merge scoring and ordering (kc_core)
Goal
Implement RRF merge and deterministic ordering per spec/09.
Preconditions
Task E.2 and F.1.
Files
crates/kc_core/src/retrieval.rs (new)
crates/kc_core/tests/retrieval_ordering.rs
Code stubs (exact)
crates/kc_core/src/retrieval.rs
use crate::app_error::{AppError, AppResult};
use crate::index_traits::{LexicalCandidate, VectorCandidate};
use crate::types::{ChunkId, DocId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfigV1 {
    pub rrf_k: i64,
    pub w_lex: f64,
    pub w_vec: f64,
    pub recency: RecencyConfigV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecencyConfigV1 {
    pub enabled: bool,
    pub window_days: i64,
    pub max_boost: f64,
}

#[derive(Debug, Clone)]
pub struct MergedHit {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub ordinal: i64,
    pub final_score: f64,
}

pub fn merge_candidates(
    lexical: &[LexicalCandidate],
    vector: &[VectorCandidate],
    meta_lookup: impl Fn(&ChunkId) -> AppResult<(DocId, i64, String, i64)>, // doc_id, ordinal, source_kind, effective_ts_ms
    cfg: &RetrievalConfigV1,
    now_ms: i64,
) -> AppResult<Vec<MergedHit>>;
Commands
cargo test -p kc_core -- retrieval_ordering
Done criteria
deterministic ordering matches snapshot for fixed now_ms.
PART 6: Locators, export, verify, ask, then UI (Phase G–K)
Due to output size limits, subsequent tasks are continued in PART 2 of this file, including:
locator resolver implementation,
export manifest generation,
verifier CLI stable exit codes,
ask service and trace logs,
tauri RPC wiring,
React UI scaffolding and feature implementation,
packaging and performance harness tasks.
(See PART 2 below.)
PART 2: Phase G–K tasks (continued)
Task G.1: Implement locator resolver and snippet renderer (kc_core)
Goal
Implement locator schema validation, strict resolver, and display-only snippet rendering.
Preconditions
Canonical persistence and chunking completed (Tasks D.4, E.1).
Files
crates/kc_core/src/locator.rs (implement)
crates/kc_core/src/snippet.rs (new)
crates/kc_core/tests/locator.rs
Code stubs (exact)
crates/kc_core/src/locator.rs
use crate::app_error::{AppError, AppResult};
use crate::types::{CanonicalHash, DocId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorV1 {
    pub v: i64,
    pub doc_id: DocId,
    pub canonical_hash: CanonicalHash,
    pub range: LocatorRange,
    pub hints: Option<LocatorHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorRange { pub start: i64, pub end: i64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorHints {
    pub kind: Option<String>,
    pub pages: Option<PageRange>,
    pub heading_path: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRange { pub start: i64, pub end: i64 }

pub fn resolve_locator_strict(
    conn: &rusqlite::Connection,
    object_store: &crate::object_store::ObjectStore,
    locator: &LocatorV1,
) -> AppResult<String>;
crates/kc_core/src/snippet.rs
use crate::app_error::{AppError, AppResult};

pub fn render_snippet_display_only(text: &str) -> AppResult<String>;
Commands
cargo test -p kc_core -- locator
Done criteria
strict failures return correct AppError codes.
marker stripping affects display only.
Task H.1: Implement export bundle writer (kc_core) and CLI command (kc_cli)
Goal
Write deterministic folder bundle and manifest.json per spec/12.
Preconditions
Locator, chunking, canonical registry, object store complete.
Files
crates/kc_core/src/export.rs (implement)
crates/kc_cli/src/commands/export.rs (new)
crates/kc_core/tests/export_manifest.rs
Code stubs (exact)
crates/kc_core/src/export.rs
use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_vectors: bool,
}

pub fn export_bundle(vault_path: &Path, export_dir: &Path, opts: &ExportOptions, now_ms: i64) -> AppResult<std::path::PathBuf>;
Commands
cargo test -p kc_core -p kc_cli -- export_manifest
Done criteria
manifest ordering deterministic; db hash computed.
Task H.2: Implement verifier library + CLI stable exit codes (kc_cli)
Goal
Implement verifier per spec/13, output deterministic JSON report.
Preconditions
Task H.1.
Files
crates/kc_cli/src/commands/verify.rs (new)
crates/kc_cli/src/verifier.rs (new)
crates/kc_cli/tests/verifier.rs
Code stubs (exact)
crates/kc_cli/src/verifier.rs
use kc_core::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReportV1 {
    pub report_version: i64,
    pub status: String,
    pub exit_code: i64,
    pub errors: Vec<VerifyErrorEntry>,
    pub checked: CheckedCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyErrorEntry {
    pub code: String,
    pub path: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedCounts { pub objects: i64, pub indexes: i64 }

pub fn verify_bundle(bundle_path: &Path) -> AppResult<(i64, VerifyReportV1)>;
Commands
cargo test -p kc_cli
Done criteria
exit codes match spec; report ordering stable.
Task I.1: Implement Ask service + trace writer (kc_ask) and CLI (kc_cli)
Goal
Implement retrieved-only ask pipeline and trace log schema.
Preconditions
Retrieval merge and locator strict resolver exist (Tasks F.2, G.1).
Files
crates/kc_ask/src/ask.rs
crates/kc_ask/src/trace.rs
crates/kc_ask/tests/ask.rs
Code stubs (exact)
crates/kc_ask/src/ask.rs
use kc_core::app_error::{AppError, AppResult};
use kc_core::locator::LocatorV1;

#[derive(Debug, Clone)]
pub struct AskRequest {
    pub vault_path: std::path::PathBuf,
    pub question: String,
    pub now_ms: i64,
}

#[derive(Debug, Clone)]
pub struct AskResponse {
    pub answer_text: String,
    pub citations: Vec<(i64, Vec<LocatorV1>)>, // paragraph_index -> locators
    pub trace_path: std::path::PathBuf,
}

pub trait AskService: Send + Sync {
    fn ask(&self, req: AskRequest) -> AppResult<AskResponse>;
}
Commands
cargo test -p kc_ask -p kc_core
Done criteria
missing/invalid citations hard-fail with correct codes.
Task J.1: Implement Tauri RPC wiring (thin) (apps/desktop/src-tauri)
Goal
Implement RPC handlers that call core services and return envelopes.
Preconditions
CLI parity achieved for core features.
Files
apps/desktop/src-tauri/src/rpc.rs (new)
apps/desktop/src-tauri/src/main.rs (wire commands)
Code stubs (exact)
apps/desktop/src-tauri/src/rpc.rs
use serde::{Deserialize, Serialize};
use kc_core::app_error::AppError;

#[derive(Debug, Serialize)]
#[serde(tag = "ok", rename_all = "lowercase")]
pub enum RpcResponse<T> {
    #[serde(rename_all = "camelCase")]
    True { data: T },
    #[serde(rename_all = "camelCase")]
    False { error: AppError },
}

#[derive(Debug, Deserialize)]
pub struct VaultInitReq { pub vault_path: String, pub vault_slug: String }

#[derive(Debug, Serialize)]
pub struct VaultInitRes { pub vault_id: String }
Commands
pnpm tauri build (after UI scaffold exists)
Done criteria
handlers return correct response envelope and propagate AppError.
Task J.2: Implement full React UI (apps/desktop/ui)
Goal
Implement UI screens per spec/18 using RPC v1.
Preconditions
Task J.1.
Files
UI routes/components, state store, api client.
Code stubs (exact, TS)
apps/desktop/ui/src/api/rpc.ts
export type AppError = {
  schema_version: number;
  code: string;
  category: string;
  message: string;
  retryable: boolean;
  details: any;
};

export type RpcOk<T> = { ok: true; data: T };
export type RpcErr = { ok: false; error: AppError };
export type RpcResp<T> = RpcOk<T> | RpcErr;

export async function rpc<TReq, TRes>(cmd: string, req: TReq): Promise<RpcResp<TRes>> {
  // uses Tauri invoke; no business logic
  throw new Error("stub");
}
Commands
pnpm lint && pnpm test && pnpm tauri build
Done criteria
UI covers full scope and branches only on error.code.
Task K.1: Packaging and ops tooling
Goal
Implement maintenance CLI commands and dependency checks.
Preconditions
Export/verify and ingestion pipeline stable.
Files
kc_cli commands: index rebuild, gc run, vault verify, bench run
Commands
full Rust tests + kc_cli bench run --corpus v1 (smoke)
Done criteria
ops commands exist and are documented; dependency checks produce correct AppError codes.

