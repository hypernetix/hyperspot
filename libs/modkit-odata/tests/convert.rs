#[cfg(feature = "with-odata-params")]
mod tests {
    use modkit_odata::ast::*;
    use odata_params::filters as od;

    #[test]
    fn converts_and_contains() {
        let src = od::parse_str("score ge 10 and contains(email,'@acme.com')").unwrap();
        let dst: Expr = src.into();

        match dst {
            Expr::And(a, b) => {
                match *a {
                    Expr::Compare(l, op, r) => {
                        matches!(*l, Expr::Identifier(_));
                        assert!(matches!(op, CompareOperator::Ge));
                        matches!(*r, Expr::Value(Value::Number(_)));
                    }
                    _ => panic!("left not compare"),
                }
                match *b {
                    Expr::Function(ref name, ref args) => {
                        assert_eq!(name.to_lowercase(), "contains");
                        assert!(matches!(args[0], Expr::Identifier(_)));
                        assert!(matches!(args[1], Expr::Value(Value::String(_))));
                    }
                    _ => panic!("right not function"),
                }
            }
            _ => panic!("not And()"),
        }
    }

    #[test]
    fn converts_in_uuid() {
        let src = od::parse_str(
            "id in (00000000-0000-0000-0000-000000000001, 00000000-0000-0000-0000-000000000002)",
        )
        .unwrap();
        let dst: Expr = src.into();
        if let Expr::In(lhs, list) = dst {
            assert!(matches!(*lhs, Expr::Identifier(_)));
            assert_eq!(list.len(), 2);
        } else {
            panic!("expected In()");
        }
    }
}
