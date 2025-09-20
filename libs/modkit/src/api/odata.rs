use axum::extract::{FromRequestParts, Query};
use axum::http::{request::Parts, StatusCode};
use odata_core::{ast, CursorV1, ODataOrderBy, OrderKey, SortDir};
use odata_params::filters as od;
use serde::Deserialize;

// Re-export types from odata-core for convenience
pub use odata_core::{CursorError, ODataQuery};
// SortDir and CursorV1 are available through the private imports above for internal use

#[derive(Deserialize, Default)]
pub struct ODataParams {
    #[serde(rename = "$filter")]
    pub filter: Option<String>,
    #[serde(rename = "$orderby")]
    pub orderby: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

pub const MAX_FILTER_LEN: usize = 8 * 1024;
pub const MAX_NODES: usize = 2000;
pub const MAX_ORDERBY_LEN: usize = 1024;
pub const MAX_ORDER_FIELDS: usize = 10;

/// Parse $orderby string into ODataOrderBy
/// Format: "field1 [asc|desc], field2 [asc|desc], ..."
/// Default direction is asc if not specified
pub fn parse_orderby(raw: &str) -> Result<ODataOrderBy, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(ODataOrderBy::empty());
    }

    if raw.len() > MAX_ORDERBY_LEN {
        return Err("orderby too long".into());
    }

    let mut keys = Vec::new();

    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = part.split_whitespace().collect();
        let (field, dir) = match tokens.as_slice() {
            [field] => (*field, SortDir::Asc),
            [field, "asc"] => (*field, SortDir::Asc),
            [field, "desc"] => (*field, SortDir::Desc),
            _ => return Err(format!("invalid orderby clause: {}", part)),
        };

        if field.is_empty() {
            return Err("empty field name in orderby".into());
        }

        keys.push(OrderKey {
            field: field.to_string(),
            dir,
        });
    }

    if keys.len() > MAX_ORDER_FIELDS {
        return Err("too many order fields".into());
    }

    Ok(ODataOrderBy(keys))
}

/// Extract and validate full OData query from request parts
/// - Parses $filter, $orderby, limit, cursor
/// - Enforces budgets and validates formats
/// - Returns unified ODataQuery
pub async fn extract_odata_query<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<ODataQuery, (StatusCode, String)>
where
    S: Send + Sync,
{
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
                return Err((StatusCode::BAD_REQUEST, "filter too long".into()));
            }

            // Parse into odata_params AST
            let ast_src = od::parse_str(raw)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid $filter: {:?}", e)))?;

            // Complexity budget (node count)
            fn count_nodes(e: &od::Expr) -> usize {
                use od::Expr::*;
                match e {
                    Value(_) | Identifier(_) => 1,
                    Not(x) => 1 + count_nodes(x),
                    And(a, b) | Or(a, b) | Compare(a, _, b) => 1 + count_nodes(a) + count_nodes(b),
                    In(a, list) => 1 + count_nodes(a) + list.iter().map(count_nodes).sum::<usize>(),
                    Function(_, args) => 1 + args.iter().map(count_nodes).sum::<usize>(),
                }
            }
            if count_nodes(&ast_src) > MAX_NODES {
                return Err((StatusCode::BAD_REQUEST, "filter too complex".into()));
            }

            // Convert to transport-agnostic core AST
            let core_expr: ast::Expr = ast_src.into();

            // Generate filter hash for cursor consistency
            let filter_hash = crate::api::pagination::short_filter_hash(Some(&core_expr));

            query = query.with_filter(core_expr);
            if let Some(hash) = filter_hash {
                query = query.with_filter_hash(hash);
            }
        }
    }

    // Check for cursor+orderby conflict before parsing either
    if params.cursor.is_some() && params.orderby.is_some() {
        return Err((StatusCode::BAD_REQUEST, "ORDER_WITH_CURSOR".into()));
    }

    // Parse cursor first (if present, skip orderby)
    if let Some(cursor_str) = params.cursor.as_ref() {
        let cursor = CursorV1::decode(cursor_str)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid cursor: {:?}", e)))?;
        query = query.with_cursor(cursor);
        // When cursor is present, order is empty (derived from cursor.s later)
        query = query.with_order(ODataOrderBy::empty());
    } else {
        // Parse orderby only when cursor is absent
        if let Some(raw_orderby) = params.orderby.as_ref() {
            let order = parse_orderby(raw_orderby)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid $orderby: {}", e)))?;
            query = query.with_order(order);
        }
    }

    // Parse limit
    if let Some(limit) = params.limit {
        if limit == 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "limit must be greater than 0".into(),
            ));
        }
        query = query.with_limit(limit);
    }

    Ok(query)
}

use std::ops::Deref;

/// Simple Axum extractor for full OData query parameters.
/// Parses $filter, $orderby, limit, and cursor parameters.
/// Usage in handlers:
///   async fn list_users(OData(query): OData, /* ... */) { /* use `query` */ }
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
    type Rejection = (StatusCode, String);

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
#[path = "odata_tests.rs"]
mod odata_tests;
