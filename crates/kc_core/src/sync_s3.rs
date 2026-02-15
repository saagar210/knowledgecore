use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use crate::sync::SyncHeadV1;
use crate::sync_transport::{SyncTargetUri, SyncTransport};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct S3SyncTransport {
    pub bucket: String,
    pub prefix: String,
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
    pub emulate_root: Option<PathBuf>,
}

impl S3SyncTransport {
    pub fn new(bucket: String, prefix: String) -> Self {
        Self {
            bucket,
            prefix,
            endpoint_url: std::env::var("KC_SYNC_S3_ENDPOINT").ok(),
            region: std::env::var("KC_SYNC_S3_REGION").ok(),
            emulate_root: std::env::var("KC_SYNC_S3_EMULATE_ROOT")
                .ok()
                .map(PathBuf::from),
        }
    }

    pub fn key_for(&self, leaf: &str) -> String {
        if self.prefix.is_empty() {
            leaf.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_matches('/'), leaf)
        }
    }

    fn emulated_path(&self, leaf: &str) -> Option<PathBuf> {
        self.emulate_root.as_ref().map(|root| {
            let mut p = root.join(&self.bucket);
            if !self.prefix.is_empty() {
                p = p.join(self.prefix.trim_matches('/'));
            }
            p.join(leaf)
        })
    }

    fn classify_remote_error(message: &str) -> (&'static str, bool) {
        let lower = message.to_ascii_lowercase();
        if lower.contains("credential")
            || lower.contains("accessdenied")
            || lower.contains("signature")
            || lower.contains("unauthorized")
            || lower.contains("forbidden")
            || lower.contains("403")
        {
            ("KC_SYNC_AUTH_FAILED", false)
        } else {
            ("KC_SYNC_NETWORK_FAILED", true)
        }
    }

    fn map_remote_error(&self, message: String, operation: &str, key: &str) -> AppError {
        let (code, retryable) = Self::classify_remote_error(&message);
        AppError::new(
            code,
            "sync",
            &format!("failed {operation} s3 object"),
            retryable,
            serde_json::json!({
                "error": message,
                "bucket": self.bucket,
                "key": key
            }),
        )
    }

    fn run_async<T>(&self, fut: impl std::future::Future<Output = AppResult<T>>) -> AppResult<T> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                AppError::new(
                    "KC_SYNC_NETWORK_FAILED",
                    "sync",
                    "failed creating async runtime for s3 sync",
                    true,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        rt.block_on(fut)
    }

    async fn build_client(
        endpoint_url: Option<String>,
        region: Option<String>,
    ) -> AppResult<aws_sdk_s3::Client> {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(region) = region {
            loader = loader.region(aws_sdk_s3::config::Region::new(region));
        }
        if let Some(endpoint) = endpoint_url {
            loader = loader.endpoint_url(endpoint);
        }
        let cfg = loader.load().await;
        Ok(aws_sdk_s3::Client::new(&cfg))
    }

    fn is_not_found(message: &str) -> bool {
        message.contains("NoSuchKey")
            || message.contains("NotFound")
            || message.contains("status: 404")
    }

    pub fn read_bytes(&self, leaf: &str) -> AppResult<Option<Vec<u8>>> {
        if let Some(path) = self.emulated_path(leaf) {
            if !path.exists() {
                return Ok(None);
            }
            let bytes = std::fs::read(&path).map_err(|e| {
                AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "failed reading emulated s3 object",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            return Ok(Some(bytes));
        }

        let bucket = self.bucket.clone();
        let key = self.key_for(leaf);
        let endpoint = self.endpoint_url.clone();
        let region = self.region.clone();
        let this = self.clone();
        self.run_async(async move {
            let client = Self::build_client(endpoint, region).await?;
            let out = client.get_object().bucket(&bucket).key(&key).send().await;
            let out = match out {
                Ok(v) => v,
                Err(e) => {
                    let msg = e.to_string();
                    if Self::is_not_found(&msg) {
                        return Ok(None);
                    }
                    return Err(this.map_remote_error(msg, "reading", &key));
                }
            };
            let bytes = out.body.collect().await.map_err(|e| {
                this.map_remote_error(e.to_string(), "collecting response body for", &key)
            })?;
            Ok(Some(bytes.into_bytes().to_vec()))
        })
    }

    pub fn write_bytes(&self, leaf: &str, bytes: &[u8], content_type: &str) -> AppResult<()> {
        if let Some(path) = self.emulated_path(leaf) {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::new(
                        "KC_SYNC_TARGET_INVALID",
                        "sync",
                        "failed creating emulated s3 parent directory",
                        false,
                        serde_json::json!({ "error": e.to_string(), "path": parent }),
                    )
                })?;
            }
            std::fs::write(&path, bytes).map_err(|e| {
                AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "failed writing emulated s3 object",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            return Ok(());
        }

        let bucket = self.bucket.clone();
        let key = self.key_for(leaf);
        let endpoint = self.endpoint_url.clone();
        let region = self.region.clone();
        let payload = bytes.to_vec();
        let this = self.clone();
        self.run_async(async move {
            let client = Self::build_client(endpoint, region).await?;
            client
                .put_object()
                .bucket(&bucket)
                .key(&key)
                .content_type(content_type)
                .body(payload.into())
                .send()
                .await
                .map_err(|e| this.map_remote_error(e.to_string(), "writing", &key))?;
            Ok(())
        })
    }

    pub fn write_bytes_if_absent(
        &self,
        leaf: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> AppResult<bool> {
        if let Some(path) = self.emulated_path(leaf) {
            if path.exists() {
                return Ok(false);
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::new(
                        "KC_SYNC_TARGET_INVALID",
                        "sync",
                        "failed creating emulated s3 lock parent directory",
                        false,
                        serde_json::json!({ "error": e.to_string(), "path": parent }),
                    )
                })?;
            }
            std::fs::write(&path, bytes).map_err(|e| {
                AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "failed writing emulated s3 lock file",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            return Ok(true);
        }

        let bucket = self.bucket.clone();
        let key = self.key_for(leaf);
        let endpoint = self.endpoint_url.clone();
        let region = self.region.clone();
        let payload = bytes.to_vec();
        let this = self.clone();
        self.run_async(async move {
            let client = Self::build_client(endpoint, region).await?;
            let out = client
                .put_object()
                .bucket(&bucket)
                .key(&key)
                .content_type(content_type)
                .if_none_match("*")
                .body(payload.into())
                .send()
                .await;
            match out {
                Ok(_) => Ok(true),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("PreconditionFailed") || msg.contains("412") {
                        Ok(false)
                    } else {
                        Err(this.map_remote_error(msg, "writing", &key))
                    }
                }
            }
        })
    }

    pub fn delete_key(&self, leaf: &str) -> AppResult<()> {
        if let Some(path) = self.emulated_path(leaf) {
            if path.exists() {
                std::fs::remove_file(&path).map_err(|e| {
                    AppError::new(
                        "KC_SYNC_TARGET_INVALID",
                        "sync",
                        "failed deleting emulated s3 object",
                        false,
                        serde_json::json!({ "error": e.to_string(), "path": path }),
                    )
                })?;
            }
            return Ok(());
        }

        let bucket = self.bucket.clone();
        let key = self.key_for(leaf);
        let endpoint = self.endpoint_url.clone();
        let region = self.region.clone();
        let this = self.clone();
        self.run_async(async move {
            let client = Self::build_client(endpoint, region).await?;
            client
                .delete_object()
                .bucket(&bucket)
                .key(&key)
                .send()
                .await
                .map_err(|e| this.map_remote_error(e.to_string(), "deleting", &key))?;
            Ok(())
        })
    }
}

impl SyncTransport for S3SyncTransport {
    fn target(&self) -> SyncTargetUri {
        SyncTargetUri::S3 {
            bucket: self.bucket.clone(),
            prefix: self.prefix.clone(),
        }
    }

    fn read_head(&self) -> AppResult<Option<SyncHeadV1>> {
        let Some(bytes) = self.read_bytes("head.json")? else {
            return Ok(None);
        };
        serde_json::from_slice::<SyncHeadV1>(&bytes)
            .map(Some)
            .map_err(|e| {
                AppError::new(
                    "KC_SYNC_TARGET_INVALID",
                    "sync",
                    "failed parsing s3 sync head",
                    false,
                    serde_json::json!({
                        "error": e.to_string(),
                        "target": self.target().display(),
                        "key": self.key_for("head.json")
                    }),
                )
            })
    }

    fn write_head(&self, head: &SyncHeadV1) -> AppResult<()> {
        let bytes = to_canonical_bytes(&serde_json::to_value(head).map_err(|e| {
            AppError::new(
                "KC_SYNC_TARGET_INVALID",
                "sync",
                "failed serializing s3 sync head",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?)?;
        self.write_bytes("head.json", &bytes, "application/json")
    }
}
