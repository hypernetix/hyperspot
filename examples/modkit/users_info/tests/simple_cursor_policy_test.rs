//! Simple test to verify cursor+orderby policy enforcement

use odata_core::{CursorV1, SortDir};

#[tokio::test]
async fn test_cursor_orderby_policy_validation() {
    // Create a cursor with order information
    let cursor = CursorV1 {
        k: vec!["test-value".to_string()],
        o: SortDir::Desc,
        s: "-id,+email".to_string(), // This represents the order from cursor
        f: None,
    };

    // Verify cursor encodes/decodes properly
    let encoded = cursor.encode();
    let decoded = CursorV1::decode(&encoded).expect("Failed to decode cursor");

    assert_eq!(decoded.s, "-id,+email");
    assert_eq!(decoded.o, SortDir::Desc);
    assert_eq!(decoded.k, vec!["test-value"]);

    // Test the from_signed_tokens functionality
    let order_from_cursor = odata_core::ODataOrderBy::from_signed_tokens(&decoded.s)
        .expect("Failed to parse order from cursor");

    assert_eq!(order_from_cursor.0.len(), 2);
    assert_eq!(order_from_cursor.0[0].field, "id");
    assert_eq!(order_from_cursor.0[0].dir, SortDir::Desc);
    assert_eq!(order_from_cursor.0[1].field, "email");
    assert_eq!(order_from_cursor.0[1].dir, SortDir::Asc);

    // Test that we can convert back to signed tokens
    let signed_tokens = order_from_cursor.to_signed_tokens();
    assert_eq!(signed_tokens, "-id,+email");
}

#[test]
fn test_order_with_cursor_error_mapping() {
    use odata_core::ODataPageError;
    use users_info::contract::error::UsersInfoError;

    // Test that OrderWithCursor error maps properly
    let page_error = ODataPageError::OrderWithCursor;
    let users_error: UsersInfoError = page_error.into();

    match users_error {
        UsersInfoError::Validation { message } => {
            assert_eq!(message, "Cannot specify both orderby and cursor");
        }
        _ => panic!("Expected validation error"),
    }
}
