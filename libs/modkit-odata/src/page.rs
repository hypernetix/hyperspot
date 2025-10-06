use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageInfo {
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub limit: u64,
}

#[cfg_attr(feature = "with-utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

impl<T> Page<T> {
    /// Create a new page with items and page info
    pub fn new(items: Vec<T>, page_info: PageInfo) -> Self {
        Self { items, page_info }
    }

    /// Create an empty page with the given limit
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

    /// Map items while preserving page_info (Domain->DTO mapping convenience)
    pub fn map_items<U>(self, mut f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(&mut f).collect(),
            page_info: self.page_info,
        }
    }
}
