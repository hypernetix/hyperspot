//! Tests for cursor+orderby policy enforcement

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::super::odata::*;
    use axum::http::{Uri, request::Parts};
    use modkit_odata::{CursorV1, SortDir};

    fn mock_parts(query_string: &str) -> Parts {
        let uri: Uri = format!("http://example.com/test?{query_string}")
            .parse()
            .unwrap();
        let request = axum::http::Request::builder().uri(uri).body(()).unwrap();
        let (parts, ()) = request.into_parts();
        parts
    }

    #[tokio::test]
    async fn test_cursor_with_orderby_conflict() {
        let mut parts = mock_parts("cursor=dGVzdA%3D%3D&%24orderby=id%20desc");
        let result = extract_odata_query(&mut parts, &()).await;

        assert!(result.is_err());
        let _problem_response = result.unwrap_err();
    }

    #[tokio::test]
    async fn test_cursor_only_success() {
        // Create a valid cursor
        let cursor = CursorV1 {
            k: vec!["test".to_owned()],
            o: SortDir::Desc,
            s: "-id".to_owned(),
            f: None,
            d: "fwd".to_owned(),
        };
        let cursor_encoded = cursor.encode().unwrap();

        let mut parts = mock_parts(&format!("cursor={}", urlencoding::encode(&cursor_encoded)));
        let result = extract_odata_query(&mut parts, &()).await;

        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.cursor.is_some());
        assert!(query.order.is_empty()); // Order should be empty when cursor is present
    }

    #[tokio::test]
    async fn test_orderby_only_success() {
        let mut parts = mock_parts("%24orderby=id%20desc%2C%20name%20asc");
        let result = extract_odata_query(&mut parts, &()).await;

        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.cursor.is_none());
        assert!(!query.order.is_empty());
        assert_eq!(query.order.0.len(), 2);
        assert_eq!(query.order.0[0].field, "id");
        assert_eq!(query.order.0[0].dir, SortDir::Desc);
        assert_eq!(query.order.0[1].field, "name");
        assert_eq!(query.order.0[1].dir, SortDir::Asc);
    }

    #[tokio::test]
    async fn test_neither_cursor_nor_orderby() {
        let mut parts = mock_parts("limit=10");
        let result = extract_odata_query(&mut parts, &()).await;

        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.cursor.is_none());
        assert!(query.order.is_empty());
        assert_eq!(query.limit, Some(10));
    }

    #[tokio::test]
    async fn test_invalid_cursor_error() {
        let mut parts = mock_parts("cursor=invalid_base64");
        let result = extract_odata_query(&mut parts, &()).await;

        assert!(result.is_err());
        let _problem_response = result.unwrap_err();
    }
}
