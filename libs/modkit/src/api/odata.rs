use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use modkit_odata::{ast, CursorV1, Error as ODataError, ODataOrderBy, OrderKey, SortDir};
use odata_params::filters as od;
use serde::Deserialize;

// Re-export types from modkit-odata for convenience and better DX
pub use modkit_odata::ODataQuery;
// CursorV1 is available through the private import above for internal use

// Re-export error mapping from the error module
pub mod error;
pub use error::odata_error_to_problem;

#[derive(Deserialize, Default)]
pub struct ODataParams {
    #[serde(rename = "$filter")]
    pub filter: Option<String>,
    #[serde(rename = "$orderby")]
    pub orderby: Option<String>,
    #[serde(rename = "$select")]
    pub select: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

pub const MAX_FILTER_LEN: usize = 8 * 1024;
pub const MAX_NODES: usize = 2000;
pub const MAX_ORDERBY_LEN: usize = 1024;
pub const MAX_ORDER_FIELDS: usize = 10;
pub const MAX_SELECT_LEN: usize = 2048;
pub const MAX_SELECT_FIELDS: usize = 100;

/// Parse $select string into a list of field names.
/// Format: "field1, field2, field3, ..."
/// Field names are case-insensitive and whitespace is trimmed.
///
/// # Errors
/// Returns a `Problem` if the select string is invalid.
pub fn parse_select(raw: &str) -> Result<Vec<String>, crate::api::problem::Problem> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(crate::api::bad_request("$select cannot be empty"));
    }

    if raw.len() > MAX_SELECT_LEN {
        return Err(crate::api::bad_request("$select too long"));
    }

    let fields: Vec<String> = raw
        .split(',')
        .map(|f| f.trim().to_lowercase())
        .filter(|f| !f.is_empty())
        .collect();

    if fields.is_empty() {
        return Err(crate::api::bad_request(
            "$select must contain at least one field",
        ));
    }

    if fields.len() > MAX_SELECT_FIELDS {
        return Err(crate::api::bad_request("$select contains too many fields"));
    }

    // Check for duplicate fields
    let mut seen = std::collections::HashSet::new();
    for field in &fields {
        if !seen.insert(field.clone()) {
            return Err(crate::api::bad_request(format!(
                "duplicate field in $select: {field}",
            )));
        }
    }

    Ok(fields)
}

/// Parse $orderby string into `ODataOrderBy`.
/// Format: "field1 [asc|desc], field2 [asc|desc], ..."
/// Default direction is asc if not specified.
///
/// # Errors
/// Returns `modkit_odata::Error::InvalidOrderByField` if the orderby string is invalid.
pub fn parse_orderby(raw: &str) -> Result<ODataOrderBy, modkit_odata::Error> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(ODataOrderBy::empty());
    }

    if raw.len() > MAX_ORDERBY_LEN {
        return Err(modkit_odata::Error::InvalidOrderByField(
            "orderby too long".into(),
        ));
    }

    let mut keys = Vec::new();

    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = part.split_whitespace().collect();
        let (field, dir) = match tokens.as_slice() {
            [field] | [field, "asc"] => (*field, SortDir::Asc),
            [field, "desc"] => (*field, SortDir::Desc),
            _ => {
                return Err(modkit_odata::Error::InvalidOrderByField(format!(
                    "invalid orderby clause: {part}"
                )))
            }
        };

        if field.is_empty() {
            return Err(modkit_odata::Error::InvalidOrderByField(
                "empty field name in orderby".into(),
            ));
        }

        keys.push(OrderKey {
            field: field.to_owned(),
            dir,
        });
    }

    if keys.len() > MAX_ORDER_FIELDS {
        return Err(modkit_odata::Error::InvalidOrderByField(
            "too many order fields".into(),
        ));
    }

    Ok(ODataOrderBy(keys))
}

