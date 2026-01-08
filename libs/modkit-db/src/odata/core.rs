//! `OData` (filters) → `sea_orm::Condition` compiler (AST in, SQL out).
//! Parsing belongs to API/gateway. This module only consumes `modkit_odata::ast::Expr`.

use std::collections::HashMap;

use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{NaiveDate, NaiveTime, Utc};
use modkit_odata::{ast as core, CursorV1, Error as ODataError, ODataOrderBy, ODataQuery, SortDir};
use rust_decimal::Decimal;
use sea_orm::{
    sea_query::{Expr, Order},
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use thiserror::Error;

use crate::odata::{FieldKind, LimitCfg};

/// Type alias for cursor extraction function to reduce type complexity
type CursorExtractor<E> = fn(&<E as EntityTrait>::Model) -> String;

#[derive(Clone)]
pub struct Field<E: EntityTrait> {
    pub col: E::Column,
    pub kind: FieldKind,
    pub to_string_for_cursor: Option<CursorExtractor<E>>,
}

#[derive(Clone)]
#[must_use]
pub struct FieldMap<E: EntityTrait> {
    map: HashMap<String, Field<E>>,
}

impl<E: EntityTrait> Default for FieldMap<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EntityTrait> FieldMap<E> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(mut self, api_name: impl Into<String>, col: E::Column, kind: FieldKind) -> Self {
        self.map.insert(
            api_name.into().to_lowercase(),
            Field {
                col,
                kind,
                to_string_for_cursor: None,
            },
        );
        self
    }

    pub fn insert_with_extractor(
        mut self,
        api_name: impl Into<String>,
        col: E::Column,
        kind: FieldKind,
        to_string_for_cursor: CursorExtractor<E>,
    ) -> Self {
        self.map.insert(
            api_name.into().to_lowercase(),
            Field {
                col,
                kind,
                to_string_for_cursor: Some(to_string_for_cursor),
            },
        );
        self
    }

    pub fn encode_model_key(&self, model: &E::Model, field_name: &str) -> Option<String> {
        let f = self.get(field_name)?;
        f.to_string_for_cursor.map(|f| f(model))
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Field<E>> {
        self.map.get(&name.to_lowercase())
    }
}

#[derive(Debug, Error, Clone)]
pub enum ODataBuildError {
    #[error("unknown field: {0}")]
    UnknownField(String),

    #[error("type mismatch: expected {expected:?}, got {got}")]
    TypeMismatch {
        expected: FieldKind,
        got: &'static str,
    },

    #[error("unsupported operator: {0:?}")]
    UnsupportedOp(core::CompareOperator),

    #[error("unsupported function or args: {0}()")]
    UnsupportedFn(String),

    #[error("IN() list supports only literals")]
    NonLiteralInList,

    #[error("bare identifier not allowed: {0}")]
    BareIdentifier(String),

    #[error("bare literal not allowed")]
    BareLiteral,

    #[error("{0}")]
    Other(&'static str),
}
pub type ODataBuildResult<T> = Result<T, ODataBuildError>;

/* ---------- coercion helpers ---------- */

fn bigdecimal_to_decimal(bd: &BigDecimal) -> ODataBuildResult<Decimal> {
    // Robust conversion: preserve precision via string.
    let s = bd.normalized().to_string();
    Decimal::from_str_exact(&s)
        .or_else(|_| s.parse::<Decimal>())
        .map_err(|_| ODataBuildError::Other("invalid decimal"))
}

fn coerce(kind: FieldKind, v: &core::Value) -> ODataBuildResult<sea_orm::Value> {
    use core::Value as V;
    Ok(match (kind, v) {
        (FieldKind::String, V::String(s)) => sea_orm::Value::String(Some(Box::new(s.clone()))),

        (FieldKind::I64, V::Number(n)) => {
            let i = n.to_i64().ok_or(ODataBuildError::TypeMismatch {
                expected: FieldKind::I64,
                got: "number",
            })?;
            sea_orm::Value::BigInt(Some(i))
        }

        (FieldKind::F64, V::Number(n)) => {
            let f = n.to_f64().ok_or(ODataBuildError::TypeMismatch {
                expected: FieldKind::F64,
                got: "number",
            })?;
            sea_orm::Value::Double(Some(f))
        }

        // Box the Decimal
        (FieldKind::Decimal, V::Number(n)) => {
            sea_orm::Value::Decimal(Some(Box::new(bigdecimal_to_decimal(n)?)))
        }

        (FieldKind::Bool, V::Bool(b)) => sea_orm::Value::Bool(Some(*b)),

        // Box the Uuid
        (FieldKind::Uuid, V::Uuid(u)) => sea_orm::Value::Uuid(Some(Box::new(*u))),

        // Box chrono types
        (FieldKind::DateTimeUtc, V::DateTime(dt)) => {
            sea_orm::Value::ChronoDateTimeUtc(Some(Box::new(*dt)))
        }
        (FieldKind::Date, V::Date(d)) => sea_orm::Value::ChronoDate(Some(Box::new(*d))),
        (FieldKind::Time, V::Time(t)) => sea_orm::Value::ChronoTime(Some(Box::new(*t))),

        (expected, V::Null) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "null",
            })
        }
        (expected, V::String(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "string",
            })
        }
        (expected, V::Number(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "number",
            })
        }
        (expected, V::Bool(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "bool",
            })
        }
        (expected, V::Uuid(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "uuid",
            })
        }
        (expected, V::DateTime(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "datetime",
            })
        }
        (expected, V::Date(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "date",
            })
        }
        (expected, V::Time(_)) => {
            return Err(ODataBuildError::TypeMismatch {
                expected,
                got: "time",
            })
        }
    })
}

