use crate::app_error::{AppError, AppResult};
use crate::hashing::blake3_hex_prefixed;
use crate::lineage::LINEAGE_LOCK_LEASE_MS;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageRoleBindingV2 {
    pub subject_id: String,
    pub role_name: String,
    pub role_rank: i64,
    pub granted_by: String,
    pub granted_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineagePermissionDecisionV2 {
    pub subject_id: String,
    pub action: String,
    pub allowed: bool,
    pub matched_role: Option<String>,
    pub matched_rank: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageScopeLockLeaseV2 {
    pub scope_kind: String,
    pub scope_value: String,
    pub owner: String,
    pub token: String,
    pub acquired_at_ms: i64,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageScopeLockStatusV2 {
    pub scope_kind: String,
    pub scope_value: String,
    pub held: bool,
    pub owner: Option<String>,
    pub acquired_at_ms: Option<i64>,
    pub expires_at_ms: Option<i64>,
    pub expired: bool,
}

#[derive(Debug, Clone)]
struct ScopeLockRow {
    owner: String,
    token: String,
    acquired_at_ms: i64,
    expires_at_ms: i64,
}

fn governance_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, "lineage", message, false, details)
}

fn require_non_empty(name: &str, value: &str, code: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(governance_error(
            code,
            "lineage governance field must not be empty",
            serde_json::json!({ "field": name }),
        ));
    }
    Ok(())
}

fn role_rank(conn: &Connection, role_name: &str) -> AppResult<Option<i64>> {
    let out = conn.query_row(
        "SELECT role_rank FROM lineage_roles WHERE role_name=?1",
        params![role_name],
        |row| row.get::<_, i64>(0),
    );
    match out {
        Ok(rank) => Ok(Some(rank)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed loading lineage role rank",
            serde_json::json!({ "error": e.to_string(), "role_name": role_name }),
        )),
    }
}

pub fn lineage_role_grant(
    conn: &Connection,
    subject_id: &str,
    role_name: &str,
    granted_by: &str,
    now_ms: i64,
) -> AppResult<LineageRoleBindingV2> {
    require_non_empty("subject_id", subject_id, "KC_LINEAGE_ROLE_INVALID")?;
    require_non_empty("role_name", role_name, "KC_LINEAGE_ROLE_INVALID")?;
    require_non_empty("granted_by", granted_by, "KC_LINEAGE_ROLE_INVALID")?;

    let Some(rank) = role_rank(conn, role_name)? else {
        return Err(governance_error(
            "KC_LINEAGE_ROLE_INVALID",
            "lineage role does not exist",
            serde_json::json!({ "role_name": role_name }),
        ));
    };

    conn.execute(
        "INSERT INTO lineage_role_bindings(subject_id, role_name, granted_by, granted_at_ms)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(subject_id, role_name) DO UPDATE SET
           granted_by=excluded.granted_by,
           granted_at_ms=excluded.granted_at_ms",
        params![subject_id, role_name, granted_by, now_ms],
    )
    .map_err(|e| {
        governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage role binding",
            serde_json::json!({
                "error": e.to_string(),
                "subject_id": subject_id,
                "role_name": role_name
            }),
        )
    })?;

    Ok(LineageRoleBindingV2 {
        subject_id: subject_id.to_string(),
        role_name: role_name.to_string(),
        role_rank: rank,
        granted_by: granted_by.to_string(),
        granted_at_ms: now_ms,
    })
}

pub fn lineage_role_revoke(conn: &Connection, subject_id: &str, role_name: &str) -> AppResult<()> {
    require_non_empty("subject_id", subject_id, "KC_LINEAGE_ROLE_INVALID")?;
    require_non_empty("role_name", role_name, "KC_LINEAGE_ROLE_INVALID")?;

    if role_rank(conn, role_name)?.is_none() {
        return Err(governance_error(
            "KC_LINEAGE_ROLE_INVALID",
            "lineage role does not exist",
            serde_json::json!({ "role_name": role_name }),
        ));
    }

    let removed = conn
        .execute(
            "DELETE FROM lineage_role_bindings WHERE subject_id=?1 AND role_name=?2",
            params![subject_id, role_name],
        )
        .map_err(|e| {
            governance_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed deleting lineage role binding",
                serde_json::json!({
                    "error": e.to_string(),
                    "subject_id": subject_id,
                    "role_name": role_name
                }),
            )
        })?;

    if removed == 0 {
        return Err(governance_error(
            "KC_LINEAGE_ROLE_INVALID",
            "lineage role binding does not exist",
            serde_json::json!({ "subject_id": subject_id, "role_name": role_name }),
        ));
    }
    Ok(())
}

