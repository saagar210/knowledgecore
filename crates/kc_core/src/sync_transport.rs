use crate::app_error::{AppError, AppResult};
use crate::sync::SyncHeadV1;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncTargetUri {
    FilePath { path: String },
    S3 { bucket: String, prefix: String },
}

impl SyncTargetUri {
    pub fn parse(raw: &str) -> AppResult<Self> {
        if raw.trim().is_empty() {
            return Err(AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "sync target path is required",
                false,
                serde_json::json!({ "target": raw }),
            ));
        }

        if let Some(rest) = raw.strip_prefix("s3://") {
            let mut parts = rest.splitn(2, '/');
            let bucket = parts.next().unwrap_or_default().trim();
            let prefix = parts.next().unwrap_or_default().trim_matches('/');
            if bucket.is_empty() {
                return Err(AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "s3 target missing bucket",
                    false,
                    serde_json::json!({ "target": raw }),
                ));
            }
            return Ok(SyncTargetUri::S3 {
                bucket: bucket.to_string(),
                prefix: prefix.to_string(),
            });
        }

        if let Some(rest) = raw.strip_prefix("file://") {
            if rest.trim().is_empty() {
                return Err(AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "file target missing path",
                    false,
                    serde_json::json!({ "target": raw }),
                ));
            }
            return Ok(SyncTargetUri::FilePath {
                path: rest.to_string(),
            });
        }

        if raw.contains("://") {
            return Err(AppError::new(
                "KC_SYNC_TARGET_UNSUPPORTED",
                "sync",
                "unsupported sync target scheme",
                false,
                serde_json::json!({ "target": raw }),
            ));
        }

        Ok(SyncTargetUri::FilePath {
            path: raw.to_string(),
        })
    }

    pub fn display(&self) -> String {
        match self {
            SyncTargetUri::FilePath { path } => path.clone(),
            SyncTargetUri::S3 { bucket, prefix } => {
                if prefix.is_empty() {
                    format!("s3://{}", bucket)
                } else {
                    format!("s3://{}/{}", bucket, prefix)
                }
            }
        }
    }
}

pub trait SyncTransport: Send + Sync {
    fn target(&self) -> SyncTargetUri;
    fn read_head(&self) -> AppResult<Option<SyncHeadV1>>;
    fn write_head(&self, head: &SyncHeadV1) -> AppResult<()>;
}

#[derive(Debug, Clone)]
pub struct FsSyncTransport {
    pub root: PathBuf,
}

impl FsSyncTransport {
    pub fn new(path: &Path) -> Self {
        Self {
            root: path.to_path_buf(),
        }
    }
}

impl SyncTransport for FsSyncTransport {
    fn target(&self) -> SyncTargetUri {
        SyncTargetUri::FilePath {
            path: self.root.display().to_string(),
        }
    }

    fn read_head(&self) -> AppResult<Option<SyncHeadV1>> {
        let head_path = self.root.join("head.json");
        if !head_path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&head_path).map_err(|e| {
            AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "failed reading sync head file",
                false,
                serde_json::json!({ "error": e.to_string(), "path": head_path }),
            )
        })?;
        serde_json::from_slice(&bytes).map(Some).map_err(|e| {
            AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "failed parsing sync head file",
                false,
                serde_json::json!({ "error": e.to_string(), "path": head_path }),
            )
        })
    }

    fn write_head(&self, head: &SyncHeadV1) -> AppResult<()> {
        std::fs::create_dir_all(&self.root).map_err(|e| {
            AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "failed creating sync target root",
                false,
                serde_json::json!({ "error": e.to_string(), "path": self.root }),
            )
        })?;
        let bytes =
            crate::canon_json::to_canonical_bytes(&serde_json::to_value(head).map_err(|e| {
                AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "failed serializing sync head",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?)?;
        std::fs::write(self.root.join("head.json"), bytes).map_err(|e| {
            AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "failed writing sync head",
                false,
                serde_json::json!({ "error": e.to_string(), "path": self.root.join("head.json") }),
            )
        })?;
        Ok(())
    }
}
