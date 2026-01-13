// @fdd-change:fdd-analytics-feature-gts-core-change-routing-infrastructure
use gts::GtsID;

// @fdd-change:fdd-analytics-feature-gts-core-change-routing-infrastructure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GtsTypeIdentifier(String);

impl GtsTypeIdentifier {
    pub fn parse(gts_id: &str) -> Result<Self, String> {
        let _parsed = GtsID::new(gts_id).map_err(|e| format!("Invalid GTS identifier: {}", e))?;

        let type_part = if gts_id.contains('~') {
            let parts: Vec<&str> = gts_id.split('~').collect();
            if parts.len() >= 2 {
                format!("{}~", parts[0])
            } else {
                return Err("Invalid GTS identifier format: missing instance separator".to_string());
            }
        } else {
            return Err(
                "Invalid GTS identifier format: schema identifiers must end with ~".to_string(),
            );
        };

        Ok(Self(type_part))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_identifier_with_named_instance() {
        let id = "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1";
        let type_id = GtsTypeIdentifier::parse(id).unwrap();
        assert_eq!(type_id.as_str(), "gts.hypernetix.hyperspot.ax.query.v1~");
    }

    #[test]
    fn test_parse_valid_identifier_with_uuid_instance() {
        let id = "gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.instance123.v1";
        let type_id = GtsTypeIdentifier::parse(id).unwrap();
        assert_eq!(type_id.as_str(), "gts.hypernetix.hyperspot.ax.schema.v1~");
    }

    #[test]
    fn test_parse_complex_identifier() {
        let id = "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1";
        let type_id = GtsTypeIdentifier::parse(id).unwrap();
        assert_eq!(type_id.as_str(), "gts.hypernetix.hyperspot.ax.query.v1~");
    }

    #[test]
    fn test_parse_invalid_identifier_no_separator() {
        let id = ["gts", "vendor", "pkg", "ns", "type", "v1"].join(".");
        let result = GtsTypeIdentifier::parse(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_identifier_malformed() {
        let id = "invalid-gts-id";
        let result = GtsTypeIdentifier::parse(id);
        assert!(result.is_err());
    }
}
