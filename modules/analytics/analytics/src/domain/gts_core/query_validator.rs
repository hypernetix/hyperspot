use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct QueryValidator {
    indexed_fields: HashSet<String>,
}

impl QueryValidator {
    pub fn new(indexed_fields: Vec<String>) -> Self {
        Self {
            indexed_fields: indexed_fields.into_iter().collect(),
        }
    }

    pub fn validate_filter(&self, filter: &str) -> Result<(), ValidationError> {
        let fields = extract_fields_from_filter(filter);
        
        for field in fields {
            if !self.indexed_fields.contains(&field) {
                return Err(ValidationError::UnindexedField {
                    field: field.clone(),
                    available_fields: self.indexed_fields.iter().cloned().collect(),
                });
            }
        }

        Ok(())
    }

    pub fn available_fields(&self) -> Vec<String> {
        self.indexed_fields.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    UnindexedField {
        field: String,
        available_fields: Vec<String>,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::UnindexedField { field, available_fields } => {
                write!(
                    f,
                    "Field '{}' is not indexed. Available indexed fields: {}",
                    field,
                    available_fields.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

fn extract_fields_from_filter(filter: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let parts: Vec<&str> = filter.split_whitespace().collect();
    
    for part in parts {
        if part.starts_with("entity/") {
            fields.push(part.to_string());
        }
    }
    
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_accepts_indexed_field() {
        let validator = QueryValidator::new(vec![
            "entity/name".to_string(),
            "entity/created_at".to_string(),
        ]);

        let result = validator.validate_filter("entity/name eq 'test'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validator_rejects_unindexed_field() {
        let validator = QueryValidator::new(vec![
            "entity/name".to_string(),
            "entity/created_at".to_string(),
        ]);

        let result = validator.validate_filter("entity/unsupported_field eq 'value'");
        assert!(result.is_err());

        if let Err(ValidationError::UnindexedField { field, available_fields }) = result {
            assert_eq!(field, "entity/unsupported_field");
            assert!(available_fields.contains(&"entity/name".to_string()));
            assert!(available_fields.contains(&"entity/created_at".to_string()));
        }
    }

    #[test]
    fn test_validator_complex_filter() {
        let validator = QueryValidator::new(vec![
            "entity/name".to_string(),
            "entity/age".to_string(),
        ]);

        let result = validator.validate_filter("entity/name eq 'test' and entity/age gt 18");
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_fields_from_filter() {
        let fields = extract_fields_from_filter("entity/name eq 'test' and entity/age gt 18");
        assert_eq!(fields, vec!["entity/name", "entity/age"]);
    }
}
