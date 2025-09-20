use serde::{Deserialize, Serialize};

/// Page envelope with items and pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub limit: u64,
}

/// Limit configuration for pagination
#[derive(Debug, Clone)]
pub struct LimitCfg {
    pub default: u64,
    pub max: u64,
}

impl Default for LimitCfg {
    fn default() -> Self {
        Self {
            default: 50,
            max: 1000,
        }
    }
}

/// Calculate effective limit based on query and configuration
pub fn effective_limit(query_limit: Option<u64>, cfg: &LimitCfg) -> u64 {
    match query_limit {
        None => cfg.default,
        Some(0) => 1,                      // 0 -> 1
        Some(n) if n > cfg.max => cfg.max, // >max -> max
        Some(n) => n,
    }
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page_info: PageInfo) -> Self {
        Self { items, page_info }
    }

    pub fn empty(limit: u64) -> Self {
        Self {
            items: Vec::new(),
            page_info: PageInfo {
                next_cursor: None,
                prev_cursor: None,
                limit,
            },
        }
    }
}
