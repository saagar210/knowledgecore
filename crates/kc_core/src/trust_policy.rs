use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

const DEFAULT_TENANT_TEMPLATE_CLOCK_SKEW_MS: i64 = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustProviderPolicyV1 {
    pub provider_id: String,
    pub max_clock_skew_ms: i64,
    pub require_claims_json: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustSessionRevocationV1 {
    pub session_id: String,
    pub revoked_by: String,
    pub revoked_at_ms: i64,
    pub details_json: String,
}

fn policy_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, "trust_policy", message, false, details)
}

fn canonical_json_string(value: &Value) -> AppResult<String> {
    let bytes = to_canonical_bytes(value)?;
    String::from_utf8(bytes).map_err(|e| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "failed to encode canonical trust policy JSON",
            serde_json::json!({ "error": e.to_string() }),
        )
    })
}

fn parse_require_claims(require_claims_json: &str) -> AppResult<Map<String, Value>> {
    let parsed = serde_json::from_str::<Value>(require_claims_json).map_err(|e| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "failed parsing required claims JSON",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    match parsed {
        Value::Object(map) => {
            for (key, value) in &map {
                if key.trim().is_empty() {
                    return Err(policy_error(
                        "KC_TRUST_PROVIDER_POLICY_INVALID",
                        "required claim key must not be empty",
                        serde_json::json!({ "key": key }),
                    ));
                }
                if !value.is_string() {
                    return Err(policy_error(
                        "KC_TRUST_PROVIDER_POLICY_INVALID",
                        "required claims values must be strings",
                        serde_json::json!({ "key": key, "value_type": value }),
                    ));
                }
            }
            Ok(map)
        }
        _ => Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "required claims JSON must be an object",
            serde_json::json!({}),
        )),
    }
}

fn normalize_tenant_id(tenant_id: &str) -> AppResult<String> {
    let normalized = tenant_id.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "tenant_id must not be empty",
            serde_json::json!({}),
        ));
    }
    Ok(normalized)
}

pub fn trust_provider_policy_set_tenant_template(
    conn: &Connection,
    provider_id: &str,
    issuer: &str,
    audience: &str,
    tenant_id: &str,
    now_ms: i64,
) -> AppResult<TrustProviderPolicyV1> {
    let tenant_id = normalize_tenant_id(tenant_id)?;
    let issuer = issuer.trim().trim_end_matches('/').to_ascii_lowercase();
    if issuer.is_empty() || audience.trim().is_empty() {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "issuer and audience are required",
            serde_json::json!({ "issuer": issuer, "audience": audience }),
        ));
    }

    let claims = serde_json::json!({
        "aud": audience.trim(),
        "iss": issuer,
        "tenant": tenant_id
    });
    let require_claims_json = canonical_json_string(&claims)?;

    trust_provider_policy_set(
        conn,
        provider_id,
        DEFAULT_TENANT_TEMPLATE_CLOCK_SKEW_MS,
        &require_claims_json,
        now_ms,
    )
}

pub fn trust_provider_policy_set(
    conn: &Connection,
    provider_id: &str,
    max_clock_skew_ms: i64,
    require_claims_json: &str,
    now_ms: i64,
) -> AppResult<TrustProviderPolicyV1> {
    if provider_id.trim().is_empty() {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "provider_id is required",
            serde_json::json!({}),
        ));
    }
    if max_clock_skew_ms < 0 {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "max_clock_skew_ms must be >= 0",
            serde_json::json!({ "max_clock_skew_ms": max_clock_skew_ms }),
        ));
    }

    let provider_exists = conn
        .query_row(
            "SELECT 1 FROM trust_providers WHERE provider_id=?1",
            [provider_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|e| {
            policy_error(
                "KC_TRUST_PROVIDER_POLICY_INVALID",
                "failed querying trust providers",
                serde_json::json!({ "error": e.to_string(), "provider_id": provider_id }),
            )
        })?
        .is_some();
    if !provider_exists {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "provider must exist before policy can be set",
            serde_json::json!({ "provider_id": provider_id }),
        ));
    }

    let claims_obj = parse_require_claims(require_claims_json)?;
    let canonical_claims_json = canonical_json_string(&Value::Object(claims_obj))?;

    conn.execute(
        "INSERT INTO trust_provider_policies(provider_id, max_clock_skew_ms, require_claims_json, updated_at_ms)
         VALUES(?1, ?2, ?3, ?4)
         ON CONFLICT(provider_id) DO UPDATE SET
           max_clock_skew_ms=excluded.max_clock_skew_ms,
           require_claims_json=excluded.require_claims_json,
           updated_at_ms=excluded.updated_at_ms",
        params![
            provider_id,
            max_clock_skew_ms,
            canonical_claims_json,
            now_ms
        ],
    )
    .map_err(|e| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "failed writing trust provider policy",
            serde_json::json!({ "error": e.to_string(), "provider_id": provider_id }),
        )
    })?;

    trust_provider_policy_get(conn, provider_id)?.ok_or_else(|| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "policy disappeared after write",
            serde_json::json!({ "provider_id": provider_id }),
        )
    })
}