pub fn lineage_role_list(conn: &Connection) -> AppResult<Vec<LineageRoleBindingV2>> {
    let mut stmt = conn
        .prepare(
            "SELECT b.subject_id, b.role_name, r.role_rank, b.granted_by, b.granted_at_ms
             FROM lineage_role_bindings b
             JOIN lineage_roles r ON r.role_name=b.role_name
             ORDER BY r.role_rank ASC, b.subject_id ASC, b.role_name ASC",
        )
        .map_err(|e| {
            governance_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing lineage role list query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LineageRoleBindingV2 {
                subject_id: row.get(0)?,
                role_name: row.get(1)?,
                role_rank: row.get(2)?,
                granted_by: row.get(3)?,
                granted_at_ms: row.get(4)?,
            })
        })
        .map_err(|e| {
            governance_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying lineage role bindings",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| {
            governance_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding lineage role binding row",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?);
    }
    Ok(out)
}

pub fn lineage_permission_decision(
    conn: &Connection,
    subject_id: &str,
    action: &str,
) -> AppResult<LineagePermissionDecisionV2> {
    require_non_empty("subject_id", subject_id, "KC_LINEAGE_PERMISSION_DENIED")?;
    require_non_empty("action", action, "KC_LINEAGE_PERMISSION_DENIED")?;

    let out = conn.query_row(
        "SELECT b.role_name, r.role_rank, p.allowed
         FROM lineage_role_bindings b
         JOIN lineage_roles r ON r.role_name=b.role_name
         JOIN lineage_permissions p ON p.role_name=b.role_name AND p.action=?2
         WHERE b.subject_id=?1
         ORDER BY r.role_rank ASC, b.subject_id ASC, b.role_name ASC
         LIMIT 1",
        params![subject_id, action],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    );

    match out {
        Ok((role_name, role_rank, allowed)) => Ok(LineagePermissionDecisionV2 {
            subject_id: subject_id.to_string(),
            action: action.to_string(),
            allowed: allowed == 1,
            matched_role: Some(role_name),
            matched_rank: Some(role_rank),
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(LineagePermissionDecisionV2 {
            subject_id: subject_id.to_string(),
            action: action.to_string(),
            allowed: false,
            matched_role: None,
            matched_rank: None,
        }),
        Err(e) => Err(governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed evaluating lineage permission",
            serde_json::json!({ "error": e.to_string(), "subject_id": subject_id, "action": action }),
        )),
    }
}

pub fn ensure_lineage_permission(
    conn: &Connection,
    subject_id: &str,
    action: &str,
    doc_id: Option<&str>,
) -> AppResult<()> {
    let decision = lineage_permission_decision(conn, subject_id, action)?;
    if decision.allowed {
        return Ok(());
    }
    Err(governance_error(
        "KC_LINEAGE_PERMISSION_DENIED",
        "lineage action is not allowed for subject",
        serde_json::json!({
            "subject_id": decision.subject_id,
            "action": decision.action,
            "doc_id": doc_id,
            "matched_role": decision.matched_role,
            "matched_rank": decision.matched_rank
        }),
    ))
}

fn normalize_scope(scope_kind: &str, scope_value: &str) -> AppResult<(String, String)> {
    let kind = scope_kind.trim().to_ascii_lowercase();
    let value = scope_value.trim().to_string();

    if kind != "doc" && kind != "set" {
        return Err(governance_error(
            "KC_LINEAGE_SCOPE_INVALID",
            "unsupported lineage lock scope kind",
            serde_json::json!({ "scope_kind": scope_kind, "supported": ["doc", "set"] }),
        ));
    }
    if value.is_empty() {
        return Err(governance_error(
            "KC_LINEAGE_SCOPE_INVALID",
            "lineage lock scope value must not be empty",
            serde_json::json!({ "scope_kind": kind }),
        ));
    }
    Ok((kind, value))
}

fn scope_lock_token_for(scope_kind: &str, scope_value: &str, owner: &str, now_ms: i64) -> String {
    blake3_hex_prefixed(
        format!("kc.lineage.scope-lock.v2\n{scope_kind}\n{scope_value}\n{owner}\n{now_ms}")
            .as_bytes(),
    )
}

fn read_scope_lock_row(
    conn: &Connection,
    scope_kind: &str,
    scope_value: &str,
) -> AppResult<Option<ScopeLockRow>> {
    let out = conn.query_row(
        "SELECT owner, token, acquired_at_ms, expires_at_ms
         FROM lineage_lock_scopes
         WHERE scope_kind=?1 AND scope_value=?2",
        params![scope_kind, scope_value],
        |row| {
            Ok(ScopeLockRow {
                owner: row.get(0)?,
                token: row.get(1)?,
                acquired_at_ms: row.get(2)?,
                expires_at_ms: row.get(3)?,
            })
        },
    );
    match out {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed reading lineage scoped lock",
            serde_json::json!({
                "error": e.to_string(),
                "scope_kind": scope_kind,
                "scope_value": scope_value
            }),
        )),
    }
}

