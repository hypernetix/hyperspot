//! Simple test to verify cursor+orderby policy enforcement

use modkit_odata::{CursorV1, SortDir};

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
    let order_from_cursor = modkit_odata::ODataOrderBy::from_signed_tokens(&decoded.s)
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
fn test_order_with_cursor_error_converts_to_problem() {
    use modkit_errors::problem::Problem;
    use modkit_odata::Error as ODataError;

    // Test that OrderWithCursor error converts to Problem properly
    let odata_error = ODataError::OrderWithCursor;
    let problem: Problem = odata_error.into();

    // Verify the problem has the correct properties
    assert_eq!(problem.status, 422);
    assert!(problem.code.contains("odata"));
    assert!(problem.code.contains("invalid_cursor"));
}