pub fn trust_provider_policy_get(
    conn: &Connection,
    provider_id: &str,
) -> AppResult<Option<TrustProviderPolicyV1>> {
    conn.query_row(
        "SELECT provider_id, max_clock_skew_ms, require_claims_json, updated_at_ms
         FROM trust_provider_policies
         WHERE provider_id=?1",
        [provider_id],
        |row| {
            Ok(TrustProviderPolicyV1 {
                provider_id: row.get(0)?,
                max_clock_skew_ms: row.get(1)?,
                require_claims_json: row.get(2)?,
                updated_at_ms: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|e| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "failed reading trust provider policy",
            serde_json::json!({ "error": e.to_string(), "provider_id": provider_id }),
        )
    })
}

pub fn trust_session_revoke(
    conn: &Connection,
    session_id: &str,
    revoked_by: &str,
    now_ms: i64,
) -> AppResult<TrustSessionRevocationV1> {
    if session_id.trim().is_empty() || revoked_by.trim().is_empty() {
        return Err(policy_error(
            "KC_TRUST_SESSION_REVOKED",
            "session_id and revoked_by are required",
            serde_json::json!({}),
        ));
    }

    let session_exists = conn
        .query_row(
            "SELECT 1 FROM identity_sessions WHERE session_id=?1",
            [session_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|e| {
            policy_error(
                "KC_TRUST_SESSION_REVOKED",
                "failed querying identity sessions",
                serde_json::json!({ "error": e.to_string(), "session_id": session_id }),
            )
        })?
        .is_some();
    if !session_exists {
        return Err(policy_error(
            "KC_TRUST_SESSION_REVOKED",
            "cannot revoke unknown identity session",
            serde_json::json!({ "session_id": session_id }),
        ));
    }

    let details_json = canonical_json_string(&serde_json::json!({
        "revoked_by": revoked_by
    }))?;

    conn.execute(
        "INSERT INTO trust_session_revocations(session_id, revoked_by, revoked_at_ms, details_json)
         VALUES(?1, ?2, ?3, ?4)
         ON CONFLICT(session_id) DO UPDATE SET
           revoked_by=excluded.revoked_by,
           revoked_at_ms=excluded.revoked_at_ms,
           details_json=excluded.details_json",
        params![session_id, revoked_by, now_ms, details_json],
    )
    .map_err(|e| {
        policy_error(
            "KC_TRUST_SESSION_REVOKED",
            "failed writing session revocation",
            serde_json::json!({ "error": e.to_string(), "session_id": session_id }),
        )
    })?;

    Ok(TrustSessionRevocationV1 {
        session_id: session_id.to_string(),
        revoked_by: revoked_by.to_string(),
        revoked_at_ms: now_ms,
        details_json,
    })
}

pub fn trust_session_is_revoked(conn: &Connection, session_id: &str) -> AppResult<bool> {
    conn.query_row(
        "SELECT 1 FROM trust_session_revocations WHERE session_id=?1",
        [session_id],
        |_| Ok(()),
    )
    .optional()
    .map(|opt| opt.is_some())
    .map_err(|e| {
        policy_error(
            "KC_TRUST_SESSION_REVOKED",
            "failed reading session revocation state",
            serde_json::json!({ "error": e.to_string(), "session_id": session_id }),
        )
    })
}

pub fn ensure_session_policy_allows(
    conn: &Connection,
    provider_id: &str,
    session_id: &str,
    claim_subset_json: &str,
    issued_at_ms: i64,
    expires_at_ms: i64,
    now_ms: i64,
) -> AppResult<()> {
    if trust_session_is_revoked(conn, session_id)? {
        return Err(policy_error(
            "KC_TRUST_SESSION_REVOKED",
            "identity session has been revoked",
            serde_json::json!({ "session_id": session_id, "provider_id": provider_id }),
        ));
    }

    let Some(policy) = trust_provider_policy_get(conn, provider_id)? else {
        return Ok(());
    };

    let lower = issued_at_ms - policy.max_clock_skew_ms;
    let upper = expires_at_ms + policy.max_clock_skew_ms;
    if now_ms < lower || now_ms > upper {
        return Err(policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "identity session falls outside provider clock skew policy",
            serde_json::json!({
                "provider_id": provider_id,
                "session_id": session_id,
                "now_ms": now_ms,
                "lower_bound_ms": lower,
                "upper_bound_ms": upper
            }),
        ));
    }

    let required = parse_require_claims(&policy.require_claims_json)?;
    if required.is_empty() {
        return Ok(());
    }

    let claim_subset = serde_json::from_str::<Value>(claim_subset_json).map_err(|e| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "session claim subset is not valid JSON",
            serde_json::json!({
                "error": e.to_string(),
                "provider_id": provider_id,
                "session_id": session_id
            }),
        )
    })?;
    let claim_obj = claim_subset.as_object().ok_or_else(|| {
        policy_error(
            "KC_TRUST_PROVIDER_POLICY_INVALID",
            "session claim subset must be an object",
            serde_json::json!({ "provider_id": provider_id, "session_id": session_id }),
        )
    })?;

    for (key, value) in required {
        match claim_obj.get(&key) {
            Some(actual) if actual == &value => {}
            _ => {
                return Err(policy_error(
                    "KC_TRUST_PROVIDER_POLICY_INVALID",
                    "session claims do not satisfy provider policy",
                    serde_json::json!({
                        "provider_id": provider_id,
                        "session_id": session_id,
                        "claim_key": key,
                        "expected": value,
                        "actual": claim_obj.get(&key).cloned()
                    }),
                ));
            }
        }
    }

    Ok(())
}
