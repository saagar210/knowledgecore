use kc_core::app_error::{AppError, AppResult};
use std::time::Instant;

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
    for i in 0..100_000u64 {
        accum = accum.wrapping_add(i ^ 0x9E37);
    }
    let elapsed = start.elapsed().as_millis();

    println!(
        "bench run complete: corpus=v1 elapsed_ms={} checksum={}",
        elapsed, accum
    );
    Ok(())
}
