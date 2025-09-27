//! SQLite PRAGMA parameter handling with typed enums.

use std::collections::HashMap;

/// SQLite journal mode options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JournalMode {
    Delete,
    Wal,
    Memory,
    Truncate,
    Persist,
    Off,
}

impl JournalMode {
    /// Convert to SQL string representation.
    pub(crate) fn as_sql(&self) -> &'static str {
        match self {
            JournalMode::Delete => "DELETE",
            JournalMode::Wal => "WAL",
            JournalMode::Memory => "MEMORY",
            JournalMode::Truncate => "TRUNCATE",
            JournalMode::Persist => "PERSIST",
            JournalMode::Off => "OFF",
        }
    }

    /// Parse from string (case-insensitive).
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DELETE" => Some(JournalMode::Delete),
            "WAL" => Some(JournalMode::Wal),
            "MEMORY" => Some(JournalMode::Memory),
            "TRUNCATE" => Some(JournalMode::Truncate),
            "PERSIST" => Some(JournalMode::Persist),
            "OFF" => Some(JournalMode::Off),
            _ => None,
        }
    }
}

/// SQLite synchronous mode options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SyncMode {
    Off,
    Normal,
    Full,
    Extra,
}

impl SyncMode {
    /// Convert to SQL string representation.
    pub(crate) fn as_sql(&self) -> &'static str {
        match self {
            SyncMode::Off => "OFF",
            SyncMode::Normal => "NORMAL",
            SyncMode::Full => "FULL",
            SyncMode::Extra => "EXTRA",
        }
    }

    /// Parse from string (case-insensitive).
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "OFF" => Some(SyncMode::Off),
            "NORMAL" => Some(SyncMode::Normal),
            "FULL" => Some(SyncMode::Full),
            "EXTRA" => Some(SyncMode::Extra),
            _ => None,
        }
    }
}

/// Parsed SQLite PRAGMA parameters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pragmas {
    pub journal_mode: Option<JournalMode>,
    pub synchronous: Option<SyncMode>,
    pub busy_timeout_ms: Option<i64>,
    /// Compatibility: support legacy `wal=true|false|1|0`
    pub wal_toggle: Option<bool>,
}

impl Pragmas {
    /// Parse PRAGMA parameters from a key-value map.
    pub(crate) fn from_pairs(pairs: &HashMap<String, String>) -> Self {
        let mut pragmas = Pragmas::default();

        for (key, value) in pairs {
            match key.to_lowercase().as_str() {
                "journal_mode" => {
                    if let Some(mode) = JournalMode::from_str(value) {
                        pragmas.journal_mode = Some(mode);
                    } else {
                        tracing::warn!("Invalid 'journal_mode' PRAGMA value '{}', ignoring", value);
                    }
                }
                "synchronous" => {
                    if let Some(mode) = SyncMode::from_str(value) {
                        pragmas.synchronous = Some(mode);
                    } else {
                        tracing::warn!("Invalid 'synchronous' PRAGMA value '{}', ignoring", value);
                    }
                }
                "busy_timeout" => match value.parse::<i64>() {
                    Ok(timeout) if timeout >= 0 => {
                        pragmas.busy_timeout_ms = Some(timeout);
                    }
                    _ => {
                        tracing::warn!("Invalid 'busy_timeout' PRAGMA value '{}', ignoring", value);
                    }
                },
                "wal" => match value.to_lowercase().as_str() {
                    "true" | "1" => pragmas.wal_toggle = Some(true),
                    "false" | "0" => pragmas.wal_toggle = Some(false),
                    _ => {
                        tracing::warn!("Invalid 'wal' PRAGMA value '{}', ignoring", value);
                    }
                },
                _ => {
                    tracing::debug!("Unknown SQLite PRAGMA parameter: {}", key);
                }
            }
        }

        pragmas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_mode_parsing() {
        assert_eq!(JournalMode::from_str("DELETE"), Some(JournalMode::Delete));
        assert_eq!(JournalMode::from_str("wal"), Some(JournalMode::Wal));
        assert_eq!(JournalMode::from_str("MEMORY"), Some(JournalMode::Memory));
        assert_eq!(JournalMode::from_str("invalid"), None);
    }

    #[test]
    fn test_sync_mode_parsing() {
        assert_eq!(SyncMode::from_str("OFF"), Some(SyncMode::Off));
        assert_eq!(SyncMode::from_str("normal"), Some(SyncMode::Normal));
        assert_eq!(SyncMode::from_str("FULL"), Some(SyncMode::Full));
        assert_eq!(SyncMode::from_str("invalid"), None);
    }

    #[test]
    fn test_pragmas_from_pairs() {
        let mut pairs = HashMap::new();
        pairs.insert("journal_mode".to_string(), "WAL".to_string());
        pairs.insert("synchronous".to_string(), "NORMAL".to_string());
        pairs.insert("busy_timeout".to_string(), "5000".to_string());
        pairs.insert("wal".to_string(), "true".to_string());

        let pragmas = Pragmas::from_pairs(&pairs);

        assert_eq!(pragmas.journal_mode, Some(JournalMode::Wal));
        assert_eq!(pragmas.synchronous, Some(SyncMode::Normal));
        assert_eq!(pragmas.busy_timeout_ms, Some(5000));
        assert_eq!(pragmas.wal_toggle, Some(true));
    }

    #[test]
    fn test_pragmas_invalid_values() {
        let mut pairs = HashMap::new();
        pairs.insert("journal_mode".to_string(), "INVALID".to_string());
        pairs.insert("synchronous".to_string(), "INVALID".to_string());
        pairs.insert("busy_timeout".to_string(), "-1".to_string());
        pairs.insert("wal".to_string(), "maybe".to_string());

        let pragmas = Pragmas::from_pairs(&pairs);

        assert_eq!(pragmas.journal_mode, None);
        assert_eq!(pragmas.synchronous, None);
        assert_eq!(pragmas.busy_timeout_ms, None);
        assert_eq!(pragmas.wal_toggle, None);
    }

    #[test]
    fn test_pragmas_case_insensitive() {
        let mut pairs = HashMap::new();
        pairs.insert("JOURNAL_MODE".to_string(), "delete".to_string());
        pairs.insert("SYNCHRONOUS".to_string(), "off".to_string());
        pairs.insert("WAL".to_string(), "FALSE".to_string());

        let pragmas = Pragmas::from_pairs(&pairs);

        assert_eq!(pragmas.journal_mode, Some(JournalMode::Delete));
        assert_eq!(pragmas.synchronous, Some(SyncMode::Off));
        assert_eq!(pragmas.wal_toggle, Some(false));
    }

    #[test]
    fn test_pragmas_unknown_keys() {
        let mut pairs = HashMap::new();
        pairs.insert("unknown_param".to_string(), "value".to_string());
        pairs.insert("journal_mode".to_string(), "WAL".to_string());

        let pragmas = Pragmas::from_pairs(&pairs);

        assert_eq!(pragmas.journal_mode, Some(JournalMode::Wal));
        // Unknown params should be ignored without error
    }
}