fn coerce_many(kind: FieldKind, items: &[core::Expr]) -> ODataBuildResult<Vec<sea_orm::Value>> {
    items
        .iter()
        .map(|e| match e {
            core::Expr::Value(v) => coerce(kind, v),
            _ => Err(ODataBuildError::NonLiteralInList),
        })
        .collect()
}

/* ---------- LIKE helpers ---------- */

fn like_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '%' | '_' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            c => out.push(c),
        }
    }
    out
}
fn like_contains(s: &str) -> String {
    format!("%{}%", like_escape(s))
}
fn like_starts(s: &str) -> String {
    format!("{}%", like_escape(s))
}
fn like_ends(s: &str) -> String {
    format!("%{}", like_escape(s))
}

/* ---------- small guards ---------- */

#[inline]
fn ensure_string_field<E: EntityTrait>(f: &Field<E>, _field_name: &str) -> ODataBuildResult<()> {
    if f.kind != FieldKind::String {
        return Err(ODataBuildError::TypeMismatch {
            expected: FieldKind::String,
            got: "non-string field",
        });
    }
    Ok(())
}

/* ---------- cursor value encoding/decoding ---------- */

/// Parse a cursor value from string based on field kind
pub fn parse_cursor_value(kind: FieldKind, s: &str) -> ODataBuildResult<sea_orm::Value> {
    use sea_orm::Value as V;

    let result = match kind {
        FieldKind::String => V::String(Some(Box::new(s.to_owned()))),
        FieldKind::I64 => {
            let i = s
                .parse::<i64>()
                .map_err(|_| ODataBuildError::Other("invalid i64 in cursor"))?;
            V::BigInt(Some(i))
        }
        FieldKind::F64 => {
            let f = s
                .parse::<f64>()
                .map_err(|_| ODataBuildError::Other("invalid f64 in cursor"))?;
            V::Double(Some(f))
        }
        FieldKind::Bool => {
            let b = s
                .parse::<bool>()
                .map_err(|_| ODataBuildError::Other("invalid bool in cursor"))?;
            V::Bool(Some(b))
        }
        FieldKind::Uuid => {
            let u = s
                .parse::<uuid::Uuid>()
                .map_err(|_| ODataBuildError::Other("invalid uuid in cursor"))?;
            V::Uuid(Some(Box::new(u)))
        }
        FieldKind::DateTimeUtc => {
            let dt = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|_| ODataBuildError::Other("invalid datetime in cursor"))?
                .with_timezone(&Utc);
            V::ChronoDateTimeUtc(Some(Box::new(dt)))
        }
        FieldKind::Date => {
            let d = s
                .parse::<NaiveDate>()
                .map_err(|_| ODataBuildError::Other("invalid date in cursor"))?;
            V::ChronoDate(Some(Box::new(d)))
        }
        FieldKind::Time => {
            let t = s
                .parse::<NaiveTime>()
                .map_err(|_| ODataBuildError::Other("invalid time in cursor"))?;
            V::ChronoTime(Some(Box::new(t)))
        }
        FieldKind::Decimal => {
            let d = s
                .parse::<Decimal>()
                .map_err(|_| ODataBuildError::Other("invalid decimal in cursor"))?;
            V::Decimal(Some(Box::new(d)))
        }
    };

    Ok(result)
}

