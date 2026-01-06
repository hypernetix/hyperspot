use super::routing_table::RoutingTable;

pub struct GtsCoreRouter {
    routing_table: RoutingTable,
}

impl GtsCoreRouter {
    pub fn new(routing_table: RoutingTable) -> Self {
        Self { routing_table }
    }
    
    pub fn route(&self, gts_id: &str) -> Result<Option<&str>, String> {
        self.routing_table
            .lookup(gts_id)
            .map(|opt| opt.map(|s| s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_router() -> GtsCoreRouter {
        let mut table = RoutingTable::new();
        table.register("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-one").unwrap();
        table.register("gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.test.v1", "feature-two").unwrap();
        GtsCoreRouter::new(table)
    }

    #[test]
    fn test_router_routes_to_correct_feature() {
        let router = create_test_router();
        
        let feature = router.route("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1").unwrap();
        assert_eq!(feature, Some("feature-one"));
        
        let feature = router.route("gts.hypernetix.hyperspot.ax.schema.v1~acme.analytics._.instance_456.v1").unwrap();
        assert_eq!(feature, Some("feature-two"));
    }

    #[test]
    fn test_router_returns_none_for_unknown_type() {
        let router = create_test_router();
        
        let feature = router.route("gts.hypernetix.hyperspot.ax.unknown_type.v1~acme.analytics._.instance.v1").unwrap();
        assert_eq!(feature, None);
    }

    #[test]
    fn test_router_handles_invalid_identifier() {
        let router = create_test_router();
        
        let result = router.route("invalid-gts-id");
        assert!(result.is_err());
    }
}
