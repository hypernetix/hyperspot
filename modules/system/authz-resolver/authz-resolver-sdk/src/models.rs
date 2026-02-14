//! Domain models for the `AuthZ` resolver module.
//!
//! Based on `AuthZEN` 1.0 evaluation model with constraint extensions.

use std::collections::HashMap;

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constraints::Constraint;

/// Tenant hierarchy mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenantMode {
    /// Only the specified root tenant (no subtree expansion).
    RootOnly,
    /// The root tenant and all descendants (default).
    #[default]
    Subtree,
}

/// Controls how barriers (self-managed tenants) are handled during `AuthZ` evaluation.
///
/// Consistent with `tenant_resolver_sdk::BarrierMode`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BarrierMode {
    /// Respect all barriers — stop at barrier boundaries (default).
    #[default]
    Respect,
    /// Ignore barriers — traverse through self-managed tenants.
    Ignore,
}

/// PEP-level capability declarations.
///
/// Tells the PDP which advanced features the PEP can handle so the PDP
/// can tailor its response accordingly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// PEP understands tenant hierarchy constraints.
    TenantHierarchy,
    /// PEP understands group membership constraints.
    GroupMembership,
    /// PEP understands group hierarchy constraints.
    GroupHierarchy,
}

/// Reason for an explicit deny from the PDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenyReason {
    /// Machine-readable error code.
    pub error_code: String,
    /// Human-readable details (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Authorization evaluation request.
///
/// Follows the `AuthZEN` 1.0 model: Subject + Action + Resource + Context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRequest {
    /// The subject (who is making the request).
    pub subject: Subject,
    /// The action being performed.
    pub action: Action,
    /// The resource being accessed.
    pub resource: Resource,
    /// Additional context for the evaluation.
    pub context: EvaluationRequestContext,
}

/// The authenticated subject making the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // field names follow AuthZEN spec
pub struct Subject {
    /// Subject identifier (user ID, service ID).
    pub id: Uuid,
    /// Subject type (e.g., "user", "service").
    /// Serialized as `"type"` to match the `AuthZEN` spec.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<String>,
    /// Additional subject properties for policy evaluation.
    /// The subject's home tenant ID goes here as `"tenant_id"`.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// The action being performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action name (e.g., "list", "get", "create", "update", "delete").
    pub name: String,
}

/// The resource being accessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // field names follow AuthZEN spec
pub struct Resource {
    /// Resource type identifier (e.g., "`users_info.user`").
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Specific resource ID (for GET/UPDATE/DELETE on a single resource).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    /// Additional resource properties for policy evaluation.
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Tenant context for the evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantContext {
    /// Tenant hierarchy mode (default: `Subtree`).
    #[serde(default)]
    pub mode: TenantMode,
    /// The context tenant ID (tenant being operated on).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_id: Option<Uuid>,
    /// Barrier enforcement mode (default: `All`).
    #[serde(default)]
    pub barrier_mode: BarrierMode,
    /// Required tenant status filter (e.g., `["active"]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_status: Option<Vec<String>>,
}

/// Additional evaluation request context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // field names follow design doc
pub struct EvaluationRequestContext {
    /// Tenant context for multi-tenant scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_context: Option<TenantContext>,
    /// Token scopes from the `AuthN` result.
    #[serde(default)]
    pub token_scopes: Vec<String>,
    /// Whether the PDP should return row-level constraints.
    /// - `true` for LIST/GET/UPDATE/DELETE (need scope filtering)
    /// - `false` for CREATE (just need decision)
    #[serde(default)]
    pub require_constraints: bool,
    /// PEP capabilities (tells PDP what the PEP can handle).
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// Supported constraint properties (tells PDP which properties the PEP understands).
    #[serde(default)]
    pub supported_properties: Vec<String>,
    /// Original bearer token for PDP forwarding. Wrapped in `SecretString` to prevent
    /// accidental logging. Skipped during serialization — the PDP receives the token
    /// through a separate channel if needed.
    #[serde(skip)]
    pub bearer_token: Option<SecretString>,
}

/// Authorization evaluation response context.
///
/// Contains constraints (when `decision` is `true`) or deny reason
/// (when `decision` is `false`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationResponseContext {
    /// Row-level constraints to apply when `decision` is `true`.
    /// Empty when `require_constraints` was `false` or when access is unrestricted.
    /// Multiple constraints are `ORed` (any one matching is sufficient).
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Reason for denial (present when `decision` is `false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny_reason: Option<DenyReason>,
}

/// Authorization evaluation response.
///
/// The PDP returns a decision (allow/deny) and optionally a context
/// containing constraints or deny reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResponse {
    /// Whether access is granted.
    pub decision: bool,
    /// Response context with constraints or deny reason.
    #[serde(default)]
    pub context: EvaluationResponseContext,
}
