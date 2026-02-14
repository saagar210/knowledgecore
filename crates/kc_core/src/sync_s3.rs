use crate::app_error::{AppError, AppResult};
use crate::sync::SyncHeadV1;
use crate::sync_transport::{SyncTargetUri, SyncTransport};

#[derive(Debug, Clone)]
pub struct S3SyncTransport {
    pub bucket: String,
    pub prefix: String,
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
}

impl S3SyncTransport {
    pub fn new(bucket: String, prefix: String) -> Self {
        Self {
            bucket,
            prefix,
            endpoint_url: std::env::var("KC_SYNC_S3_ENDPOINT").ok(),
            region: std::env::var("KC_SYNC_S3_REGION").ok(),
        }
    }

    fn key_for(&self, leaf: &str) -> String {
        if self.prefix.is_empty() {
            leaf.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_matches('/'), leaf)
        }
    }

    fn run_async<T>(
        &self,
        fut: impl std::future::Future<Output = AppResult<T>>,
    ) -> AppResult<T> {
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

    async fn client(&self) -> AppResult<aws_sdk_s3::Client> {
        let mut loader =
            aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(region) = &self.region {
            loader = loader.region(aws_sdk_s3::config::Region::new(region.clone()));
        }
        if let Some(endpoint) = &self.endpoint_url {
            loader = loader.endpoint_url(endpoint);
        }
        let cfg = loader.load().await;
        Ok(aws_sdk_s3::Client::new(&cfg))
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
        self.run_async(async move {
            let client = self.client().await?;
            let key = self.key_for("head.json");
            let out = client
                .get_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await;
            let out = match out {
                Ok(v) => v,
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("NoSuchKey") || msg.contains("NotFound") {
                        return Ok(None);
                    }
                    return Err(AppError::new(
                        "KC_SYNC_NETWORK_FAILED",
                        "sync",
                        "failed reading s3 sync head",
                        true,
                        serde_json::json!({
                            "error": msg,
                            "bucket": self.bucket,
                            "key": key
                        }),
                    ));
                }
            };
            let bytes = out.body.collect().await.map_err(|e| {
                AppError::new(
                    "KC_SYNC_NETWORK_FAILED",
                    "sync",
                    "failed collecting s3 head body",
                    true,
                    serde_json::json!({
                        "error": e.to_string(),
                        "bucket": self.bucket,
                        "key": key
                    }),
                )
            })?;
            serde_json::from_slice::<SyncHeadV1>(&bytes.into_bytes())
                .map(Some)
                .map_err(|e| {
                    AppError::new(
                        "KC_SYNC_TARGET_INVALID",
                        "sync",
                        "failed parsing s3 sync head",
                        false,
                        serde_json::json!({
                            "error": e.to_string(),
                            "bucket": self.bucket,
                            "key": key
                        }),
                    )
                })
        })
    }

    fn write_head(&self, head: &SyncHeadV1) -> AppResult<()> {
        self.run_async(async move {
            let client = self.client().await?;
            let key = self.key_for("head.json");
            let bytes = crate::canon_json::to_canonical_bytes(
                &serde_json::to_value(head).map_err(|e| {
                    AppError::new(
                        "KC_SYNC_TARGET_INVALID",
                        "sync",
                        "failed serializing sync head",
                        false,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?,
            )?;
            client
                .put_object()
                .bucket(&self.bucket)
                .key(&key)
                .body(bytes.into())
                .content_type("application/json")
                .send()
                .await
                .map_err(|e| {
                    AppError::new(
                        "KC_SYNC_NETWORK_FAILED",
                        "sync",
                        "failed writing s3 sync head",
                        true,
                        serde_json::json!({
                            "error": e.to_string(),
                            "bucket": self.bucket,
                            "key": key
                        }),
                    )
                })?;
            Ok(())
        })
    }
}
