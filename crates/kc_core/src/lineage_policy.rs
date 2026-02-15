use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use crate::hashing::blake3_hex_prefixed;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineagePolicyV3 {
    pub policy_id: String,
    pub policy_name: String,
    pub effect: String,
    pub priority: i64,
    pub condition_json: String,
    pub created_by: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineagePolicyBindingV3 {
    pub subject_id: String,
    pub policy_id: String,
    pub policy_name: String,
    pub effect: String,
    pub priority: i64,
    pub condition_json: String,
    pub bound_by: String,
    pub bound_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineagePolicyDecisionV3 {
    pub subject_id: String,
    pub action: String,
    pub doc_id: Option<String>,
    pub allowed: bool,
    pub reason: String,
    pub matched_policy_id: Option<String>,
    pub matched_policy_name: Option<String>,
    pub matched_effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct LineagePolicyConditionV3 {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    doc_id_prefix: Option<String>,
}

#[derive(Debug, Clone)]
struct EvaluatedPolicy {
    policy_id: String,
    policy_name: String,
    effect: String,
    condition_json: String,
}

fn policy_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, "lineage", message, false, details)
}

fn canonical_json_string(value: &Value) -> AppResult<String> {
    let bytes = to_canonical_bytes(value)?;
    String::from_utf8(bytes).map_err(|e| {
        policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "failed to encode canonical lineage policy condition JSON",
            serde_json::json!({ "error": e.to_string() }),
        )
    })
}

fn normalize_effect(effect: &str) -> AppResult<String> {
    let normalized = effect.trim().to_ascii_lowercase();
    if normalized == "allow" || normalized == "deny" {
        Ok(normalized)
    } else {
        Err(policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "lineage policy effect must be allow or deny",
            serde_json::json!({ "effect": effect, "supported": ["allow", "deny"] }),
        ))
    }
}

fn default_priority(effect: &str) -> i64 {
    if effect == "deny" {
        100
    } else {
        200
    }
}

fn parse_condition(condition_json: &str) -> AppResult<(LineagePolicyConditionV3, String)> {
    let parsed = serde_json::from_str::<Value>(condition_json).map_err(|e| {
        policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "failed parsing lineage policy condition JSON",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let Value::Object(_) = parsed else {
        return Err(policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "lineage policy condition JSON must be an object",
            serde_json::json!({}),
        ));
    };

    let canonical = canonical_json_string(&parsed)?;
    let condition = serde_json::from_str::<LineagePolicyConditionV3>(&canonical).map_err(|e| {
        policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "failed decoding lineage policy condition fields",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    if let Some(action) = &condition.action {
        if action.trim().is_empty() {
            return Err(policy_error(
                "KC_LINEAGE_POLICY_CONDITION_INVALID",
                "lineage policy action condition must not be empty",
                serde_json::json!({ "field": "action" }),
            ));
        }
    }
    if let Some(prefix) = &condition.doc_id_prefix {
        if prefix.trim().is_empty() {
            return Err(policy_error(
                "KC_LINEAGE_POLICY_CONDITION_INVALID",
                "lineage policy doc_id_prefix condition must not be empty",
                serde_json::json!({ "field": "doc_id_prefix" }),
            ));
        }
    }

    Ok((condition, canonical))
}

fn policy_id_for_name(name: &str) -> String {
    blake3_hex_prefixed(format!("kc.lineage.policy.v3\n{}", name.trim()).as_bytes())
}

fn evaluate_condition(
    condition: &LineagePolicyConditionV3,
    action: &str,
    doc_id: Option<&str>,
) -> bool {
    if let Some(expected_action) = &condition.action {
        if expected_action != action {
            return false;
        }
    }
    if let Some(prefix) = &condition.doc_id_prefix {
        let Some(doc) = doc_id else {
            return false;
        };
        if !doc.starts_with(prefix) {
            return false;
        }
    }
    true
}

pub fn lineage_policy_add(
    conn: &Connection,
    policy_name: &str,
    effect: &str,
    condition_json: &str,
    created_by: &str,
    now_ms: i64,
) -> AppResult<LineagePolicyV3> {
    if policy_name.trim().is_empty() || created_by.trim().is_empty() {
        return Err(policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "lineage policy requires non-empty policy_name and created_by",
            serde_json::json!({ "policy_name": policy_name, "created_by": created_by }),
        ));
    }
    let effect_norm = normalize_effect(effect)?;
    let (_condition, canonical_condition_json) = parse_condition(condition_json)?;
    let policy_id = policy_id_for_name(policy_name);
    let priority = default_priority(&effect_norm);

    conn.execute(
        "INSERT INTO lineage_policies(
            policy_id, policy_name, effect, priority, condition_json, created_by, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(policy_name) DO UPDATE SET
           effect=excluded.effect,
           priority=excluded.priority,
           condition_json=excluded.condition_json,
           created_by=excluded.created_by,
           created_at_ms=excluded.created_at_ms",
        params![
            policy_id,
            policy_name,
            effect_norm,
            priority,
            canonical_condition_json,
            created_by,
            now_ms
        ],
    )
    .map_err(|e| {
        policy_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage policy",
            serde_json::json!({ "error": e.to_string(), "policy_name": policy_name }),
        )
    })?;

    conn.query_row(
        "SELECT policy_id, policy_name, effect, priority, condition_json, created_by, created_at_ms
         FROM lineage_policies WHERE policy_name=?1",
        [policy_name],
        |row| {
            Ok(LineagePolicyV3 {
                policy_id: row.get(0)?,
                policy_name: row.get(1)?,
                effect: row.get(2)?,
                priority: row.get(3)?,
                condition_json: row.get(4)?,
                created_by: row.get(5)?,
                created_at_ms: row.get(6)?,
            })
        },
    )
    .map_err(|e| {
        policy_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed reading lineage policy after write",
            serde_json::json!({ "error": e.to_string(), "policy_name": policy_name }),
        )
    })
}

