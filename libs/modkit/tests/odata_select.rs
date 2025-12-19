use modkit::api::{
    odata::{parse_select, ODataParams},
    select::{apply_select, page_to_projected_json, project_json},
};
use modkit_odata::Page;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn test_parse_select_single_field() {
    let result = parse_select("id").unwrap();
    assert_eq!(result, vec!["id"]);
}

#[test]
fn test_parse_select_multiple_fields() {
    let result = parse_select("id, name, email").unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"id".to_owned()));
    assert!(result.contains(&"name".to_owned()));
    assert!(result.contains(&"email".to_owned()));
}

#[test]
fn test_parse_select_case_insensitive() {
    let result = parse_select("ID, Name, EMAIL").unwrap();
    assert_eq!(result, vec!["id", "name", "email"]);
}

#[test]
fn test_parse_select_with_whitespace() {
    let result = parse_select("  id  ,  name  ,  email  ").unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"id".to_owned()));
    assert!(result.contains(&"name".to_owned()));
    assert!(result.contains(&"email".to_owned()));
}

#[test]
fn test_parse_select_empty_string() {
    let result = parse_select("");
    assert!(result.is_err());
}

#[test]
fn test_parse_select_only_whitespace() {
    let result = parse_select("   ");
    assert!(result.is_err());
}

#[test]
fn test_parse_select_duplicate_fields() {
    let result = parse_select("id, name, id");
    assert!(result.is_err());
}

#[test]
fn test_parse_select_too_long() {
    let long_string = "a".repeat(3000);
    let result = parse_select(&long_string);
    assert!(result.is_err());
}

#[test]
fn test_parse_select_too_many_fields() {
    let fields = (0..150)
        .map(|i| format!("field{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let result = parse_select(&fields);
    assert!(result.is_err());
}

#[test]
fn test_odata_params_with_select() {
    let params = ODataParams {
        filter: None,
        orderby: None,
        select: Some("id, name".to_owned()),
        limit: None,
        cursor: None,
    };
    assert_eq!(params.select, Some("id, name".to_owned()));
}

#[test]
fn test_project_json_simple_object() {
    let value = json!({
        "id": "123",
        "name": "John",
        "email": "john@example.com"
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("name".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    assert_eq!(projected.get("name").and_then(|v| v.as_str()), Some("John"));
    assert!(projected.get("email").is_none());
}

#[test]
fn test_project_json_array() {
    let value = json!([
        {"id": "1", "name": "John", "email": "john@example.com"},
        {"id": "2", "name": "Jane", "email": "jane@example.com"}
    ]);

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("name".to_owned());

    let projected = project_json(&value, &fields);

    let arr = projected.as_array().unwrap();
    assert_eq!(arr.len(), 2);

    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some("1"));
    assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("John"));
    assert!(arr[0].get("email").is_none());

    assert_eq!(arr[1].get("id").and_then(|v| v.as_str()), Some("2"));
    assert_eq!(arr[1].get("name").and_then(|v| v.as_str()), Some("Jane"));
    assert!(arr[1].get("email").is_none());
}

#[test]
fn test_project_json_nested_object() {
    let value = json!({
        "id": "123",
        "user": {
            "name": "John",
            "email": "john@example.com"
        },
        "metadata": {
            "created": "2023-01-01"
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("user".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    assert!(projected.get("user").is_some());
    assert!(projected.get("metadata").is_none());
}

#[test]
fn test_project_json_case_insensitive() {
    let value = json!({
        "Id": "123",
        "Name": "John",
        "Email": "john@example.com"
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("name".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("Id").and_then(|v| v.as_str()), Some("123"));
    assert_eq!(projected.get("Name").and_then(|v| v.as_str()), Some("John"));
    assert!(projected.get("Email").is_none());
}

#[test]
fn test_apply_select_with_serializable() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
        email: String,
    }

    let user = User {
        id: "123".to_owned(),
        name: "John".to_owned(),
        email: "john@example.com".to_owned(),
    };

    let selected = vec!["id".to_owned(), "name".to_owned()];
    let result = apply_select(&user, Some(&selected));

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("123"));
    assert_eq!(result.get("name").and_then(|v| v.as_str()), Some("John"));
    assert!(result.get("email").is_none());
}

#[test]
fn test_apply_select_without_fields() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
    }

    let user = User {
        id: "123".to_owned(),
        name: "John".to_owned(),
    };

    let result = apply_select(&user, None);

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("123"));
    assert_eq!(result.get("name").and_then(|v| v.as_str()), Some("John"));
}

#[test]
fn test_apply_select_empty_fields() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
    }

    let user = User {
        id: "123".to_owned(),
        name: "John".to_owned(),
    };

    let result = apply_select(&user, Some(&[]));

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("123"));
    assert_eq!(result.get("name").and_then(|v| v.as_str()), Some("John"));
}

