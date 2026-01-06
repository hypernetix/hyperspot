use std::collections::HashMap;
use super::identifier::GtsTypeIdentifier;

pub type FeatureHandler = String;

#[derive(Debug, Clone)]
pub struct RoutingTable {
    routes: HashMap<GtsTypeIdentifier, FeatureHandler>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, type_pattern: &str, feature_name: impl Into<String>) -> Result<(), String> {
        let type_id = GtsTypeIdentifier::parse(type_pattern)?;
        self.routes.insert(type_id, feature_name.into());
        Ok(())
    }
    
    pub fn lookup(&self, gts_id: &str) -> Result<Option<&FeatureHandler>, String> {
        let type_id = GtsTypeIdentifier::parse(gts_id)?;
        Ok(self.routes.get(&type_id))
    }
    
    pub fn len(&self) -> usize {
        self.routes.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_table_register_and_lookup() {
        let mut table = RoutingTable::new();
        
        table.register("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-one").unwrap();
        table.register("gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.test.v1", "feature-two").unwrap();
        
        let handler = table.lookup("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1").unwrap();
        assert_eq!(handler, Some(&"feature-one".to_string()));
        
        let handler = table.lookup("gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.instance_456.v1").unwrap();
        assert_eq!(handler, Some(&"feature-two".to_string()));
    }

    #[test]
    fn test_routing_table_lookup_unknown_type() {
        let table = RoutingTable::new();
        
        let handler = table.lookup("gts.hypernetix.hyperspot.ax.unknown_type.v1~acme.analytics._.instance.v1").unwrap();
        assert_eq!(handler, None);
    }

    #[test]
    fn test_routing_table_o1_lookup_performance() {
        let mut table = RoutingTable::new();
        
        for i in 0..100 {
            let type_pattern = format!("gts.hypernetix.hyperspot.ax.type_{}.v1~acme.analytics._.test.v1", i);
            table.register(&type_pattern, format!("feature-{}", i)).unwrap();
        }
        
        assert_eq!(table.len(), 100);
        
        let start = std::time::Instant::now();
        for i in 0..1000 {
            let idx = i % 100;
            let test_id = format!("gts.hypernetix.hyperspot.ax.type_{}.v1~acme.analytics._.instance_{}.v1", idx, i);
            let _ = table.lookup(&test_id).unwrap();
        }
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 100, "Routing should be fast (O(1))");
    }

    #[test]
    fn test_routing_table_all_patterns_covered() {
        let mut table = RoutingTable::new();
        
        let patterns = vec![
            ("gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.test.v1", "feature-schema-query-returns"),
            ("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-query-definitions"),
            ("gts.hypernetix.hyperspot.ax.query_capabilities.v1~acme.analytics._.test.v1", "feature-query-capabilities"),
        ];
        
        for (pattern, feature) in &patterns {
            table.register(pattern, *feature).unwrap();
        }
        
        for (pattern, expected_feature) in &patterns {
            let test_id = pattern.replace("~acme.analytics._.test.v1", "~acme.analytics._.instance_123.v1");
            let handler = table.lookup(&test_id).unwrap().unwrap();
            assert_eq!(handler, expected_feature);
        }
    }
}
