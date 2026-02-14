use kc_core::app_error::{AppError, AppResult};
use kc_core::canon_json::hash_canonical;
use kc_core::chunking::{chunk_document, default_chunking_config_v1};
use kc_core::types::DocId;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchBaselineV1 {
    corpus: String,
    elapsed_ms: u128,
    checksum: u64,
}

fn checksum_str(accum: u64, value: &str) -> u64 {
    value.bytes().fold(accum, |state, b| {
        state
            .wrapping_mul(0x100000001b3)
            .wrapping_add(b as u64)
    })
}

fn workload_v1(iterations: usize) -> AppResult<u64> {
    let docs: [(&str, &str, &str); 4] = [
        (
            "doc-md-1",
            "text/markdown",
            "# Incident 2026-01-03\n\nRoot cause was dependency mismatch.\nMitigation landed in patch 3.\n",
        ),
        (
            "doc-md-2",
            "text/markdown",
            "## Runbook\n\n1. Verify bundle\n2. Rebuild index\n3. Ask with strict citations\n",
        ),
        (
            "doc-html-1",
            "text/html",
            "<h1>Deployment Notes</h1><p>Canary passed in 12 minutes.</p><p>No data loss.</p>",
        ),
        (
            "doc-pdf-like",
            "application/pdf",
            "[[PAGE:0001]]\nVendor statement and OCR fallback notes.\n[[PAGE:0002]]\nAppendix with evidence.",
        ),
    ];

    let cfg = default_chunking_config_v1();
    let mut checksum = 0u64;
    for i in 0..iterations {
        for (doc_key, mime, text) in docs {
            let doc_id = DocId(format!("kc-bench-{}-{}", doc_key, i));
            let chunks = chunk_document(&doc_id, text, mime, &cfg)?;
            checksum = checksum_str(checksum, &doc_id.0);
            checksum = checksum_str(checksum, mime);
            checksum = checksum.wrapping_add(chunks.len() as u64);

            for chunk in chunks {
                checksum = checksum_str(checksum, &chunk.chunk_id.0);
                checksum = checksum.wrapping_add(chunk.start_char as u64);
                checksum = checksum.wrapping_add(chunk.end_char as u64);
            }

            let value = serde_json::json!({
              "doc_id": doc_id.0,
              "mime": mime,
              "chars": text.chars().count(),
              "iter": i,
            });
            let canonical_hash = hash_canonical(&value)?;
            checksum = checksum_str(checksum, &canonical_hash);
        }
    }
    Ok(checksum)
}

fn baseline_path(corpus: &str) -> PathBuf {
    PathBuf::from(".bench").join(format!("baseline-{}.json", corpus))
}

pub fn run_bench(corpus: &str) -> AppResult<()> {
    if corpus != "v1" {
        return Err(AppError::new(
            "KC_INTERNAL_ERROR",
            "bench",
            "only corpus v1 is currently supported",
            false,
            serde_json::json!({ "corpus": corpus }),
        ));
    }

    let start = Instant::now();
    let accum = workload_v1(60)?;
    let elapsed = start.elapsed().as_millis();

    let path = baseline_path(corpus);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::new(
                "KC_INTERNAL_ERROR",
                "bench",
                "failed creating bench baseline directory",
                false,
                serde_json::json!({ "error": e.to_string(), "path": parent }),
            )
        })?;
    }

    if path.exists() {
        let baseline: BenchBaselineV1 = serde_json::from_slice(&fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_INTERNAL_ERROR",
                "bench",
                "failed reading bench baseline",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?)
        .map_err(|e| {
            AppError::new(
                "KC_INTERNAL_ERROR",
                "bench",
                "failed parsing bench baseline",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;

        if baseline.checksum != accum {
            let refreshed = BenchBaselineV1 {
                corpus: corpus.to_string(),
                elapsed_ms: elapsed,
                checksum: accum,
            };
            fs::write(
                &path,
                serde_json::to_vec_pretty(&refreshed).map_err(|e| {
                    AppError::new(
                        "KC_INTERNAL_ERROR",
                        "bench",
                        "failed serializing refreshed bench baseline",
                        false,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?,
            )
            .map_err(|e| {
                AppError::new(
                    "KC_INTERNAL_ERROR",
                    "bench",
                    "failed writing refreshed bench baseline",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            println!(
                "bench baseline refreshed due workload checksum change: corpus={} elapsed_ms={} checksum={}",
                corpus, elapsed, accum
            );
            return Ok(());
        }

        let threshold = baseline.elapsed_ms.saturating_mul(3);
        if elapsed > threshold {
            return Err(AppError::new(
                "KC_INTERNAL_ERROR",
                "bench",
                "performance regression exceeds threshold",
                false,
                serde_json::json!({
                    "elapsed_ms": elapsed,
                    "baseline_ms": baseline.elapsed_ms,
                    "threshold_ms": threshold,
                    "corpus": corpus
                }),
            ));
        }
        println!(
            "bench run complete: corpus={} elapsed_ms={} baseline_ms={} checksum={}",
            corpus, elapsed, baseline.elapsed_ms, accum
        );
        return Ok(());
    }

    let baseline = BenchBaselineV1 {
        corpus: corpus.to_string(),
        elapsed_ms: elapsed,
        checksum: accum,
    };
    fs::write(
        &path,
        serde_json::to_vec_pretty(&baseline).map_err(|e| {
            AppError::new(
                "KC_INTERNAL_ERROR",
                "bench",
                "failed serializing bench baseline",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?,
    )
    .map_err(|e| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "bench",
            "failed writing bench baseline",
            false,
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })?;

    println!(
        "bench baseline created: corpus={} elapsed_ms={} checksum={}",
        corpus, elapsed, accum
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{baseline_path, workload_v1};

    #[test]
    fn baseline_path_is_stable() {
        assert_eq!(
            baseline_path("v1").to_string_lossy(),
            ".bench/baseline-v1.json"
        );
    }

    #[test]
    fn workload_v1_is_deterministic() {
        let a = workload_v1(4).expect("workload run");
        let b = workload_v1(4).expect("workload run");
        assert_eq!(a, b);
    }
}
