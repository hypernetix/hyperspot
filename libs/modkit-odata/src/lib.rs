#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
pub mod builder;
pub mod errors;
pub mod filter;
pub mod limits;
pub mod page;
pub mod pagination;
pub mod problem_mapping;
pub mod schema;

pub use builder::QueryBuilder;
pub use limits::ODataLimits;
pub use page::{Page, PageInfo};
pub use pagination::{normalize_filter_for_hash, short_filter_hash};
pub use schema::{FieldRef, Schema};

pub mod ast {
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
    use uuid::Uuid;

    #[derive(Clone, Debug)]
    pub enum Expr {
        And(Box<Expr>, Box<Expr>),
        Or(Box<Expr>, Box<Expr>),
        Not(Box<Expr>),
        Compare(Box<Expr>, CompareOperator, Box<Expr>),
        In(Box<Expr>, Vec<Expr>),
        Function(String, Vec<Expr>),
        Identifier(String),
        Value(Value),
    }

    impl Expr {
        /// Combine two expressions with AND: `expr1 and expr2`
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let filter = ID.eq(user_id).and(NAME.contains("john"));
        /// ```
        #[must_use]
        pub fn and(self, other: Expr) -> Expr {
            Expr::And(Box::new(self), Box::new(other))
        }

        /// Combine two expressions with OR: `expr1 or expr2`
        #[must_use]
        pub fn or(self, other: Expr) -> Expr {
            Expr::Or(Box::new(self), Box::new(other))
        }

        /// Negate an expression: `not expr`
        #[must_use]
        #[allow(clippy::should_implement_trait)]
        pub fn not(self) -> Expr {
            !self
        }
    }

    impl std::ops::Not for Expr {
        type Output = Expr;

        fn not(self) -> Self::Output {
            Expr::Not(Box::new(self))
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CompareOperator {
        Eq,
        Ne,
        Gt,
        Ge,
        Lt,
        Le,
    }

    #[derive(Clone, Debug)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(BigDecimal),
        Uuid(Uuid),
        DateTime(DateTime<Utc>),
        Date(NaiveDate),
        Time(NaiveTime),
        String(String),
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Null => write!(f, "null"),
                Value::Bool(_) => write!(f, "bool"),
                Value::Number(_) => write!(f, "number"),
                Value::Uuid(_) => write!(f, "uuid"),
                Value::DateTime(_) => write!(f, "datetime"),
                Value::Date(_) => write!(f, "date"),
                Value::Time(_) => write!(f, "time"),
                Value::String(_) => write!(f, "string"),
            }
        }
    }
}