/// Extract and validate full `OData` query from request parts.
/// - Parses $filter, $orderby, limit, cursor
/// - Enforces budgets and validates formats
/// - Returns unified `ODataQuery`
///
/// # Errors
/// Returns `Problem` if any `OData` parameter is invalid.
pub async fn extract_odata_query<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<ODataQuery, crate::api::problem::Problem>
where
    S: Send + Sync,
{
    // Complexity budget (node count)
    fn count_nodes(e: &od::Expr) -> usize {
        use od::Expr::{And, Compare, Function, Identifier, In, Not, Or, Value};
        match e {
            Value(_) | Identifier(_) => 1,
            Not(x) => 1 + count_nodes(x),
            And(a, b) | Or(a, b) | Compare(a, _, b) => 1 + count_nodes(a) + count_nodes(b),
            In(a, list) => 1 + count_nodes(a) + list.iter().map(count_nodes).sum::<usize>(),
            Function(_, args) => 1 + args.iter().map(count_nodes).sum::<usize>(),
        }
    }

    // Parse query; default if missing
    let Query(params) = Query::<ODataParams>::from_request_parts(parts, state)
        .await
        .unwrap_or_else(|_| Query(ODataParams::default()));

    let mut query = ODataQuery::new();

    // Parse filter
    if let Some(raw_filter) = params.filter.as_ref() {
        let raw = raw_filter.trim();
        if !raw.is_empty() {
            if raw.len() > MAX_FILTER_LEN {
                return Err(crate::api::bad_request("Filter too long"));
            }

            // Parse into odata_params AST
            let ast_src = od::parse_str(raw)
                .map_err(|e| crate::api::bad_request(format!("invalid $filter: {e:?}")))?;

            if count_nodes(&ast_src) > MAX_NODES {
                return Err(crate::api::bad_request("Filter too complex"));
            }

            // Convert to transport-agnostic core AST
            let core_expr: ast::Expr = ast_src.into();

            // Generate filter hash for cursor consistency
            let filter_hash = modkit_odata::pagination::short_filter_hash(Some(&core_expr));

            query = query.with_filter(core_expr);
            if let Some(hash) = filter_hash {
                query = query.with_filter_hash(hash);
            }
        }
    }

    // Check for cursor+orderby conflict before parsing either
    if params.cursor.is_some() && params.orderby.is_some() {
        return Err(crate::api::odata::odata_error_to_problem(
            &ODataError::OrderWithCursor,
            "/",
            None,
        ));
    }

    // Parse cursor first (if present, skip orderby)
    if let Some(cursor_str) = params.cursor.as_ref() {
        let cursor = CursorV1::decode(cursor_str).map_err(|_| {
            crate::api::odata::odata_error_to_problem(&ODataError::InvalidCursor, "/", None)
        })?;
        query = query.with_cursor(cursor);
        // When cursor is present, order is empty (derived from cursor.s later)
        query = query.with_order(ODataOrderBy::empty());
    } else if let Some(raw_orderby) = params.orderby.as_ref() {
        // Parse orderby only when cursor is absent
        let order = parse_orderby(raw_orderby)
            .map_err(|e| crate::api::odata::odata_error_to_problem(&e, "/", None))?;
        query = query.with_order(order);
    }

    // Parse limit
    if let Some(limit) = params.limit {
        if limit == 0 {
            return Err(crate::api::odata::odata_error_to_problem(
                &ODataError::InvalidLimit,
                "/",
                None,
            ));
        }
        query = query.with_limit(limit);
    }

    // Parse select
    if let Some(raw_select) = params.select.as_ref() {
        let fields = parse_select(raw_select)?;
        query = query.with_select(fields);
    }

    Ok(query)
}

use std::ops::Deref;

/// Simple Axum extractor for full `OData` query parameters.
/// Parses $filter, $orderby, limit, and cursor parameters.
/// Usage in handlers:
///   async fn `list_users(OData(query)`: `OData`, /* ... */) { /* use `query` */ }
#[derive(Debug, Clone)]
pub struct OData(pub ODataQuery);

impl OData {
    #[inline]
    pub fn into_inner(self) -> ODataQuery {
        self.0
    }
}

impl Deref for OData {
    type Target = ODataQuery;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<ODataQuery> for OData {
    #[inline]
    fn as_ref(&self) -> &ODataQuery {
        &self.0
    }
}

impl From<OData> for ODataQuery {
    #[inline]
    fn from(x: OData) -> Self {
        x.0
    }
}

impl<S> FromRequestParts<S> for OData
where
    S: Send + Sync,
{
    type Rejection = crate::api::problem::Problem;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl core::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let query = extract_odata_query(parts, state).await?;
            Ok(OData(query))
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "odata_tests.rs"]
mod odata_tests;