pub fn lineage_lock_acquire_scope(
    conn: &Connection,
    scope_kind: &str,
    scope_value: &str,
    owner: &str,
    now_ms: i64,
) -> AppResult<LineageScopeLockLeaseV2> {
    require_non_empty("owner", owner, "KC_LINEAGE_LOCK_INVALID")?;
    let (scope_kind, scope_value) = normalize_scope(scope_kind, scope_value)?;

    if let Some(lock) = read_scope_lock_row(conn, &scope_kind, &scope_value)? {
        if lock.expires_at_ms > now_ms {
            return Err(governance_error(
                "KC_LINEAGE_LOCK_HELD",
                "lineage scoped lock is already held",
                serde_json::json!({
                    "scope_kind": scope_kind,
                    "scope_value": scope_value,
                    "owner": lock.owner,
                    "expires_at_ms": lock.expires_at_ms
                }),
            ));
        }
    }

    let token = scope_lock_token_for(&scope_kind, &scope_value, owner, now_ms);
    let expires_at_ms = now_ms + LINEAGE_LOCK_LEASE_MS;
    conn.execute(
        "INSERT INTO lineage_lock_scopes(scope_kind, scope_value, owner, token, acquired_at_ms, expires_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(scope_kind, scope_value) DO UPDATE SET
           owner=excluded.owner,
           token=excluded.token,
           acquired_at_ms=excluded.acquired_at_ms,
           expires_at_ms=excluded.expires_at_ms",
        params![scope_kind, scope_value, owner, token, now_ms, expires_at_ms],
    )
    .map_err(|e| {
        governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage scoped lock",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    Ok(LineageScopeLockLeaseV2 {
        scope_kind,
        scope_value,
        owner: owner.to_string(),
        token,
        acquired_at_ms: now_ms,
        expires_at_ms,
    })
}

pub fn lineage_lock_release_scope(
    conn: &Connection,
    scope_kind: &str,
    scope_value: &str,
    token: &str,
) -> AppResult<()> {
    let (scope_kind, scope_value) = normalize_scope(scope_kind, scope_value)?;
    let Some(lock) = read_scope_lock_row(conn, &scope_kind, &scope_value)? else {
        return Err(governance_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage scoped lock does not exist",
            serde_json::json!({ "scope_kind": scope_kind, "scope_value": scope_value }),
        ));
    };
    if lock.token != token {
        return Err(governance_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage scoped lock token is invalid",
            serde_json::json!({ "scope_kind": scope_kind, "scope_value": scope_value }),
        ));
    }

    conn.execute(
        "DELETE FROM lineage_lock_scopes WHERE scope_kind=?1 AND scope_value=?2",
        params![scope_kind, scope_value],
    )
    .map_err(|e| {
        governance_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed deleting lineage scoped lock",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    Ok(())
}

pub fn lineage_lock_scope_status(
    conn: &Connection,
    scope_kind: &str,
    scope_value: &str,
    now_ms: i64,
) -> AppResult<LineageScopeLockStatusV2> {
    let (scope_kind, scope_value) = normalize_scope(scope_kind, scope_value)?;
    let Some(lock) = read_scope_lock_row(conn, &scope_kind, &scope_value)? else {
        return Ok(LineageScopeLockStatusV2 {
            scope_kind,
            scope_value,
            held: false,
            owner: None,
            acquired_at_ms: None,
            expires_at_ms: None,
            expired: false,
        });
    };

    let expired = lock.expires_at_ms <= now_ms;
    Ok(LineageScopeLockStatusV2 {
        scope_kind,
        scope_value,
        held: !expired,
        owner: Some(lock.owner),
        acquired_at_ms: Some(lock.acquired_at_ms),
        expires_at_ms: Some(lock.expires_at_ms),
        expired,
    })
}
