#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_odata::{ODataOrderBy, OrderKey, SortDir};

#[test]
fn signed_tokens_roundtrip() {
    let ob = ODataOrderBy(vec![
        OrderKey {
            field: "created_at".into(),
            dir: SortDir::Desc,
        },
        OrderKey {
            field: "id".into(),
            dir: SortDir::Asc,
        },
    ]);
    let s = ob.to_signed_tokens();
    assert_eq!(s, "-created_at,+id");
    let parsed = ODataOrderBy::from_signed_tokens(&s).expect("parse");
    assert!(parsed.equals_signed_tokens(&s));
}

#[test]
fn signed_tokens_single_field() {
    let ob = ODataOrderBy(vec![OrderKey {
        field: "name".into(),
        dir: SortDir::Asc,
    }]);
    let s = ob.to_signed_tokens();
    assert_eq!(s, "+name");
    let parsed = ODataOrderBy::from_signed_tokens(&s).expect("parse");
    assert!(parsed.equals_signed_tokens(&s));
}

#[test]
fn signed_tokens_empty() {
    let ob = ODataOrderBy::empty();
    let s = ob.to_signed_tokens();
    assert_eq!(s, "");
    // Empty should fail parsing
    assert!(ODataOrderBy::from_signed_tokens(&s).is_err());
}
