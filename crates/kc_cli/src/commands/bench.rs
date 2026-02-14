use kc_core::app_error::{AppError, AppResult};
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
    let mut accum = 0u64;
    for i in 0..500_000u64 {
        accum = accum.wrapping_add(i ^ 0x9E37);
    }
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
    use super::baseline_path;

    #[test]
    fn baseline_path_is_stable() {
        assert_eq!(
            baseline_path("v1").to_string_lossy(),
            ".bench/baseline-v1.json"
        );
    }
}
