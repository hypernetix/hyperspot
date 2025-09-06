use indexmap::IndexMap;
use utoipa::openapi::{schema::Schema, RefOr};

#[derive(Default, Clone)]
pub struct ComponentsRegistry {
    /// Component schema name -> Schema (or Ref)
    schemas: IndexMap<String, RefOr<Schema>>,
}

impl ComponentsRegistry {
    /// Insert or update schema by name with conflict detection.
    pub fn insert_schema(&mut self, name: String, schema: RefOr<Schema>) {
        if let Some(existing) = self.schemas.get(&name) {
            let a = serde_json::to_value(existing).ok();
            let b = serde_json::to_value(&schema).ok();
            if a == b {
                return;
            }
            tracing::warn!(%name, "Schema {name} conflicts with existing schema with the same name");
        }
        self.schemas.insert(name, schema);
    }

    /// Read-only accessors for OpenAPI builder.
    pub fn get(&self, name: &str) -> Option<&RefOr<Schema>> {
        self.schemas.get(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RefOr<Schema>)> {
        self.schemas.iter()
    }
    pub fn len(&self) -> usize {
        self.schemas.len()
    }
    pub fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }
}
