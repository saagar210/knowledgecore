use crate::app_error::AppResult;
use crate::index_traits::{LexicalCandidate, VectorCandidate};
use crate::types::{ChunkId, DocId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
struct Interim {
    chunk_id: ChunkId,
    doc_id: DocId,
    ordinal: i64,
    source_kind: String,
    effective_ts_ms: i64,
    score: f64,
}

fn source_prior(source_kind: &str) -> f64 {
    let raw: f64 = match source_kind {
        "manuals" => 1.10,
        "confluence_exports" => 1.07,
        "notes" => 1.05,
        "evidence_packs" => 1.08,
        "inbox" => 0.98,
        _ => 1.00,
    };
    raw.clamp(0.90, 1.15)
}

fn recency_boost(now_ms: i64, ts_ms: i64, cfg: &RecencyConfigV1) -> f64 {
    if !cfg.enabled || cfg.window_days <= 0 || now_ms < ts_ms {
        return 0.0;
    }
    let age_ms = now_ms - ts_ms;
    let window_ms = cfg.window_days * 24 * 60 * 60 * 1000;
    if age_ms >= window_ms {
        0.0
    } else {
        let factor = 1.0 - (age_ms as f64 / window_ms as f64);
        (cfg.max_boost * factor).clamp(0.0, cfg.max_boost)
    }
}

fn round12(value: f64) -> f64 {
    (value * 1_000_000_000_000.0).round() / 1_000_000_000_000.0
}

pub fn merge_candidates(
    lexical: &[LexicalCandidate],
    vector: &[VectorCandidate],
    meta_lookup: impl Fn(&ChunkId) -> AppResult<(DocId, i64, String, i64)>,
    cfg: &RetrievalConfigV1,
    now_ms: i64,
) -> AppResult<Vec<MergedHit>> {
    let mut by_chunk: HashMap<String, Interim> = HashMap::new();

    for c in lexical {
        let (doc_id, ordinal, source_kind, effective_ts_ms) = meta_lookup(&c.chunk_id)?;
        let rrf = cfg.w_lex * (1.0 / (cfg.rrf_k as f64 + c.rank as f64));

        by_chunk
            .entry(c.chunk_id.0.clone())
            .and_modify(|s| s.score += rrf)
            .or_insert(Interim {
                chunk_id: c.chunk_id.clone(),
                doc_id,
                ordinal,
                source_kind,
                effective_ts_ms,
                score: rrf,
            });
    }

    for c in vector {
        let (doc_id, ordinal, source_kind, effective_ts_ms) = meta_lookup(&c.chunk_id)?;
        let rrf = cfg.w_vec * (1.0 / (cfg.rrf_k as f64 + c.rank as f64));

        by_chunk
            .entry(c.chunk_id.0.clone())
            .and_modify(|s| s.score += rrf)
            .or_insert(Interim {
                chunk_id: c.chunk_id.clone(),
                doc_id,
                ordinal,
                source_kind,
                effective_ts_ms,
                score: rrf,
            });
    }

    let mut hits: Vec<MergedHit> = by_chunk
        .into_values()
        .map(|it| {
            let prior = source_prior(&it.source_kind);
            let boost = recency_boost(now_ms, it.effective_ts_ms, &cfg.recency);
            let final_score = round12(it.score * prior * (1.0 + boost));
            MergedHit {
                chunk_id: it.chunk_id,
                doc_id: it.doc_id,
                ordinal: it.ordinal,
                final_score,
            }
        })
        .collect();

    hits.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.doc_id.0.cmp(&b.doc_id.0))
            .then(a.ordinal.cmp(&b.ordinal))
            .then(a.chunk_id.0.cmp(&b.chunk_id.0))
    });

    Ok(hits)
}