#[test]
fn test_project_json_dot_notation_entire_nested_object() {
    let value = json!({
        "id": "123",
        "access_control": {
            "read": true,
            "write": false,
            "delete": false
        },
        "metadata": {
            "created": "2023-01-01"
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("access_control".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    let ac = projected.get("access_control").and_then(|v| v.as_object());
    assert!(ac.is_some());
    assert_eq!(ac.unwrap().len(), 3);
    assert!(projected.get("metadata").is_none());
}

#[test]
fn test_project_json_dot_notation_specific_nested_fields() {
    let value = json!({
        "id": "123",
        "access_control": {
            "read": true,
            "write": false,
            "delete": false
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("access_control.read".to_owned());
    fields.insert("access_control.write".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    let ac = projected.get("access_control").and_then(|v| v.as_object());
    assert!(ac.is_some());
    let ac_obj = ac.unwrap();
    assert_eq!(ac_obj.len(), 2);
    assert_eq!(
        ac_obj.get("read").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        ac_obj.get("write").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert!(ac_obj.get("delete").is_none());
}

#[test]
fn test_project_json_dot_notation_case_insensitive() {
    let value = json!({
        "id": "123",
        "AccessControl": {
            "Read": true,
            "Write": false
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("accesscontrol.read".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    let ac = projected.get("AccessControl").and_then(|v| v.as_object());
    assert!(ac.is_some());
    let ac_obj = ac.unwrap();
    assert_eq!(
        ac_obj.get("Read").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(ac_obj.get("Write").is_none());
}

#[test]
fn test_project_json_dot_notation_deeply_nested() {
    let value = json!({
        "id": "123",
        "user": {
            "profile": {
                "name": "John",
                "email": "john@example.com",
                "age": 30
            },
            "settings": {
                "notifications": true
            }
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("user.profile.name".to_owned());
    fields.insert("user.profile.email".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    let user = projected.get("user").and_then(|v| v.as_object());
    assert!(user.is_some());
    let user_obj = user.unwrap();

    let profile = user_obj.get("profile").and_then(|v| v.as_object());
    assert!(profile.is_some());
    let profile_obj = profile.unwrap();
    assert_eq!(
        profile_obj.get("name").and_then(|v| v.as_str()),
        Some("John")
    );
    assert_eq!(
        profile_obj.get("email").and_then(|v| v.as_str()),
        Some("john@example.com")
    );
    assert!(profile_obj.get("age").is_none());
    assert!(user_obj.get("settings").is_none());
}

#[test]
fn test_project_json_dot_notation_with_arrays() {
    let value = json!({
        "id": "123",
        "items": [
            {
                "name": "Item1",
                "access_control": {
                    "read": true,
                    "write": false
                }
            },
            {
                "name": "Item2",
                "access_control": {
                    "read": false,
                    "write": true
                }
            }
        ]
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("items.access_control.read".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    let items = projected.get("items").and_then(|v| v.as_array());
    assert!(items.is_some());
    let items_arr = items.unwrap();
    assert_eq!(items_arr.len(), 2);

    let item1 = items_arr[0].as_object().unwrap();
    assert!(item1.get("name").is_none());
    let ac1 = item1.get("access_control").and_then(|v| v.as_object());
    assert!(ac1.is_some());
    assert_eq!(
        ac1.unwrap()
            .get("read")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(ac1.unwrap().get("write").is_none());
}

#[test]
fn test_project_json_dot_notation_mixed_selection() {
    let value = json!({
        "id": "123",
        "name": "John",
        "access_control": {
            "read": true,
            "write": false,
            "delete": false
        },
        "profile": {
            "bio": "A developer",
            "location": "NYC"
        }
    });

    let mut fields = HashSet::new();
    fields.insert("id".to_owned());
    fields.insert("access_control".to_owned());
    fields.insert("profile.bio".to_owned());

    let projected = project_json(&value, &fields);

    assert_eq!(projected.get("id").and_then(|v| v.as_str()), Some("123"));
    assert!(projected.get("name").is_none());

    let ac = projected.get("access_control").and_then(|v| v.as_object());
    assert!(ac.is_some());
    assert_eq!(ac.unwrap().len(), 3);

    let profile = projected.get("profile").and_then(|v| v.as_object());
    assert!(profile.is_some());
    let profile_obj = profile.unwrap();
    assert_eq!(
        profile_obj.get("bio").and_then(|v| v.as_str()),
        Some("A developer")
    );
    assert!(profile_obj.get("location").is_none());
}

#[test]
fn test_page_to_projected_json() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
        email: String,
    }

    let page = Page {
        items: vec![
            User {
                id: "1".to_owned(),
                name: "John".to_owned(),
                email: "john@example.com".to_owned(),
            },
            User {
                id: "2".to_owned(),
                name: "Jane".to_owned(),
                email: "jane@example.com".to_owned(),
            },
        ],
        page_info: modkit_odata::PageInfo {
            next_cursor: Some("abc123".to_owned()),
            prev_cursor: None,
            limit: 10,
        },
    };

    let selected = vec!["id".to_owned(), "name".to_owned()];
    let result = page_to_projected_json(&page, Some(&selected));

    assert_eq!(result.items.len(), 2);
    assert_eq!(
        result.items[0]
            .get("id")
            .and_then(serde_json::Value::as_str),
        Some("1")
    );
    assert_eq!(
        result.items[0]
            .get("name")
            .and_then(serde_json::Value::as_str),
        Some("John")
    );
    assert!(result.items[0].get("email").is_none());

    assert_eq!(
        result.items[1]
            .get("id")
            .and_then(serde_json::Value::as_str),
        Some("2")
    );
    assert_eq!(
        result.items[1]
            .get("name")
            .and_then(serde_json::Value::as_str),
        Some("Jane")
    );
    assert!(result.items[1].get("email").is_none());

    assert_eq!(result.page_info.next_cursor, Some("abc123".to_owned()));
    assert_eq!(result.page_info.limit, 10);
}

#[test]
fn test_page_to_projected_json_without_fields() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
    }

    let page = Page {
        items: vec![User {
            id: "1".to_owned(),
            name: "John".to_owned(),
        }],
        page_info: modkit_odata::PageInfo {
            next_cursor: None,
            prev_cursor: None,
            limit: 20,
        },
    };

    let result = page_to_projected_json(&page, None);

    assert_eq!(result.items.len(), 1);
    assert_eq!(
        result.items[0]
            .get("id")
            .and_then(serde_json::Value::as_str),
        Some("1")
    );
    assert_eq!(
        result.items[0]
            .get("name")
            .and_then(serde_json::Value::as_str),
        Some("John")
    );
    assert_eq!(result.page_info.limit, 20);
}