/* ---------- cursor predicate building ---------- */

/// Build a cursor predicate for pagination.
/// This builds the lexicographic OR-chain condition for cursor-based pagination.
///
/// For backward pagination (cursor.d == "bwd"), the comparison operators are reversed
/// to fetch items before the cursor, but the order remains the same for display consistency.
///
/// # Errors
/// Returns `ODataBuildError` if cursor keys don't match order fields or field resolution fails.
pub fn build_cursor_predicate<E: EntityTrait>(
    cursor: &CursorV1,
    order: &ODataOrderBy,
    fmap: &FieldMap<E>,
) -> ODataBuildResult<Condition>
where
    E::Column: ColumnTrait + Copy,
{
    if cursor.k.len() != order.0.len() {
        return Err(ODataBuildError::Other(
            "cursor keys count mismatch with order fields",
        ));
    }

    // Parse cursor values
    let mut cursor_values = Vec::new();
    for (i, key_str) in cursor.k.iter().enumerate() {
        let order_key = &order.0[i];
        let field = fmap
            .get(&order_key.field)
            .ok_or_else(|| ODataBuildError::UnknownField(order_key.field.clone()))?;
        let value = parse_cursor_value(field.kind, key_str)?;
        cursor_values.push((field, value, order_key.dir));
    }

    // Determine if we're going backward
    let is_backward = cursor.d == "bwd";

    // Build lexicographic condition
    // Forward (fwd):
    //   For ASC: (k0 > v0) OR (k0 = v0 AND k1 > v1) OR ...
    //   For DESC: (k0 < v0) OR (k0 = v0 AND k1 < v1) OR ...
    // Backward (bwd): Reverse the comparisons
    //   For ASC: (k0 < v0) OR (k0 = v0 AND k1 < v1) OR ...
    //   For DESC: (k0 > v0) OR (k0 = v0 AND k1 > v1) OR ...
    let mut main_condition = Condition::any();

    for i in 0..cursor_values.len() {
        let mut prefix_condition = Condition::all();

        // Add equality conditions for all previous fields
        for (field, value, _) in cursor_values.iter().take(i) {
            prefix_condition = prefix_condition.add(Expr::col(field.col).eq(value.clone()));
        }

        // Add the comparison condition for current field
        let (field, value, dir) = &cursor_values[i];
        let comparison = if is_backward {
            // Backward: reverse the comparison
            match dir {
                SortDir::Asc => Expr::col(field.col).lt(value.clone()),
                SortDir::Desc => Expr::col(field.col).gt(value.clone()),
            }
        } else {
            // Forward: normal comparison
            match dir {
                SortDir::Asc => Expr::col(field.col).gt(value.clone()),
                SortDir::Desc => Expr::col(field.col).lt(value.clone()),
            }
        };
        prefix_condition = prefix_condition.add(comparison);

        main_condition = main_condition.add(prefix_condition);
    }

    Ok(main_condition)
}

/* ---------- error mapping helpers ---------- */

/// Resolve a field by name, converting `UnknownField` errors to `InvalidOrderByField`
fn resolve_field<'a, E: EntityTrait>(
    fld_map: &'a FieldMap<E>,
    name: &str,
) -> Result<&'a Field<E>, ODataError> {
    fld_map
        .get(name)
        .ok_or_else(|| ODataError::InvalidOrderByField(name.to_owned()))
}

/* ---------- tiebreaker handling ---------- */

