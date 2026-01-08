//! Cursor-based pagination with Stream API
//!
//! This module provides a reusable cursor-based pager that converts a page-fetching function
//! into a Stream of pages or items, hiding cursor management from SDK users.
//!
//! # Example
//!
//! ```rust,ignore
//! use modkit_sdk::pager::{CursorPager, PagerError};
//! use modkit_sdk::odata::QueryBuilder;
//! use futures_util::StreamExt;
//!
//! // Stream of pages
//! let pages = QueryBuilder::<UserSchema>::new()
//!     .filter(NAME.contains("john"))
//!     .page_size(50)
//!     .pages_stream(|query| async move {
//!         client.list_users(query).await
//!     });
//!
//! // Stream of items
//! let items = QueryBuilder::<UserSchema>::new()
//!     .filter(NAME.contains("john"))
//!     .page_size(50)
//!     .items_stream(|query| async move {
//!         client.list_users(query).await
//!     });
//!
//! // Consume the stream
//! while let Some(result) = items.next().await {
//!     match result {
//!         Ok(user) => println!("User: {:?}", user),
//!         Err(PagerError::Fetch(e)) => eprintln!("Fetch error: {}", e),
//!         Err(PagerError::InvalidCursor(c)) => eprintln!("Invalid cursor: {}", c),
//!     }
//! }
//! ```

use futures_core::Stream;
use modkit_odata::{ODataQuery, Page};
use pin_project_lite::pin_project;
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Error type for pagination operations.
///
/// This enum wraps both fetcher errors and cursor decoding failures,
/// ensuring that invalid cursors are not silently ignored.
#[derive(Debug)]
pub enum PagerError<E> {
    /// Error from the fetcher function.
    Fetch(E),
    /// Invalid cursor string that failed to decode.
    InvalidCursor(String),
}

impl<E: fmt::Display> fmt::Display for PagerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(e) => write!(f, "Fetch error: {e}"),
            Self::InvalidCursor(cursor) => write!(f, "Invalid cursor: {cursor}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PagerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fetch(e) => Some(e),
            Self::InvalidCursor(_) => None,
        }
    }
}

pin_project! {
    /// A cursor-based pager that implements `Stream` for paginated items.
    ///
    /// This pager manages cursor state internally and fetches pages on-demand,
    /// yielding individual items from the stream.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The item type
    /// * `E` - The error type
    /// * `F` - The fetcher function type
    /// * `Fut` - The future returned by the fetcher
    pub struct CursorPager<T, E, F, Fut>
    where
        F: FnMut(ODataQuery) -> Fut,
        Fut: Future<Output = Result<Page<T>, E>>,
    {
        base_query: ODataQuery,
        next_cursor: Option<String>,
        buffer: VecDeque<T>,
        done: bool,
        fetcher: F,
        #[pin]
        current_fetch: Option<Fut>,
    }
}

impl<T, E, F, Fut> CursorPager<T, E, F, Fut>
where
    F: FnMut(ODataQuery) -> Fut,
    Fut: Future<Output = Result<Page<T>, E>>,
{
    /// Create a new cursor pager with the given base query and fetcher function.
    ///
    /// # Arguments
    ///
    /// * `base_query` - The base `OData` query (without cursor)
    /// * `fetcher` - Function that fetches a page given an `ODataQuery`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pager = CursorPager::new(query, |q| async move {
    ///     client.list_users(q).await
    /// });
    /// ```
    pub fn new(base_query: ODataQuery, fetcher: F) -> Self {
        Self {
            base_query,
            next_cursor: None,
            buffer: VecDeque::new(),
            done: false,
            fetcher,
            current_fetch: None,
        }
    }
}