pub fn lineage_policy_bind(
    conn: &Connection,
    subject_id: &str,
    policy_name: &str,
    bound_by: &str,
    now_ms: i64,
) -> AppResult<LineagePolicyBindingV3> {
    if subject_id.trim().is_empty() || policy_name.trim().is_empty() || bound_by.trim().is_empty() {
        return Err(policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "lineage policy binding requires non-empty subject_id, policy_name, and bound_by",
            serde_json::json!({
                "subject_id": subject_id,
                "policy_name": policy_name,
                "bound_by": bound_by
            }),
        ));
    }

    let policy = conn
        .query_row(
            "SELECT policy_id, policy_name, effect, priority, condition_json
             FROM lineage_policies
             WHERE policy_name=?1",
            [policy_name],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|e| {
            policy_error(
                "KC_LINEAGE_POLICY_CONDITION_INVALID",
                "lineage policy must exist before binding",
                serde_json::json!({ "error": e.to_string(), "policy_name": policy_name }),
            )
        })?;

    conn.execute(
        "INSERT INTO lineage_policy_bindings(subject_id, policy_id, bound_by, bound_at_ms)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(subject_id, policy_id) DO UPDATE SET
           bound_by=excluded.bound_by,
           bound_at_ms=excluded.bound_at_ms",
        params![subject_id, policy.0, bound_by, now_ms],
    )
    .map_err(|e| {
        policy_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage policy binding",
            serde_json::json!({
                "error": e.to_string(),
                "subject_id": subject_id,
                "policy_name": policy_name
            }),
        )
    })?;

    Ok(LineagePolicyBindingV3 {
        subject_id: subject_id.to_string(),
        policy_id: policy.0,
        policy_name: policy.1,
        effect: policy.2,
        priority: policy.3,
        condition_json: policy.4,
        bound_by: bound_by.to_string(),
        bound_at_ms: now_ms,
    })
}

pub fn lineage_policy_list(conn: &Connection) -> AppResult<Vec<LineagePolicyBindingV3>> {
    let mut stmt = conn
        .prepare(
            "SELECT b.subject_id, p.policy_id, p.policy_name, p.effect, p.priority, p.condition_json, b.bound_by, b.bound_at_ms
             FROM lineage_policy_bindings b
             JOIN lineage_policies p ON p.policy_id=b.policy_id
             ORDER BY p.priority ASC, p.policy_id ASC, b.subject_id ASC",
        )
        .map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing lineage policy list query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LineagePolicyBindingV3 {
                subject_id: row.get(0)?,
                policy_id: row.get(1)?,
                policy_name: row.get(2)?,
                effect: row.get(3)?,
                priority: row.get(4)?,
                condition_json: row.get(5)?,
                bound_by: row.get(6)?,
                bound_at_ms: row.get(7)?,
            })
        })
        .map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying lineage policy bindings",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding lineage policy binding row",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?);
    }
    Ok(out)
}