/// Ensure a tiebreaker field is present in the order
pub fn ensure_tiebreaker(order: ODataOrderBy, tiebreaker: &str, dir: SortDir) -> ODataOrderBy {
    order.ensure_tiebreaker(tiebreaker, dir)
}

/* ---------- cursor building ---------- */

/// Build a cursor from a model using the effective order and field map extractors.
///
/// # Errors
/// Returns `ODataError::InvalidOrderByField` if a field cannot be encoded.
pub fn build_cursor_for_model<E: EntityTrait>(
    model: &E::Model,
    order: &ODataOrderBy,
    fmap: &FieldMap<E>,
    primary_dir: SortDir,
    filter_hash: Option<String>,
    direction: &str, // "fwd" or "bwd"
) -> Result<CursorV1, ODataError> {
    let mut k = Vec::with_capacity(order.0.len());
    for key in &order.0 {
        let s = fmap
            .encode_model_key(model, &key.field)
            .ok_or_else(|| ODataError::InvalidOrderByField(key.field.clone()))?;
        k.push(s);
    }
    Ok(CursorV1 {
        k,
        o: primary_dir,
        s: order.to_signed_tokens(),
        f: filter_hash,
        d: direction.to_owned(),
    })
}

/* ---------- Expr (AST) -> Condition ---------- */

/// Convert an `OData` filter expression AST to a `SeaORM` Condition.
///
/// # Errors
/// Returns `ODataBuildError` if the expression contains unknown fields or unsupported operations.
pub fn expr_to_condition<E: EntityTrait>(
    expr: &core::Expr,
    fmap: &FieldMap<E>,
) -> ODataBuildResult<Condition>
where
    E::Column: ColumnTrait + Copy,
{
    use core::CompareOperator as Op;
    use core::Expr as X;

    Ok(match expr {
        X::And(a, b) => {
            let left = expr_to_condition::<E>(a, fmap)?;
            let right = expr_to_condition::<E>(b, fmap)?;
            Condition::all().add(left).add(right) // AND
        }
        X::Or(a, b) => {
            let left = expr_to_condition::<E>(a, fmap)?;
            let right = expr_to_condition::<E>(b, fmap)?;
            Condition::any().add(left).add(right) // OR
        }
        X::Not(x) => {
            let inner = expr_to_condition::<E>(x, fmap)?;
            Condition::all().add(inner).not()
        }

        // Identifier op Value
        X::Compare(lhs, op, rhs) => {
            let (name, rhs_val) = match (&**lhs, &**rhs) {
                (X::Identifier(name), X::Value(val)) => (name, val),
                (X::Identifier(_), X::Identifier(_)) => {
                    return Err(ODataBuildError::Other(
                        "field-to-field comparison is not supported",
                    ))
                }
                _ => return Err(ODataBuildError::Other("unsupported comparison form")),
            };
            let field = fmap
                .get(name)
                .ok_or_else(|| ODataBuildError::UnknownField(name.clone()))?;
            let col = field.col;

            // null handling
            if matches!(rhs_val, core::Value::Null) {
                return Ok(match op {
                    Op::Eq => Condition::all().add(Expr::col(col).is_null()),
                    Op::Ne => Condition::all().add(Expr::col(col).is_not_null()),
                    _ => return Err(ODataBuildError::UnsupportedOp(*op)),
                });
            }

            let value = coerce(field.kind, rhs_val)?;
            let expr = match op {
                Op::Eq => Expr::col(col).eq(value),
                Op::Ne => Expr::col(col).ne(value),
                Op::Gt => Expr::col(col).gt(value),
                Op::Ge => Expr::col(col).gte(value),
                Op::Lt => Expr::col(col).lt(value),
                Op::Le => Expr::col(col).lte(value),
            };
            Condition::all().add(expr)
        }

        // Identifier IN (value, value, ...)
        X::In(l, list) => {
            let X::Identifier(name) = &**l else {
                return Err(ODataBuildError::Other("left side of IN must be a field"));
            };
            let f = fmap
                .get(name)
                .ok_or_else(|| ODataBuildError::UnknownField(name.clone()))?;
            let col = f.col;
            let vals = coerce_many(f.kind, list)?;
            if vals.is_empty() {
                // IN () → always false
                Condition::all().add(Expr::cust("1=0"))
            } else {
                Condition::all().add(Expr::col(col).is_in(vals))
            }
        }

        // Supported functions: contains/startswith/endswith
        X::Function(fname, args) => {
            let n = fname.to_ascii_lowercase();
            match (n.as_str(), args.as_slice()) {
                ("contains", [X::Identifier(name), X::Value(core::Value::String(s))]) => {
                    let f = fmap
                        .get(name)
                        .ok_or_else(|| ODataBuildError::UnknownField(name.clone()))?;
                    ensure_string_field(f, name)?;
                    Condition::all().add(Expr::col(f.col).like(like_contains(s)))
                }
                ("startswith", [X::Identifier(name), X::Value(core::Value::String(s))]) => {
                    let f = fmap
                        .get(name)
                        .ok_or_else(|| ODataBuildError::UnknownField(name.clone()))?;
                    ensure_string_field(f, name)?;
                    Condition::all().add(Expr::col(f.col).like(like_starts(s)))
                }
                ("endswith", [X::Identifier(name), X::Value(core::Value::String(s))]) => {
                    let f = fmap
                        .get(name)
                        .ok_or_else(|| ODataBuildError::UnknownField(name.clone()))?;
                    ensure_string_field(f, name)?;
                    Condition::all().add(Expr::col(f.col).like(like_ends(s)))
                }
                _ => return Err(ODataBuildError::UnsupportedFn(fname.clone())),
            }
        }

        // Leaf forms are not valid WHERE by themselves
        X::Identifier(name) => return Err(ODataBuildError::BareIdentifier(name.clone())),
        X::Value(_) => return Err(ODataBuildError::BareLiteral),
    })
}

