//! `SQLite` DSN parsing and cleaning utilities.

use std::collections::HashMap;

/// Extract `SQLite` PRAGMA parameters from DSN and return cleaned DSN.
///
/// Parses the DSN as a URL and extracts whitelisted SQLite-specific parameters:
/// - `wal`, `synchronous`, `busy_timeout`, `journal_mode` (case-insensitive)
///
/// Returns:
/// - `clean_dsn`: original DSN with `SQLite` PRAGMA parameters removed
/// - `pairs`: `HashMap` of extracted PRAGMA parameters (normalized lowercase keys)
///
/// If URL parsing fails (e.g., plain file path), returns the original DSN unchanged
/// with an empty parameters map.
pub fn extract_sqlite_pragmas(dsn: &str) -> (String, HashMap<String, String>) {
    // List of SQLite-specific parameters that should be extracted
    const SQLITE_PRAGMA_PARAMS: &[&str] = &["wal", "synchronous", "busy_timeout", "journal_mode"];

    if let Ok(mut url) = url::Url::parse(dsn) {
        let mut extracted_pairs = HashMap::new();
        let mut remaining_pairs = Vec::new();

        // Process all query parameters
        for (key, value) in url.query_pairs() {
            let key_lower = key.to_lowercase();
            if SQLITE_PRAGMA_PARAMS.contains(&key_lower.as_str()) {
                // Extract SQLite PRAGMA parameter
                extracted_pairs.insert(key_lower, value.into_owned());
            } else {
                // Keep non-SQLite parameter
                remaining_pairs.push((key.into_owned(), value.into_owned()));
            }
        }

        // Clear all query parameters
        url.set_query(None);

        // Re-add only the non-SQLite parameters
        if !remaining_pairs.is_empty() {
            let query_string = remaining_pairs
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            url.set_query(Some(&query_string));
        }

        (url.to_string(), extracted_pairs)
    } else {
        // If URL parsing fails, return the original DSN with no extracted parameters
        (dsn.to_owned(), HashMap::new())
    }
}

/// Check if the DSN represents an in-memory `SQLite` database.
///
/// Returns `true` for:
/// - `sqlite::memory:` or `sqlite://memory:`
/// - DSNs containing `mode=memory` query parameter
pub fn is_memory_dsn(dsn: &str) -> bool {
    // Check for explicit memory DSN formats
    if dsn == "sqlite::memory:" || dsn == "sqlite://memory:" {
        return true;
    }

    // Check for mode=memory query parameter
    if let Ok(url) = url::Url::parse(dsn) {
        for (key, value) in url.query_pairs() {
            if key.to_lowercase() == "mode" && value.to_lowercase() == "memory" {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sqlite_pragmas_basic() {
        let dsn = "sqlite:///path/to/db.sqlite?wal=true&synchronous=NORMAL&other_param=value";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, "sqlite:///path/to/db.sqlite?other_param=value");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs.get("wal"), Some(&"true".to_owned()));
        assert_eq!(pairs.get("synchronous"), Some(&"NORMAL".to_owned()));
        assert!(!pairs.contains_key("other_param"));
    }

    #[test]
    fn test_extract_sqlite_pragmas_all_params() {
        let dsn =
            "sqlite:///test.db?wal=false&synchronous=OFF&busy_timeout=5000&journal_mode=DELETE";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, "sqlite:///test.db");
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs.get("wal"), Some(&"false".to_owned()));
        assert_eq!(pairs.get("synchronous"), Some(&"OFF".to_owned()));
        assert_eq!(pairs.get("busy_timeout"), Some(&"5000".to_owned()));
        assert_eq!(pairs.get("journal_mode"), Some(&"DELETE".to_owned()));
    }

    #[test]
    fn test_extract_sqlite_pragmas_case_insensitive() {
        let dsn = "sqlite:///test.db?WAL=true&SYNCHRONOUS=normal&Journal_Mode=wal";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, "sqlite:///test.db");
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs.get("wal"), Some(&"true".to_owned()));
        assert_eq!(pairs.get("synchronous"), Some(&"normal".to_owned()));
        assert_eq!(pairs.get("journal_mode"), Some(&"wal".to_owned()));
    }

    #[test]
    fn test_extract_sqlite_pragmas_no_sqlite_params() {
        let dsn = "sqlite:///test.db?other=value&another=param";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, dsn);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_extract_sqlite_pragmas_only_sqlite_params() {
        let dsn = "sqlite:///test.db?wal=true&synchronous=NORMAL";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, "sqlite:///test.db");
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_extract_sqlite_pragmas_invalid_url() {
        let dsn = "/plain/file/path.db";
        let (clean_dsn, pairs) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, dsn);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_extract_sqlite_pragmas_convenience() {
        let dsn = "sqlite:///test.db?wal=true&other=value";
        let (clean_dsn, _) = extract_sqlite_pragmas(dsn);

        assert_eq!(clean_dsn, "sqlite:///test.db?other=value");
    }

    #[test]
    fn test_is_memory_dsn() {
        assert!(is_memory_dsn("sqlite::memory:"));
        assert!(is_memory_dsn("sqlite://memory:"));
        assert!(is_memory_dsn("sqlite:///test.db?mode=memory"));
        assert!(is_memory_dsn("sqlite:///test.db?other=value&mode=memory"));

        assert!(!is_memory_dsn("sqlite:///test.db"));
        assert!(!is_memory_dsn("sqlite:///test.db?mode=file"));
        assert!(!is_memory_dsn("/plain/path.db"));
    }

    #[test]
    fn test_is_memory_dsn_case_insensitive() {
        assert!(is_memory_dsn("sqlite:///test.db?MODE=MEMORY"));
        assert!(is_memory_dsn("sqlite:///test.db?mode=Memory"));
    }
}