impl<T, E, F, Fut> Stream for CursorPager<T, E, F, Fut>
where
    F: FnMut(ODataQuery) -> Fut,
    Fut: Future<Output = Result<Page<T>, E>>,
{
    type Item = Result<T, PagerError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(item) = this.buffer.pop_front() {
                return Poll::Ready(Some(Ok(item)));
            }

            if *this.done {
                return Poll::Ready(None);
            }

            if let Some(fut) = this.current_fetch.as_mut().as_pin_mut() {
                match fut.poll(cx) {
                    Poll::Ready(Ok(page)) => {
                        this.current_fetch.set(None);

                        this.next_cursor.clone_from(&page.page_info.next_cursor);

                        if this.next_cursor.is_none() {
                            *this.done = true;
                        }

                        this.buffer.extend(page.items);

                        continue;
                    }
                    Poll::Ready(Err(e)) => {
                        this.current_fetch.set(None);
                        *this.done = true;
                        return Poll::Ready(Some(Err(PagerError::Fetch(e))));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Allocation strategy: base_query cloned once per page fetch.
            // Filter AST is built once in QueryBuilder and reused here.
            let mut query = this.base_query.clone();
            if let Some(cursor_str) = this.next_cursor.as_ref() {
                if let Ok(cursor) = modkit_odata::CursorV1::decode(cursor_str) {
                    query = query.with_cursor(cursor);
                } else {
                    *this.done = true;
                    return Poll::Ready(Some(Err(PagerError::InvalidCursor(cursor_str.clone()))));
                }
            }

            let fut = (this.fetcher)(query);
            this.current_fetch.set(Some(fut));
        }
    }
}

pin_project! {
    /// A cursor-based pager that implements `Stream` for pages.
    ///
    /// This pager yields entire pages instead of individual items.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The item type
    /// * `E` - The error type
    /// * `F` - The fetcher function type
    /// * `Fut` - The future returned by the fetcher
    pub struct PagesPager<T, E, F, Fut>
    where
        F: FnMut(ODataQuery) -> Fut,
        Fut: Future<Output = Result<Page<T>, E>>,
    {
        base_query: ODataQuery,
        next_cursor: Option<String>,
        done: bool,
        fetcher: F,
        #[pin]
        current_fetch: Option<Fut>,
    }
}

impl<T, E, F, Fut> PagesPager<T, E, F, Fut>
where
    F: FnMut(ODataQuery) -> Fut,
    Fut: Future<Output = Result<Page<T>, E>>,
{
    /// Create a new pages pager with the given base query and fetcher function.
    ///
    /// # Arguments
    ///
    /// * `base_query` - The base `OData` query (without cursor)
    /// * `fetcher` - Function that fetches a page given an `ODataQuery`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pager = PagesPager::new(query, |q| async move {
    ///     client.list_users(q).await
    /// });
    /// ```
    pub fn new(base_query: ODataQuery, fetcher: F) -> Self {
        Self {
            base_query,
            next_cursor: None,
            done: false,
            fetcher,
            current_fetch: None,
        }
    }
}

impl<T, E, F, Fut> Stream for PagesPager<T, E, F, Fut>
where
    F: FnMut(ODataQuery) -> Fut,
    Fut: Future<Output = Result<Page<T>, E>>,
{
    type Item = Result<Page<T>, PagerError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if *this.done {
                return Poll::Ready(None);
            }

            if let Some(fut) = this.current_fetch.as_mut().as_pin_mut() {
                match fut.poll(cx) {
                    Poll::Ready(Ok(page)) => {
                        this.current_fetch.set(None);

                        this.next_cursor.clone_from(&page.page_info.next_cursor);

                        if this.next_cursor.is_none() {
                            *this.done = true;
                        }

                        return Poll::Ready(Some(Ok(page)));
                    }
                    Poll::Ready(Err(e)) => {
                        this.current_fetch.set(None);
                        *this.done = true;
                        return Poll::Ready(Some(Err(PagerError::Fetch(e))));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Allocation strategy: base_query cloned once per page fetch.
            // Filter AST is built once in QueryBuilder and reused here.
            let mut query = this.base_query.clone();
            if let Some(cursor_str) = this.next_cursor.as_ref() {
                if let Ok(cursor) = modkit_odata::CursorV1::decode(cursor_str) {
                    query = query.with_cursor(cursor);
                } else {
                    *this.done = true;
                    return Poll::Ready(Some(Err(PagerError::InvalidCursor(cursor_str.clone()))));
                }
            }

            let fut = (this.fetcher)(query);
            this.current_fetch.set(Some(fut));

            // Poll the newly-installed future immediately so it can register the current waker
            // naturally, avoiding a manual `wake_by_ref()` and the associated spurious wakeup.
        }
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use modkit_odata::PageInfo;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FakeError(String);

    #[derive(Clone)]
    struct FakeFetcher {
        pages: Arc<[Page<User>]>,
        call_count: Arc<Mutex<usize>>,
    }

    impl FakeFetcher {
        fn new(pages: Vec<Page<User>>) -> Self {
            Self {
                pages: Arc::from(pages),
                call_count: Arc::new(Mutex::new(0)),
            }
        }

        fn fetch(&self, _query: ODataQuery) -> Result<Page<User>, FakeError> {
            let mut count = self.call_count.lock().unwrap();
            if *count >= self.pages.len() {
                return Err(FakeError("No more pages".to_owned()));
            }
            let page = self.pages[*count].clone();
            *count += 1;
            Ok(page)
        }
    }

    #[tokio::test]
    async fn test_cursor_pager_two_pages() {
        use modkit_odata::{CursorV1, SortDir};

        let cursor = CursorV1 {
            k: vec!["2".to_owned()],
            o: SortDir::Asc,
            s: "filter_hash".to_owned(),
            f: Some("filter_hash".to_owned()),
            d: "fwd".to_owned(),
        };
        let encoded_cursor = cursor.encode().unwrap();

        let page1 = Page::new(
            vec![
                User {
                    id: 1,
                    name: "Alice".to_owned(),
                },
                User {
                    id: 2,
                    name: "Bob".to_owned(),
                },
            ],
            PageInfo {
                next_cursor: Some(encoded_cursor.clone()),
                prev_cursor: None,
                limit: 2,
            },
        );

        let page2 = Page::new(
            vec![
                User {
                    id: 3,
                    name: "Charlie".to_owned(),
                },
                User {
                    id: 4,
                    name: "Diana".to_owned(),
                },
            ],
            PageInfo {
                next_cursor: None,
                prev_cursor: Some(encoded_cursor),
                limit: 2,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1, page2]);
        let query = ODataQuery::new().with_limit(2);

        let pager = CursorPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let items: Vec<Result<User, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(items.len(), 4);
        assert!(items.iter().all(Result::is_ok));

        let users: Vec<User> = items.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[1].name, "Bob");
        assert_eq!(users[2].name, "Charlie");
        assert_eq!(users[3].name, "Diana");
    }

    #[tokio::test]
    async fn test_cursor_pager_empty_page() {
        let page = Page::new(
            vec![],
            PageInfo {
                next_cursor: None,
                prev_cursor: None,
                limit: 10,
            },
        );

        let fetcher = FakeFetcher::new(vec![page]);
        let query = ODataQuery::new().with_limit(10);

        let pager = CursorPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let items: Vec<Result<User, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(items.len(), 0);
    }

    #[tokio::test]
    async fn test_cursor_pager_error_propagation() {
        use modkit_odata::{CursorV1, SortDir};

        let cursor = CursorV1 {
            k: vec!["1".to_owned()],
            o: SortDir::Asc,
            s: "filter_hash".to_owned(),
            f: Some("filter_hash".to_owned()),
            d: "fwd".to_owned(),
        };
        let encoded_cursor = cursor.encode().unwrap();

        let page1 = Page::new(
            vec![User {
                id: 1,
                name: "Alice".to_owned(),
            }],
            PageInfo {
                next_cursor: Some(encoded_cursor),
                prev_cursor: None,
                limit: 1,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1]);
        let query = ODataQuery::new().with_limit(1);

        let pager = CursorPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let items: Vec<Result<User, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(items.len(), 2);
        assert!(items[0].is_ok());
        assert!(items[1].is_err());

        // Verify it's a Fetch error
        if let Err(PagerError::Fetch(_)) = &items[1] {
            // Expected
        } else {
            panic!("Expected PagerError::Fetch");
        }
    }

    #[tokio::test]
    async fn test_pages_pager_two_pages() {
        use modkit_odata::{CursorV1, SortDir};

        let cursor = CursorV1 {
            k: vec!["2".to_owned()],
            o: SortDir::Asc,
            s: "filter_hash".to_owned(),
            f: Some("filter_hash".to_owned()),
            d: "fwd".to_owned(),
        };
        let encoded_cursor = cursor.encode().unwrap();

        let page1 = Page::new(
            vec![
                User {
                    id: 1,
                    name: "Alice".to_owned(),
                },
                User {
                    id: 2,
                    name: "Bob".to_owned(),
                },
            ],
            PageInfo {
                next_cursor: Some(encoded_cursor.clone()),
                prev_cursor: None,
                limit: 2,
            },
        );

        let page2 = Page::new(
            vec![User {
                id: 3,
                name: "Charlie".to_owned(),
            }],
            PageInfo {
                next_cursor: None,
                prev_cursor: Some(encoded_cursor),
                limit: 2,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1.clone(), page2.clone()]);
        let query = ODataQuery::new().with_limit(2);

        let pager = PagesPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let pages: Vec<Result<Page<User>, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(pages.len(), 2);
        assert!(pages.iter().all(Result::is_ok));

        let page_results: Vec<Page<User>> = pages.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(page_results[0].items.len(), 2);
        assert_eq!(page_results[1].items.len(), 1);
        assert_eq!(page_results[0].items[0].name, "Alice");
        assert_eq!(page_results[1].items[0].name, "Charlie");
    }

    #[tokio::test]
    async fn test_pages_pager_single_page() {
        let page = Page::new(
            vec![User {
                id: 1,
                name: "Alice".to_owned(),
            }],
            PageInfo {
                next_cursor: None,
                prev_cursor: None,
                limit: 10,
            },
        );

        let fetcher = FakeFetcher::new(vec![page.clone()]);
        let query = ODataQuery::new().with_limit(10);

        let pager = PagesPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let pages: Vec<Result<Page<User>, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(pages.len(), 1);
        assert!(pages[0].is_ok());
    }

    #[tokio::test]
    async fn test_cursor_pager_invalid_cursor() {
        let page1 = Page::new(
            vec![User {
                id: 1,
                name: "Alice".to_owned(),
            }],
            PageInfo {
                next_cursor: Some("invalid_cursor_string".to_owned()),
                prev_cursor: None,
                limit: 1,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1]);
        let query = ODataQuery::new().with_limit(1);

        let pager = CursorPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let items: Vec<Result<User, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(items.len(), 2);
        assert!(items[0].is_ok());
        assert!(items[1].is_err());

        // Verify it's an InvalidCursor error
        if let Err(PagerError::InvalidCursor(cursor)) = &items[1] {
            assert_eq!(cursor, "invalid_cursor_string");
        } else {
            panic!("Expected PagerError::InvalidCursor");
        }
    }

    #[tokio::test]
    async fn test_pages_pager_invalid_cursor() {
        let page1 = Page::new(
            vec![User {
                id: 1,
                name: "Alice".to_owned(),
            }],
            PageInfo {
                next_cursor: Some("invalid_cursor_string".to_owned()),
                prev_cursor: None,
                limit: 1,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1]);
        let query = ODataQuery::new().with_limit(1);

        let pager = PagesPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let pages: Vec<Result<Page<User>, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(pages.len(), 2);
        assert!(pages[0].is_ok());
        assert!(pages[1].is_err());

        // Verify it's an InvalidCursor error
        if let Err(PagerError::InvalidCursor(cursor)) = &pages[1] {
            assert_eq!(cursor, "invalid_cursor_string");
        } else {
            panic!("Expected PagerError::InvalidCursor");
        }
    }

    #[tokio::test]
    async fn test_pages_pager_error_propagation() {
        use modkit_odata::{CursorV1, SortDir};

        let cursor = CursorV1 {
            k: vec!["1".to_owned()],
            o: SortDir::Asc,
            s: "filter_hash".to_owned(),
            f: Some("filter_hash".to_owned()),
            d: "fwd".to_owned(),
        };
        let encoded_cursor = cursor.encode().unwrap();

        let page1 = Page::new(
            vec![User {
                id: 1,
                name: "Alice".to_owned(),
            }],
            PageInfo {
                next_cursor: Some(encoded_cursor),
                prev_cursor: None,
                limit: 1,
            },
        );

        let fetcher = FakeFetcher::new(vec![page1]);
        let query = ODataQuery::new().with_limit(1);

        let pager = PagesPager::new(query, move |q| {
            let fetcher = fetcher.clone();
            async move { fetcher.fetch(q) }
        });

        let pages: Vec<Result<Page<User>, PagerError<FakeError>>> = pager.collect().await;

        assert_eq!(pages.len(), 2);
        assert!(pages[0].is_ok());
        assert!(pages[1].is_err());

        // Verify it's a Fetch error
        if let Err(PagerError::Fetch(_)) = &pages[1] {
            // Expected
        } else {
            panic!("Expected PagerError::Fetch");
        }
    }

    #[test]
    fn test_pages_pager_polls_new_future_immediately() {
        struct PollCountingFuture {
            polls: Arc<AtomicUsize>,
        }

        impl Future for PollCountingFuture {
            type Output = Result<Page<User>, FakeError>;

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                self.polls.fetch_add(1, Ordering::SeqCst);
                Poll::Pending
            }
        }

        let polls = Arc::new(AtomicUsize::new(0));
        let polls_for_fetcher = polls.clone();

        let mut pager = PagesPager::new(ODataQuery::new().with_limit(1), move |_q| {
            PollCountingFuture {
                polls: polls_for_fetcher.clone(),
            }
        });

        let waker = futures_util::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        let poll = Pin::new(&mut pager).poll_next(&mut cx);
        assert!(matches!(poll, Poll::Pending));

        // If we don't poll immediately after installing the future, this would be 0.
        assert_eq!(polls.load(Ordering::SeqCst), 1);
    }
}