/// Apply an optional `OData` filter (via wrapper) to a plain `SeaORM` Select<E>.
///
/// This extension does NOT parse the filter string — it only consumes a parsed AST
/// (`modkit_odata::ast::Expr`) and translates it into a `sea_orm::Condition`.
pub trait ODataExt<E: EntityTrait>: Sized {
    /// Apply `OData` filter to the query.
    ///
    /// # Errors
    /// Returns `ODataBuildError` if the filter contains unknown fields or invalid expressions.
    fn apply_odata_filter(
        self,
        od_query: ODataQuery,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self>;
}

impl<E> ODataExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    fn apply_odata_filter(
        self,
        od_query: ODataQuery,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self> {
        match od_query.filter() {
            Some(ast) => {
                let cond = expr_to_condition::<E>(ast, fld_map)?;
                Ok(self.filter(cond))
            }
            None => Ok(self),
        }
    }
}

/// Extension trait for applying cursor-based pagination
pub trait CursorApplyExt<E: EntityTrait>: Sized {
    /// Apply cursor-based forward pagination.
    ///
    /// # Errors
    /// Returns `ODataBuildError` if cursor validation fails.
    fn apply_cursor_forward(
        self,
        cursor: &CursorV1,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self>;
}

impl<E> CursorApplyExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    fn apply_cursor_forward(
        self,
        cursor: &CursorV1,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self> {
        let cond = build_cursor_predicate(cursor, order, fld_map)?;
        Ok(self.filter(cond))
    }
}

/// Extension trait for applying ordering (legacy version with `ODataBuildError`)
pub trait ODataOrderExt<E: EntityTrait>: Sized {
    /// Apply `OData` ordering to the query.
    ///
    /// # Errors
    /// Returns `ODataBuildError` if an unknown field is referenced.
    fn apply_odata_order(
        self,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self>;
}

impl<E> ODataOrderExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    fn apply_odata_order(
        self,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> ODataBuildResult<Self> {
        let mut query = self;

        for order_key in &order.0 {
            let field = fld_map
                .get(&order_key.field)
                .ok_or_else(|| ODataBuildError::UnknownField(order_key.field.clone()))?;

            let sea_order = match order_key.dir {
                SortDir::Asc => Order::Asc,
                SortDir::Desc => Order::Desc,
            };

            query = query.order_by(field.col, sea_order);
        }

        Ok(query)
    }
}

/// Extension trait for applying ordering with centralized error handling
pub trait ODataOrderPageExt<E: EntityTrait>: Sized {
    /// Apply `OData` ordering with page-level error handling.
    ///
    /// # Errors
    /// Returns `ODataError` if an unknown field is referenced.
    fn apply_odata_order_page(
        self,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> Result<Self, ODataError>;
}

impl<E> ODataOrderPageExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    fn apply_odata_order_page(
        self,
        order: &ODataOrderBy,
        fld_map: &FieldMap<E>,
    ) -> Result<Self, ODataError> {
        let mut query = self;

        for order_key in &order.0 {
            let field = resolve_field(fld_map, &order_key.field)?;

            let sea_order = match order_key.dir {
                SortDir::Asc => Order::Asc,
                SortDir::Desc => Order::Desc,
            };

            query = query.order_by(field.col, sea_order);
        }

        Ok(query)
    }
}

/// Extension trait for applying full `OData` query (filter + cursor + order)
pub trait ODataQueryExt<E: EntityTrait>: Sized {
    /// Apply full `OData` query including filter, cursor, and ordering.
    ///
    /// # Errors
    /// Returns `ODataBuildError` if any part of the query application fails.
    fn apply_odata_query(
        self,
        query: &ODataQuery,
        fld_map: &FieldMap<E>,
        tiebreaker: (&str, SortDir),
    ) -> ODataBuildResult<Self>;
}

impl<E> ODataQueryExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    fn apply_odata_query(
        self,
        query: &ODataQuery,
        fld_map: &FieldMap<E>,
        tiebreaker: (&str, SortDir),
    ) -> ODataBuildResult<Self> {
        let mut select = self;

        if let Some(ast) = query.filter.as_deref() {
            let cond = expr_to_condition::<E>(ast, fld_map)?;
            select = select.filter(cond);
        }

        let effective_order = ensure_tiebreaker(query.order.clone(), tiebreaker.0, tiebreaker.1);

        if let Some(cursor) = &query.cursor {
            select = select.apply_cursor_forward(cursor, &effective_order, fld_map)?;
        }

        select = select.apply_odata_order(&effective_order, fld_map)?;

        Ok(select)
    }
}

/* ---------- pagination combiner ---------- */

// Use unified pagination types from modkit-odata
pub use modkit_odata::{Page, PageInfo};

// Note: LimitCfg is imported at the top and re-exported from odata/mod.rs

fn clamp_limit(req: Option<u64>, cfg: LimitCfg) -> u64 {
    let mut l = req.unwrap_or(cfg.default);
    if l == 0 {
        l = 1;
    }
    if l > cfg.max {
        l = cfg.max;
    }
    l
}

/// One-shot pagination combiner that handles filter → cursor predicate → order → overfetch/trim → build cursors.
///
/// # Errors
/// Returns `ODataError` if filter application, cursor validation, or database query fails.
pub async fn paginate_with_odata<E, D, F, C>(
    select: sea_orm::Select<E>,
    conn: &C,
    q: &ODataQuery,
    fmap: &FieldMap<E>,
    tiebreaker: (&str, SortDir), // e.g. ("id", SortDir::Desc)
    limit_cfg: LimitCfg,         // e.g. { default: 25, max: 1000 }
    model_to_domain: F,
) -> Result<Page<D>, ODataError>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
    F: Fn(E::Model) -> D + Copy,
    C: ConnectionTrait + Send + Sync,
{
    let limit = clamp_limit(q.limit, limit_cfg);
    let fetch = limit + 1;

    // Effective order derivation based on new policy
    let effective_order = if let Some(cur) = &q.cursor {
        // Derive order from the cursor's signed tokens
        modkit_odata::ODataOrderBy::from_signed_tokens(&cur.s)
            .map_err(|_| ODataError::InvalidCursor)?
    } else {
        // Use client order; ensure tiebreaker
        q.order
            .clone()
            .ensure_tiebreaker(tiebreaker.0, tiebreaker.1)
    };

    // Validate cursor consistency (filter hash only) if cursor present
    if let Some(cur) = &q.cursor {
        // Only filter hash validation is necessary now
        if let (Some(h), Some(cf)) = (q.filter_hash.as_deref(), cur.f.as_deref()) {
            if h != cf {
                return Err(ODataError::FilterMismatch);
            }
        }
    }

    // Compose: filter → cursor predicate → order; apply limit+1 at the end
    let mut s = select;

    // Apply filter
    if let Some(ast) = q.filter.as_deref() {
        s = s.filter(
            expr_to_condition::<E>(ast, fmap)
                .map_err(|e| ODataError::InvalidFilter(e.to_string()))?,
        );
    }

    // Check if we're paginating backward
    let is_backward = q.cursor.as_ref().is_some_and(|c| c.d == "bwd");

    // Apply cursor if present
    if let Some(cursor) = &q.cursor {
        s = s.filter(
            build_cursor_predicate(cursor, &effective_order, fmap)
                .map_err(|_| ODataError::InvalidCursor)?,
        );
    }

    // Apply order (reverse it for backward pagination)
    let query_order = if is_backward {
        effective_order.clone().reverse_directions()
    } else {
        effective_order.clone()
    };
    s = s.apply_odata_order_page(&query_order, fmap)?;

    // Apply limit
    s = s.limit(fetch);

    #[allow(clippy::disallowed_methods)]
    let mut rows = s
        .all(conn)
        .await
        .map_err(|e| ODataError::Db(e.to_string()))?;

    let has_more = (rows.len() as u64) > limit;

    // For backward pagination with reversed ORDER BY:
    // - DB returns items in opposite order
    // - We fetch limit+1 to detect has_more
    // - We need to: 1) trim, 2) reverse back to original order
    if is_backward {
        // Remove the extra item (furthest back in time, which is at the END after reversed query)
        if has_more {
            rows.pop();
        }
        // Reverse to restore original display order
        rows.reverse();
    } else if has_more {
        // Forward pagination: just truncate the end
        rows.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
    }

    // Build cursors
    // After all the reversals, rows are in the display order (DESC)
    // - rows.first() = newest item
    // - rows.last() = oldest item
    //
    // For backward pagination:
    //   - has_more means "more items backward" (older)
    //   - next_cursor should always be present (we came from forward)
    //   - prev_cursor based on has_more
    // For forward pagination:
    //   - has_more means "more items forward" (older in DESC)
    //   - next_cursor based on has_more
    //   - prev_cursor always present (unless at start)

    let next_cursor = if is_backward {
        // Going backward: always have items forward (unless this was the initial query)
        // Build cursor from last item to go forward
        build_cursor(&rows, &effective_order, fmap, tiebreaker, q, true, "fwd")?
    } else if has_more {
        // Going forward: only have more if has_more is true
        build_cursor(&rows, &effective_order, fmap, tiebreaker, q, true, "fwd")?
    } else {
        None
    };

    let prev_cursor = if is_backward {
        // Going backward: only have more backward if has_more is true
        if has_more {
            build_cursor(&rows, &effective_order, fmap, tiebreaker, q, false, "bwd")?
        } else {
            None
        }
    } else if q.cursor.is_some() {
        // Going forward: have items backward only if this is NOT the initial query
        // If q.cursor is None, we're at the start of the dataset
        build_cursor(&rows, &effective_order, fmap, tiebreaker, q, false, "bwd")?
    } else {
        None
    };

    let items = rows.into_iter().map(model_to_domain).collect();

    Ok(Page {
        items,
        page_info: PageInfo {
            next_cursor,
            prev_cursor,
            limit,
        },
    })
}

fn build_cursor<E: EntityTrait>(
    rows: &[E::Model],
    effective_order: &ODataOrderBy,
    fmap: &FieldMap<E>,
    tiebreaker: (&str, SortDir),
    q: &ODataQuery,
    last: bool,
    direction: &str,
) -> Result<Option<String>, ODataError> {
    if last { rows.last() } else { rows.first() }
        .map(|m| {
            build_cursor_for_model::<E>(
                m,
                effective_order,
                fmap,
                tiebreaker.1,
                q.filter_hash.clone(),
                direction,
            )
            .and_then(|c| c.encode().map_err(|_| ODataError::InvalidCursor))
        })
        .transpose()
}