// Ordering primitives
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SortDir {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl SortDir {
    /// Reverse the sort direction (Asc <-> Desc)
    #[must_use]
    pub fn reverse(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrderKey {
    pub field: String,
    pub dir: SortDir,
}

#[derive(Clone, Debug, Default)]
#[must_use]
pub struct ODataOrderBy(pub Vec<OrderKey>);

impl ODataOrderBy {
    pub fn empty() -> Self {
        Self(vec![])
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Render as "+f1,-f2" for cursor.s
    #[must_use]
    pub fn to_signed_tokens(&self) -> String {
        self.0
            .iter()
            .map(|k| {
                if matches!(k.dir, SortDir::Asc) {
                    format!("+{}", k.field)
                } else {
                    format!("-{}", k.field)
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Parse signed tokens back to `ODataOrderBy` (e.g. "+a,-b" -> `ODataOrderBy`)
    /// Returns Error for stricter validation used in cursor processing
    ///
    /// # Errors
    /// Returns `Error::InvalidOrderByField` if the input is empty or contains invalid field names.
    pub fn from_signed_tokens(signed: &str) -> Result<Self, Error> {
        let mut out = Vec::new();
        for seg in signed.split(',') {
            let seg = seg.trim();
            if seg.is_empty() {
                continue;
            }
            let (dir, name) = match seg.as_bytes()[0] {
                b'+' => (SortDir::Asc, &seg[1..]),
                b'-' => (SortDir::Desc, &seg[1..]),
                _ => (SortDir::Asc, seg), // default '+'
            };
            if name.is_empty() {
                return Err(Error::InvalidOrderByField(seg.to_owned()));
            }
            out.push(OrderKey {
                field: name.to_owned(),
                dir,
            });
        }
        if out.is_empty() {
            return Err(Error::InvalidOrderByField("empty order".into()));
        }
        Ok(ODataOrderBy(out))
    }

    /// Check equality against signed token list (e.g. "+a,-b")
    #[must_use]
    pub fn equals_signed_tokens(&self, signed: &str) -> bool {
        let parse = |t: &str| -> Option<(String, SortDir)> {
            let t = t.trim();
            if t.is_empty() {
                return None;
            }
            let (dir, name) = match t.as_bytes()[0] {
                b'+' => (SortDir::Asc, &t[1..]),
                b'-' => (SortDir::Desc, &t[1..]),
                _ => (SortDir::Asc, t),
            };
            if name.is_empty() {
                return None;
            }
            Some((name.to_owned(), dir))
        };
        let theirs: Vec<_> = signed.split(',').filter_map(parse).collect();
        if theirs.len() != self.0.len() {
            return false;
        }
        self.0
            .iter()
            .zip(theirs.iter())
            .all(|(a, (n, d))| a.field == *n && a.dir == *d)
    }

    /// Append tiebreaker if missing
    pub fn ensure_tiebreaker(mut self, tiebreaker: &str, dir: SortDir) -> Self {
        if !self.0.iter().any(|k| k.field == tiebreaker) {
            self.0.push(OrderKey {
                field: tiebreaker.to_owned(),
                dir,
            });
        }
        self
    }

    /// Reverse all sort directions (for backward pagination)
    pub fn reverse_directions(mut self) -> Self {
        for key in &mut self.0 {
            key.dir = key.dir.reverse();
        }
        self
    }
}

// Display trait for human-readable orderby representation
impl std::fmt::Display for ODataOrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "(none)");
        }

        let formatted: Vec<String> = self
            .0
            .iter()
            .map(|key| {
                let dir_str = match key.dir {
                    SortDir::Asc => "asc",
                    SortDir::Desc => "desc",
                };
                format!("{} {}", key.field, dir_str)
            })
            .collect();

        write!(f, "{}", formatted.join(", "))
    }
}

/// Unified error type for all `OData` operations
///
/// This centralizes all OData-related errors including parsing, validation,
/// pagination, and cursor operations into a single error type using thiserror.
///
/// ## HTTP Mapping
///
/// These errors map to RFC 9457 Problem responses via the catalog in `modkit`:
/// - `InvalidFilter` → 422 `gts...~hx.odata.errors.invalid_filter.v1`
/// - `InvalidOrderByField` → 422 `gts...~hx.odata.errors.invalid_orderby.v1`
/// - Cursor errors → 422 `gts...~hx.odata.errors.invalid_cursor.v1`
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    // Filter parsing and validation errors
    #[error("invalid $filter: {0}")]
    InvalidFilter(String),

    // OrderBy parsing and validation errors
    #[error("unsupported $orderby field: {0}")]
    InvalidOrderByField(String),

    // Pagination and cursor errors
    #[error("ORDER_MISMATCH")]
    OrderMismatch,

    #[error("FILTER_MISMATCH")]
    FilterMismatch,

    #[error("INVALID_CURSOR")]
    InvalidCursor,

    #[error("INVALID_LIMIT")]
    InvalidLimit,

    #[error("ORDER_WITH_CURSOR")]
    OrderWithCursor,

    // Cursor parsing errors (previously CursorError variants)
    #[error("invalid cursor: invalid base64url encoding")]
    CursorInvalidBase64,

    #[error("invalid cursor: malformed JSON")]
    CursorInvalidJson,

    #[error("invalid cursor: unsupported version")]
    CursorInvalidVersion,

    #[error("invalid cursor: empty or invalid keys")]
    CursorInvalidKeys,

    #[error("invalid cursor: empty or invalid fields")]
    CursorInvalidFields,

    #[error("invalid cursor: invalid sort direction")]
    CursorInvalidDirection,

    // Database and low-level errors
    #[error("database error: {0}")]
    Db(String),

    // Configuration errors
    #[error("OData parsing unavailable: {0}")]
    ParsingUnavailable(&'static str),
}

/// Validate cursor consistency against effective order and filter hash.
///
/// # Errors
/// Returns `Error::OrderMismatch` if the cursor's sort order doesn't match the effective order.
/// Returns `Error::FilterMismatch` if the cursor's filter hash doesn't match the effective filter.
pub fn validate_cursor_against(
    cursor: &CursorV1,
    effective_order: &ODataOrderBy,
    effective_filter_hash: Option<&str>,
) -> Result<(), Error> {
    if !effective_order.equals_signed_tokens(&cursor.s) {
        return Err(Error::OrderMismatch);
    }
    if let (Some(h), Some(cf)) = (effective_filter_hash, cursor.f.as_deref()) {
        if h != cf {
            return Err(Error::FilterMismatch);
        }
    }
    Ok(())
}

// Cursor v1
#[derive(Clone, Debug)]
pub struct CursorV1 {
    pub k: Vec<String>,
    pub o: SortDir,
    pub s: String,
    pub f: Option<String>,
    pub d: String, // Direction: "fwd" (forward) or "bwd" (backward)
}

impl CursorV1 {
    /// Encode cursor to a base64url string.
    ///
    /// # Errors
    /// Returns a JSON serialization error if encoding fails.
    pub fn encode(&self) -> serde_json::Result<String> {
        #[derive(serde::Serialize)]
        struct Wire<'a> {
            v: u8,
            k: &'a [String],
            o: &'a str,
            s: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            f: &'a Option<String>,
            d: &'a str,
        }
        let o = match self.o {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        };
        let w = Wire {
            v: 1,
            k: &self.k,
            o,
            s: &self.s,
            f: &self.f,
            d: &self.d,
        };
        serde_json::to_vec(&w).map(|x| base64_url::encode(&x))
    }

    /// Decode cursor from base64url token.
    ///
    /// # Errors
    /// Returns `Error::CursorInvalidBase64` if base64 decoding fails.
    /// Returns `Error::CursorInvalidJson` if JSON parsing fails.
    /// Returns `Error::CursorInvalidVersion` if the version is unsupported.
    /// Returns `Error::CursorInvalidDirection` if the direction field is invalid.
    pub fn decode(token: &str) -> Result<Self, Error> {
        #[derive(serde::Deserialize)]
        struct Wire {
            v: u8,
            k: Vec<String>,
            o: String,
            s: String,
            #[serde(default)]
            f: Option<String>,
            #[serde(default = "default_direction")]
            d: String,
        }

        fn default_direction() -> String {
            "fwd".to_owned()
        }

        let bytes = base64_url::decode(token).map_err(|_| Error::CursorInvalidBase64)?;
        let w: Wire = serde_json::from_slice(&bytes).map_err(|_| Error::CursorInvalidJson)?;
        if w.v != 1 {
            return Err(Error::CursorInvalidVersion);
        }
        let o = match w.o.as_str() {
            "asc" => SortDir::Asc,
            "desc" => SortDir::Desc,
            _ => return Err(Error::CursorInvalidDirection),
        };
        if w.k.is_empty() {
            return Err(Error::CursorInvalidKeys);
        }
        if w.s.trim().is_empty() {
            return Err(Error::CursorInvalidFields);
        }
        // Validate direction
        if w.d != "fwd" && w.d != "bwd" {
            return Err(Error::CursorInvalidDirection);
        }
        Ok(CursorV1 {
            k: w.k,
            o,
            s: w.s,
            f: w.f,
            d: w.d,
        })
    }
}

// base64url helpers (no padding)
mod base64_url {
    use base64::Engine;

    pub fn encode(bytes: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s)
    }
}

// The unified ODataQuery struct as single source of truth
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct ODataQuery {
    pub filter: Option<Box<ast::Expr>>,
    pub order: ODataOrderBy,
    pub limit: Option<u64>,
    pub cursor: Option<CursorV1>,
    pub filter_hash: Option<String>,
    pub select: Option<Vec<String>>,
}

impl ODataQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_filter(mut self, expr: ast::Expr) -> Self {
        self.filter = Some(Box::new(expr));
        self
    }

    pub fn with_order(mut self, order: ODataOrderBy) -> Self {
        self.order = order;
        self
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_cursor(mut self, cursor: CursorV1) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn with_filter_hash(mut self, hash: String) -> Self {
        self.filter_hash = Some(hash);
        self
    }

    pub fn with_select(mut self, fields: Vec<String>) -> Self {
        self.select = Some(fields);
        self
    }

    /// Get filter as AST
    #[must_use]
    pub fn filter(&self) -> Option<&ast::Expr> {
        self.filter.as_deref()
    }

    /// Check if filter is present
    #[must_use]
    pub fn has_filter(&self) -> bool {
        self.filter.is_some()
    }

    /// Extract filter into AST
    #[must_use]
    pub fn into_filter(self) -> Option<ast::Expr> {
        self.filter.map(|b| *b)
    }

    /// Check if field selection is present
    #[must_use]
    pub fn has_select(&self) -> bool {
        self.select.is_some()
    }

    /// Get selected fields
    #[must_use]
    pub fn selected_fields(&self) -> Option<&[String]> {
        self.select.as_deref()
    }
}

impl From<Option<ast::Expr>> for ODataQuery {
    fn from(opt: Option<ast::Expr>) -> Self {
        match opt {
            Some(e) => Self::default().with_filter(e),
            None => Self::default(),
        }
    }
}

mod tests;

#[cfg(feature = "with-odata-params")]
mod convert_odata_params {
    use super::ast::{CompareOperator, Expr, Value};
    use odata_params::filters as od;

    impl From<od::CompareOperator> for CompareOperator {
        fn from(op: od::CompareOperator) -> Self {
            use od::CompareOperator::{
                Equal, GreaterOrEqual, GreaterThan, LessOrEqual, LessThan, NotEqual,
            };
            match op {
                Equal => CompareOperator::Eq,
                NotEqual => CompareOperator::Ne,
                GreaterThan => CompareOperator::Gt,
                GreaterOrEqual => CompareOperator::Ge,
                LessThan => CompareOperator::Lt,
                LessOrEqual => CompareOperator::Le,
            }
        }
    }

    impl From<od::Value> for Value {
        fn from(v: od::Value) -> Self {
            match v {
                od::Value::Null => Value::Null,
                od::Value::Bool(b) => Value::Bool(b),
                od::Value::Number(n) => Value::Number(n),
                od::Value::Uuid(u) => Value::Uuid(u),
                od::Value::DateTime(dt) => Value::DateTime(dt),
                od::Value::Date(d) => Value::Date(d),
                od::Value::Time(t) => Value::Time(t),
                od::Value::String(s) => Value::String(s),
            }
        }
    }

    impl From<od::Expr> for Expr {
        fn from(e: od::Expr) -> Self {
            use od::Expr::{And, Compare, Function, Identifier, In, Not, Or, Value};
            match e {
                And(a, b) => Expr::And(Box::new((*a).into()), Box::new((*b).into())),
                Or(a, b) => Expr::Or(Box::new((*a).into()), Box::new((*b).into())),
                Not(x) => Expr::Not(Box::new((*x).into())),
                Compare(l, op, r) => {
                    Expr::Compare(Box::new((*l).into()), op.into(), Box::new((*r).into()))
                }
                In(l, list) => Expr::In(
                    Box::new((*l).into()),
                    list.into_iter().map(Into::into).collect(),
                ),
                Function(n, args) => Expr::Function(n, args.into_iter().map(Into::into).collect()),
                Identifier(s) => Expr::Identifier(s),
                Value(v) => Expr::Value(v.into()),
            }
        }
    }
}

/// Result of parsing a filter string, including both the AST and complexity metadata.
#[derive(Clone, Debug)]
pub struct ParsedFilter {
    expr: ast::Expr,
    node_count: usize,
}

impl ParsedFilter {
    /// Get a reference to the parsed expression
    #[must_use]
    pub fn as_expr(&self) -> &ast::Expr {
        &self.expr
    }

    /// Consume and extract the parsed expression
    #[must_use]
    pub fn into_expr(self) -> ast::Expr {
        self.expr
    }

    /// Get the AST node count for budget enforcement
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

/// Parse a raw $filter string into internal AST with complexity metadata.
///
/// This function encapsulates the parsing logic and node counting,
/// abstracting away the underlying `odata_params` dependency.
///
/// # Errors
/// - `Error::InvalidFilter` if the filter string is malformed or parsing fails
/// - `Error::ParsingUnavailable` if the `with-odata-params` feature is disabled
///
/// # Example
/// ```ignore
/// let result = parse_filter_string("name eq 'John' and age gt 18")?;
/// if result.node_count() > MAX_NODES {
///     return Err(Error::InvalidFilter("too complex".into()));
/// }
/// ```
#[cfg(feature = "with-odata-params")]
pub fn parse_filter_string(raw: &str) -> Result<ParsedFilter, Error> {
    use odata_params::filters as od;

    /// Count nodes in `odata_params` AST for complexity budget enforcement.
    fn count_ast_nodes(e: &od::Expr) -> usize {
        use od::Expr::{And, Compare, Function, Identifier, In, Not, Or, Value};
        match e {
            Value(_) | Identifier(_) => 1,
            Not(x) => 1 + count_ast_nodes(x),
            And(a, b) | Or(a, b) | Compare(a, _, b) => 1 + count_ast_nodes(a) + count_ast_nodes(b),
            In(a, list) => 1 + count_ast_nodes(a) + list.iter().map(count_ast_nodes).sum::<usize>(),
            Function(_, args) => 1 + args.iter().map(count_ast_nodes).sum::<usize>(),
        }
    }

    let ast_src = od::parse_str(raw).map_err(|e| Error::InvalidFilter(format!("{e:?}")))?;

    let node_count = count_ast_nodes(&ast_src);
    let expr: ast::Expr = ast_src.into();

    Ok(ParsedFilter { expr, node_count })
}

/// Parse `OData` filter string.
///
/// This stub is compiled when the `with-odata-params` feature is disabled.
///
/// # Errors
///
/// Always returns `Error::ParsingUnavailable` because `OData` parsing
/// support is not enabled in this build.
#[cfg(not(feature = "with-odata-params"))]
pub fn parse_filter_string(_raw: &str) -> Result<ParsedFilter, Error> {
    Err(Error::ParsingUnavailable(
        "OData filter parsing requires 'with-odata-params' feature",
    ))
}