pub fn lineage_policy_decision(
    conn: &Connection,
    subject_id: &str,
    action: &str,
    doc_id: Option<&str>,
) -> AppResult<LineagePolicyDecisionV3> {
    if subject_id.trim().is_empty() || action.trim().is_empty() {
        return Err(policy_error(
            "KC_LINEAGE_POLICY_CONDITION_INVALID",
            "lineage policy decision requires non-empty subject_id and action",
            serde_json::json!({
                "subject_id": subject_id,
                "action": action
            }),
        ));
    }

    let mut stmt = conn
        .prepare(
            "SELECT p.policy_id, p.policy_name, p.effect, p.condition_json
             FROM lineage_policy_bindings b
             JOIN lineage_policies p ON p.policy_id=b.policy_id
             WHERE b.subject_id=?1
             ORDER BY p.priority ASC, p.policy_id ASC, b.subject_id ASC",
        )
        .map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing lineage policy decision query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map([subject_id], |row| {
                Ok(EvaluatedPolicy {
                    policy_id: row.get(0)?,
                    policy_name: row.get(1)?,
                    effect: row.get(2)?,
                    condition_json: row.get(3)?,
                })
            })
        .map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying lineage policies for decision",
                serde_json::json!({ "error": e.to_string(), "subject_id": subject_id }),
            )
        })?;

    let mut first_allow: Option<EvaluatedPolicy> = None;
    let mut first_deny: Option<EvaluatedPolicy> = None;
    for row in rows {
        let policy = row.map_err(|e| {
            policy_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding lineage policy row",
                serde_json::json!({ "error": e.to_string(), "subject_id": subject_id }),
            )
        })?;
        let (condition, _) = parse_condition(&policy.condition_json)?;
        if !evaluate_condition(&condition, action, doc_id) {
            continue;
        }
        if policy.effect == "deny" {
            if first_deny.is_none() {
                first_deny = Some(policy);
            }
            continue;
        }
        if policy.effect == "allow" && first_allow.is_none() {
            first_allow = Some(policy);
        }
    }

    if let Some(deny) = first_deny {
        return Ok(LineagePolicyDecisionV3 {
            subject_id: subject_id.to_string(),
            action: action.to_string(),
            doc_id: doc_id.map(|x| x.to_string()),
            allowed: false,
            reason: "policy_deny".to_string(),
            matched_policy_id: Some(deny.policy_id),
            matched_policy_name: Some(deny.policy_name),
            matched_effect: Some("deny".to_string()),
        });
    }
    if let Some(allow) = first_allow {
        return Ok(LineagePolicyDecisionV3 {
            subject_id: subject_id.to_string(),
            action: action.to_string(),
            doc_id: doc_id.map(|x| x.to_string()),
            allowed: true,
            reason: "policy_allow".to_string(),
            matched_policy_id: Some(allow.policy_id),
            matched_policy_name: Some(allow.policy_name),
            matched_effect: Some("allow".to_string()),
        });
    }
    Ok(LineagePolicyDecisionV3 {
        subject_id: subject_id.to_string(),
        action: action.to_string(),
        doc_id: doc_id.map(|x| x.to_string()),
        allowed: false,
        reason: "no_matching_allow_policy".to_string(),
        matched_policy_id: None,
        matched_policy_name: None,
        matched_effect: None,
    })
}

fn write_policy_audit(
    conn: &Connection,
    decision: &LineagePolicyDecisionV3,
    now_ms: i64,
) -> AppResult<()> {
    let details_json = canonical_json_string(&serde_json::json!({
        "subject_id": decision.subject_id,
        "action": decision.action,
        "doc_id": decision.doc_id,
        "reason": decision.reason,
        "matched_policy_id": decision.matched_policy_id,
        "matched_policy_name": decision.matched_policy_name,
        "matched_effect": decision.matched_effect
    }))?;
    conn.execute(
        "INSERT INTO lineage_policy_audit(
            ts_ms, subject_id, action, doc_id, allowed, reason, matched_policy_id, details_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            now_ms,
            decision.subject_id,
            decision.action,
            decision.doc_id,
            if decision.allowed { 1 } else { 0 },
            decision.reason,
            decision.matched_policy_id,
            details_json
        ],
    )
    .map_err(|e| {
        policy_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage policy audit row",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    Ok(())
}

pub fn ensure_lineage_policy_allows(
    conn: &Connection,
    subject_id: &str,
    action: &str,
    doc_id: Option<&str>,
    now_ms: i64,
) -> AppResult<()> {
    let decision = lineage_policy_decision(conn, subject_id, action, doc_id)?;
    write_policy_audit(conn, &decision, now_ms)?;
    if decision.allowed {
        return Ok(());
    }

    let code = if decision.matched_effect.as_deref() == Some("deny") {
        "KC_LINEAGE_POLICY_DENY_ENFORCED"
    } else {
        "KC_LINEAGE_PERMISSION_DENIED"
    };
    Err(policy_error(
        code,
        "lineage policy denied action",
        serde_json::json!({
            "subject_id": decision.subject_id,
            "action": decision.action,
            "doc_id": decision.doc_id,
            "reason": decision.reason,
            "matched_policy_id": decision.matched_policy_id,
            "matched_policy_name": decision.matched_policy_name,
            "matched_effect": decision.matched_effect
        }),
    ))
}
