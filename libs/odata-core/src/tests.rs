#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        base64_url, CursorError, CursorV1, ODataOrderBy, ODataPageError, ODataQuery, OrderKey,
        SortDir,
    };

    #[test]
    fn test_cursor_v1_encode_decode_round_trip() {
        let cursor = CursorV1 {
            k: vec![
                "2023-11-14T12:00:00Z".to_string(),
                "123e4567-e89b-12d3-a456-426614174000".to_string(),
            ],
            o: SortDir::Desc,
            s: "+created_at,-id".to_string(),
            f: Some("abc123".to_string()),
        };

        let encoded = cursor.encode();
        let decoded = CursorV1::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.k, cursor.k);
        assert_eq!(decoded.o, cursor.o);
        assert_eq!(decoded.s, cursor.s);
        assert_eq!(decoded.f, cursor.f);
    }

    #[test]
    fn test_cursor_v1_encode_decode_without_filter_hash() {
        let cursor = CursorV1 {
            k: vec!["value1".to_string(), "value2".to_string()],
            o: SortDir::Asc,
            s: "+field1,+field2".to_string(),
            f: None,
        };

        let encoded = cursor.encode();
        let decoded = CursorV1::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.k, cursor.k);
        assert_eq!(decoded.o, cursor.o);
        assert_eq!(decoded.s, cursor.s);
        assert_eq!(decoded.f, cursor.f);
    }

    #[test]
    fn test_cursor_v1_decode_invalid_base64() {
        let result = CursorV1::decode("invalid_base64!");
        assert!(matches!(result, Err(CursorError::InvalidBase64)));
    }

    #[test]
    fn test_cursor_v1_decode_invalid_json() {
        let invalid_json = base64_url::encode(b"not_json");
        let result = CursorV1::decode(&invalid_json);
        assert!(matches!(result, Err(CursorError::InvalidJson)));
    }

    #[test]
    fn test_cursor_v1_decode_invalid_version() {
        let cursor_data = serde_json::json!({
            "v": 2,
            "k": ["value"],
            "o": "asc",
            "s": "+field"
        });
        let encoded = base64_url::encode(serde_json::to_vec(&cursor_data).unwrap().as_slice());
        let result = CursorV1::decode(&encoded);
        assert!(matches!(result, Err(CursorError::InvalidVersion)));
    }

    #[test]
    fn test_cursor_v1_decode_empty_keys() {
        let cursor_data = serde_json::json!({
            "v": 1,
            "k": [],
            "o": "asc",
            "s": "+field"
        });
        let encoded = base64_url::encode(serde_json::to_vec(&cursor_data).unwrap().as_slice());
        let result = CursorV1::decode(&encoded);
        assert!(matches!(result, Err(CursorError::InvalidKeys)));
    }

    #[test]
    fn test_cursor_v1_decode_empty_fields() {
        let cursor_data = serde_json::json!({
            "v": 1,
            "k": ["value"],
            "o": "asc",
            "s": ""
        });
        let encoded = base64_url::encode(serde_json::to_vec(&cursor_data).unwrap().as_slice());
        let result = CursorV1::decode(&encoded);
        assert!(matches!(result, Err(CursorError::InvalidFields)));
    }

    #[test]
    fn test_cursor_v1_decode_invalid_direction() {
        let cursor_data = serde_json::json!({
            "v": 1,
            "k": ["value"],
            "o": "invalid",
            "s": "+field"
        });
        let encoded = base64_url::encode(serde_json::to_vec(&cursor_data).unwrap().as_slice());
        let result = CursorV1::decode(&encoded);
        assert!(matches!(result, Err(CursorError::InvalidDirection)));
    }

    #[test]
    fn test_odata_order_by_to_signed_tokens() {
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Asc,
            },
            OrderKey {
                field: "name".to_string(),
                dir: SortDir::Desc,
            },
        ]);

        let tokens = order.to_signed_tokens();
        assert_eq!(tokens, "-created_at,+id,-name");
    }

    #[test]
    fn test_odata_order_by_empty_to_signed_tokens() {
        let order = ODataOrderBy::empty();
        let tokens = order.to_signed_tokens();
        assert_eq!(tokens, "");
    }

    #[test]
    fn test_odata_order_by_equals_signed_tokens() {
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Asc,
            },
        ]);

        assert!(order.equals_signed_tokens("-created_at,+id"));
        assert!(order.equals_signed_tokens("  -created_at , +id  ")); // whitespace tolerance
        assert!(!order.equals_signed_tokens("-created_at,+id,+name")); // different length
        assert!(!order.equals_signed_tokens("-created_at,-id")); // different direction
        assert!(!order.equals_signed_tokens("+created_at,+id")); // different direction
    }

    #[test]
    fn test_odata_order_by_equals_signed_tokens_implicit_asc() {
        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_string(),
            dir: SortDir::Asc,
        }]);

        assert!(order.equals_signed_tokens("+name"));
        assert!(order.equals_signed_tokens("name")); // implicit asc
    }

    #[test]
    fn test_odata_order_by_ensure_tiebreaker() {
        let order = ODataOrderBy(vec![OrderKey {
            field: "created_at".to_string(),
            dir: SortDir::Desc,
        }]);

        let with_tiebreaker = order.ensure_tiebreaker("id", SortDir::Desc);
        assert_eq!(with_tiebreaker.0.len(), 2);
        assert_eq!(with_tiebreaker.0[0].field, "created_at");
        assert_eq!(with_tiebreaker.0[1].field, "id");
        assert_eq!(with_tiebreaker.0[1].dir, SortDir::Desc);
    }

    #[test]
    fn test_odata_order_by_ensure_tiebreaker_already_present() {
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Asc,
            },
        ]);

        let with_tiebreaker = order.ensure_tiebreaker("id", SortDir::Desc);
        // Should not add duplicate, keep original
        assert_eq!(with_tiebreaker.0.len(), 2);
        assert_eq!(with_tiebreaker.0[1].field, "id");
        assert_eq!(with_tiebreaker.0[1].dir, SortDir::Asc); // original direction preserved
    }

    #[test]
    fn test_odata_query_builder_pattern() {
        use crate::ast::*;

        let expr = Expr::Compare(
            Box::new(Expr::Identifier("email".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test@example.com".to_string()))),
        );

        let order = ODataOrderBy(vec![OrderKey {
            field: "created_at".to_string(),
            dir: SortDir::Desc,
        }]);

        let cursor = CursorV1 {
            k: vec!["2023-11-14T12:00:00Z".to_string()],
            o: SortDir::Desc,
            s: "-created_at".to_string(),
            f: None,
        };

        let query = ODataQuery::new()
            .with_filter(expr)
            .with_order(order)
            .with_limit(25)
            .with_cursor(cursor)
            .with_filter_hash("abc123".to_string());

        assert!(query.filter.is_some());
        assert_eq!(query.order.0.len(), 1);
        assert_eq!(query.limit, Some(25));
        assert!(query.cursor.is_some());
        assert_eq!(query.filter_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_odata_query_legacy_compatibility() {
        use crate::ast::*;

        let expr = Expr::Compare(
            Box::new(Expr::Identifier("name".to_string())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test".to_string()))),
        );

        // Test new API
        let query = ODataQuery::default().with_filter(expr.clone());
        assert!(query.has_filter());
        assert!(query.filter().is_some());

        let empty = ODataQuery::default();
        assert!(!empty.has_filter());
        assert!(empty.filter().is_none());

        // Test conversion
        let from_option = ODataQuery::from(Some(expr));
        assert!(from_option.has_filter());
    }

    #[test]
    fn test_orderby_from_signed_tokens() {
        // Test basic parsing
        let result = ODataOrderBy::from_signed_tokens("+name,-created_at").unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].field, "name");
        assert_eq!(result.0[0].dir, SortDir::Asc);
        assert_eq!(result.0[1].field, "created_at");
        assert_eq!(result.0[1].dir, SortDir::Desc);

        // Test empty string should now error
        let result = ODataOrderBy::from_signed_tokens("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ODataPageError::InvalidOrderByField(_)
        ));

        // Test single field
        let result = ODataOrderBy::from_signed_tokens("-id").unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].field, "id");
        assert_eq!(result.0[0].dir, SortDir::Desc);
    }

    #[test]
    fn test_orderby_display_formatting() {
        // Test empty order
        let order = ODataOrderBy::empty();
        assert_eq!(format!("{}", order), "(none)");

        // Test single field
        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_string(),
            dir: SortDir::Asc,
        }]);
        assert_eq!(format!("{}", order), "name asc");

        // Test multiple fields
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Desc,
            },
        ]);
        assert_eq!(format!("{}", order), "created_at desc, id desc");

        // Test mixed directions
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "email".to_string(),
                dir: SortDir::Asc,
            },
            OrderKey {
                field: "created_at".to_string(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_string(),
                dir: SortDir::Desc,
            },
        ]);
        assert_eq!(format!("{}", order), "email asc, created_at desc, id desc");
    }

    #[test]
    fn test_orderby_roundtrip_signed_tokens_display() {
        // Test that we can parse signed tokens and get readable display
        let signed = "+email,-created_at,-id";
        let order = ODataOrderBy::from_signed_tokens(signed).unwrap();
        let display = format!("{}", order);
        assert_eq!(display, "email asc, created_at desc, id desc");

        // Test roundtrip back to signed tokens
        let back_to_signed = order.to_signed_tokens();
        assert_eq!(back_to_signed, signed);
    }

    #[test]
    fn test_orderby_from_signed_tokens_error_cases() {
        // Test empty field name
        let result = ODataOrderBy::from_signed_tokens("+");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ODataPageError::InvalidOrderByField(_)
        ));

        // Test field with just sign
        let result = ODataOrderBy::from_signed_tokens("-");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ODataPageError::InvalidOrderByField(_)
        ));

        // Test field with comma but empty segment
        let result = ODataOrderBy::from_signed_tokens("+name,,+email");
        // Should skip empty segments and succeed
        let result = result.unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].field, "name");
        assert_eq!(result.0[1].field, "email");

        // Test implicit asc direction
        let result = ODataOrderBy::from_signed_tokens("name").unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].field, "name");
        assert_eq!(result.0[0].dir, SortDir::Asc);
    }
}
