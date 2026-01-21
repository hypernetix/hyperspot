#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::module_inception)]
mod tests {
    use crate::{CursorV1, Error, ODataOrderBy, ODataQuery, OrderKey, SortDir, base64_url};

    #[test]
    fn test_cursor_v1_encode_decode_round_trip() {
        let cursor = CursorV1 {
            k: vec![
                "2023-11-14T12:00:00Z".to_owned(),
                "123e4567-e89b-12d3-a456-426614174000".to_owned(),
            ],
            o: SortDir::Desc,
            s: "+created_at,-id".to_owned(),
            f: Some("abc123".to_owned()),
            d: "fwd".to_owned(),
        };

        let encoded = cursor.encode().expect("encode should succeed");
        let decoded = CursorV1::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.k, cursor.k);
        assert_eq!(decoded.o, cursor.o);
        assert_eq!(decoded.s, cursor.s);
        assert_eq!(decoded.f, cursor.f);
        assert_eq!(decoded.d, cursor.d);
    }

    #[test]
    fn test_cursor_v1_encode_decode_without_filter_hash() {
        let cursor = CursorV1 {
            k: vec!["value1".to_owned(), "value2".to_owned()],
            o: SortDir::Asc,
            s: "+field1,+field2".to_owned(),
            f: None,
            d: "fwd".to_owned(),
        };

        let encoded = cursor.encode().expect("encode should succeed");
        let decoded = CursorV1::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.k, cursor.k);
        assert_eq!(decoded.o, cursor.o);
        assert_eq!(decoded.s, cursor.s);
        assert_eq!(decoded.f, cursor.f);
    }

    #[test]
    fn test_cursor_v1_decode_invalid_base64() {
        let result = CursorV1::decode("invalid_base64!");
        assert!(matches!(result, Err(Error::CursorInvalidBase64)));
    }

    #[test]
    fn test_cursor_v1_decode_invalid_json() {
        let invalid_json = base64_url::encode(b"not_json");
        let result = CursorV1::decode(&invalid_json);
        assert!(matches!(result, Err(Error::CursorInvalidJson)));
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
        assert!(matches!(result, Err(Error::CursorInvalidVersion)));
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
        assert!(matches!(result, Err(Error::CursorInvalidKeys)));
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
        assert!(matches!(result, Err(Error::CursorInvalidFields)));
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
        assert!(matches!(result, Err(Error::CursorInvalidDirection)));
    }

    #[test]
    fn test_odata_order_by_to_signed_tokens() {
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_owned(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_owned(),
                dir: SortDir::Asc,
            },
            OrderKey {
                field: "name".to_owned(),
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
                field: "created_at".to_owned(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_owned(),
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
            field: "name".to_owned(),
            dir: SortDir::Asc,
        }]);

        assert!(order.equals_signed_tokens("+name"));
        assert!(order.equals_signed_tokens("name")); // implicit asc
    }

    #[test]
    fn test_odata_order_by_ensure_tiebreaker() {
        let order = ODataOrderBy(vec![OrderKey {
            field: "created_at".to_owned(),
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
                field: "created_at".to_owned(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_owned(),
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
            Box::new(Expr::Identifier("email".to_owned())),
            CompareOperator::Eq,
            Box::new(Expr::Value(Value::String("test@example.com".to_owned()))),
        );

        let order = ODataOrderBy(vec![OrderKey {
            field: "created_at".to_owned(),
            dir: SortDir::Desc,
        }]);

        let cursor = CursorV1 {
            k: vec!["2023-11-14T12:00:00Z".to_owned()],
            o: SortDir::Desc,
            s: "-created_at".to_owned(),
            f: None,
            d: "fwd".to_owned(),
        };

        let query = ODataQuery::new()
            .with_filter(expr)
            .with_order(order)
            .with_limit(25)
            .with_cursor(cursor)
            .with_filter_hash("abc123".to_owned());

        assert!(query.filter.is_some());
        assert_eq!(query.order.0.len(), 1);
        assert_eq!(query.limit, Some(25));
        assert!(query.cursor.is_some());
        assert_eq!(query.filter_hash, Some("abc123".to_owned()));
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
        assert!(matches!(result.unwrap_err(), Error::InvalidOrderByField(_)));

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
        assert_eq!(format!("{order}"), "(none)");

        // Test single field
        let order = ODataOrderBy(vec![OrderKey {
            field: "name".to_owned(),
            dir: SortDir::Asc,
        }]);
        assert_eq!(format!("{order}"), "name asc");

        // Test multiple fields
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "created_at".to_owned(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_owned(),
                dir: SortDir::Desc,
            },
        ]);
        assert_eq!(format!("{order}"), "created_at desc, id desc");

        // Test mixed directions
        let order = ODataOrderBy(vec![
            OrderKey {
                field: "email".to_owned(),
                dir: SortDir::Asc,
            },
            OrderKey {
                field: "created_at".to_owned(),
                dir: SortDir::Desc,
            },
            OrderKey {
                field: "id".to_owned(),
                dir: SortDir::Desc,
            },
        ]);
        assert_eq!(format!("{order}"), "email asc, created_at desc, id desc");
    }

    #[test]
    fn test_orderby_roundtrip_signed_tokens_display() {
        // Test that we can parse signed tokens and get readable display
        let signed = "+email,-created_at,-id";
        let order = ODataOrderBy::from_signed_tokens(signed).unwrap();
        let display = format!("{order}");
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
        assert!(matches!(result.unwrap_err(), Error::InvalidOrderByField(_)));

        // Test field with just sign
        let result = ODataOrderBy::from_signed_tokens("-");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidOrderByField(_)));

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

    #[test]
    fn test_unified_error_handling() {
        // Test cursor decode with unified error
        let invalid_cursor = "invalid_base64!";
        let result = CursorV1::decode(invalid_cursor);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::CursorInvalidBase64));

        // Test that all cursor errors use the unified Error type
        let invalid_json = base64_url::encode(b"not_json");
        let result = CursorV1::decode(&invalid_json);
        assert!(matches!(result.unwrap_err(), Error::CursorInvalidJson));
    }

    #[test]
    fn test_error_messages() {
        // Test that error messages are descriptive
        let filter_err = Error::InvalidFilter("malformed expression".to_owned());
        assert_eq!(
            filter_err.to_string(),
            "invalid $filter: malformed expression"
        );

        let cursor_err = Error::CursorInvalidBase64;
        assert_eq!(
            cursor_err.to_string(),
            "invalid cursor: invalid base64url encoding"
        );

        let orderby_err = Error::InvalidOrderByField("unknown_field".to_owned());
        assert_eq!(
            orderby_err.to_string(),
            "unsupported $orderby field: unknown_field"
        );
    }
}
