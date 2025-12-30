use uuid::Uuid;

/// Represents a permission consisting of a resource and an action
/// Serializes to format: `"{tenant_id}:{resource_pattern}:{resource_id}:{action}"`
/// where `tenant_id` and `resource_id` are "*" if None
/// Examples:
///  - "550e8400-e29b-41d4-a716-446655440000:gts.x.core.events.topic.v1~vendor.*:*:publish"
///  - "`*:file_parser:*:edit`"
///  - "550e8400-e29b-41d4-a716-446655440001:gts.x.core.events.type.v1~:660e8400-e29b-41d4-a716-446655440002:edit"
#[derive(Debug, Clone)]
pub struct Permission {
    /// Optional tenant ID the permission applies to
    /// e.g., a specific tenant UUID
    tenant_id: Option<Uuid>,

    /// A pattern that can include wildcards to match multiple resources
    /// examples:
    ///   - "gts.x.events.topic.v1~vendor.*"
    ///   - "`gts.x.module.v1~x.file_parser.v1`"
    resource_pattern: String,

    /// Optional specific resource ID the permission applies to
    /// e.g., a specific topic or file UUID
    resource_id: Option<Uuid>,

    /// The action that can be performed on the resource
    /// e.g., "publish", "subscribe", "edit"
    action: String,
}

impl serde::Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tenant_id_str = self
            .tenant_id
            .map_or_else(|| "*".to_owned(), |id| id.to_string());
        let resource_id_str = self
            .resource_id
            .map_or_else(|| "*".to_owned(), |id| id.to_string());
        let s = format!(
            "{}:{}:{}:{}",
            tenant_id_str, self.resource_pattern, resource_id_str, self.action
        );
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.splitn(4, ':').collect();

        if parts.len() != 4 {
            return Err(serde::de::Error::custom(format!(
                "Expected format 'tenant_id:resource_pattern:resource_id:action', got: {s}"
            )));
        }

        let tenant_id = if parts[0] == "*" {
            None
        } else {
            Some(Uuid::parse_str(parts[0]).map_err(serde::de::Error::custom)?)
        };

        let resource_id = if parts[2] == "*" {
            None
        } else {
            Some(Uuid::parse_str(parts[2]).map_err(serde::de::Error::custom)?)
        };

        let action = parts[3];
        if !action
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '*')
        {
            return Err(serde::de::Error::custom(format!(
                "Action must contain only alphanumeric characters, underscores, or '*', got: {action}"
            )));
        }

        Ok(Permission {
            tenant_id,
            resource_pattern: parts[1].to_owned(),
            resource_id,
            action: action.to_owned(),
        })
    }
}

impl Permission {
    #[must_use]
    pub fn builder() -> PermissionBuilder {
        PermissionBuilder::default()
    }

    #[must_use]
    pub fn tenant_id(&self) -> Option<Uuid> {
        self.tenant_id
    }

    #[must_use]
    pub fn resource_pattern(&self) -> &str {
        &self.resource_pattern
    }

    #[must_use]
    pub fn resource_id(&self) -> Option<Uuid> {
        self.resource_id
    }

    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Default)]
pub struct PermissionBuilder {
    tenant_id: Option<Uuid>,
    resource_pattern: Option<String>,
    resource_id: Option<Uuid>,
    action: Option<String>,
}

impl PermissionBuilder {
    #[must_use]
    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    #[must_use]
    pub fn resource_pattern(mut self, resource_pattern: &str) -> Self {
        self.resource_pattern = Some(resource_pattern.to_owned());
        self
    }

    #[must_use]
    pub fn resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    #[must_use]
    pub fn action(mut self, action: &str) -> Self {
        self.action = Some(action.to_owned());
        self
    }

    /// Build the permission
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `resource_pattern` is not set
    /// - `action` is not set
    /// - `action` contains characters other than alphanumeric or underscore
    pub fn build(self) -> anyhow::Result<Permission> {
        let resource_pattern = self
            .resource_pattern
            .ok_or_else(|| anyhow::anyhow!("resource_pattern is required"))?;

        let action = self
            .action
            .ok_or_else(|| anyhow::anyhow!("action is required"))?;

        // Validate action contains only alphanumeric characters and underscores
        if !action
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '*')
        {
            return Err(anyhow::anyhow!(
                "Action must contain only alphanumeric characters, underscores, or '*', got: {action}"
            ));
        }

        Ok(Permission {
            tenant_id: self.tenant_id,
            resource_pattern,
            resource_id: self.resource_id,
            action,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_permission_builder_with_tenant_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let permission = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
            .action("publish")
            .build()
            .unwrap();

        assert_eq!(permission.tenant_id(), Some(tenant_id));
        assert_eq!(
            permission.resource_pattern(),
            "gts.x.core.events.topic.v1~vendor.*"
        );
        assert_eq!(permission.action(), "publish");
    }

    #[test]
    fn test_permission_builder_without_tenant_id() {
        let permission = Permission::builder()
            .resource_pattern("file_parser")
            .action("edit")
            .build()
            .unwrap();

        assert_eq!(permission.tenant_id(), None);
        assert_eq!(permission.resource_pattern(), "file_parser");
        assert_eq!(permission.action(), "edit");
    }

    #[test]
    fn test_permission_builder_missing_resource_pattern() {
        let result = Permission::builder().action("edit").build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("resource_pattern is required"));
    }

    #[test]
    fn test_permission_builder_missing_action() {
        let result = Permission::builder()
            .resource_pattern("file_parser")
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("action is required"));
    }

    #[test]
    fn test_serialize_permission_with_tenant_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let permission = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
            .action("publish")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&permission).unwrap();
        assert_eq!(
            serialized,
            r#""550e8400-e29b-41d4-a716-446655440000:gts.x.core.events.topic.v1~vendor.*:*:publish""#
        );
    }

    #[test]
    fn test_serialize_permission_without_tenant_id() {
        let permission = Permission::builder()
            .resource_pattern("file_parser")
            .action("edit")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&permission).unwrap();
        assert_eq!(serialized, r#""*:file_parser:*:edit""#);
    }

    #[test]
    fn test_deserialize_permission_with_tenant_id() {
        let json = r#""550e8400-e29b-41d4-a716-446655440000:gts.x.core.events.topic.v1~vendor.*:*:publish""#;
        let permission: Permission = serde_json::from_str(json).unwrap();

        let expected_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(permission.tenant_id(), Some(expected_id));
        assert_eq!(
            permission.resource_pattern(),
            "gts.x.core.events.topic.v1~vendor.*"
        );
        assert_eq!(permission.resource_id(), None);
        assert_eq!(permission.action(), "publish");
    }

    #[test]
    fn test_deserialize_permission_without_tenant_id() {
        let json = r#""*:file_parser:*:edit""#;
        let permission: Permission = serde_json::from_str(json).unwrap();

        assert_eq!(permission.tenant_id(), None);
        assert_eq!(permission.resource_pattern(), "file_parser");
        assert_eq!(permission.resource_id(), None);
        assert_eq!(permission.action(), "edit");
    }

    #[test]
    fn test_deserialize_permission_invalid_action_with_colons() {
        let json = r#""*:file_parser:*:action:with:colons""#;
        let result: Result<Permission, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Action must contain only alphanumeric characters, underscores, or '*'"));
    }

    #[test]
    fn test_deserialize_permission_invalid_action_with_special_chars() {
        let json = r#""*:file_parser:*:edit-action""#;
        let result: Result<Permission, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Action must contain only alphanumeric characters, underscores, or '*'"));
    }

    #[test]
    fn test_permission_builder_invalid_action() {
        let result = Permission::builder()
            .resource_pattern("file_parser")
            .action("invalid:action")
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Action must contain only alphanumeric characters, underscores, or '*'"));
    }

    #[test]
    fn test_deserialize_permission_invalid_format_missing_parts() {
        let json = r#""invalid:format""#;
        let result: Result<Permission, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Expected format 'tenant_id:resource_pattern:resource_id:action'"));
    }

    #[test]
    fn test_deserialize_permission_invalid_uuid() {
        let json = r#""not-a-uuid:file_parser:edit""#;
        let result: Result<Permission, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip_with_tenant_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let original = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.type.v1~")
            .action("edit")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Permission = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.tenant_id(), original.tenant_id());
        assert_eq!(deserialized.resource_pattern(), original.resource_pattern());
        assert_eq!(deserialized.action(), original.action());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip_without_tenant_id() {
        let original = Permission::builder()
            .resource_pattern("gts.x.core.events.topic.v1~*")
            .action("subscribe")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Permission = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.tenant_id(), original.tenant_id());
        assert_eq!(deserialized.resource_pattern(), original.resource_pattern());
        assert_eq!(deserialized.action(), original.action());
    }

    #[test]
    fn test_serialize_list_of_permissions() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let permissions = vec![
            Permission::builder()
                .tenant_id(tenant_id)
                .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
                .action("publish")
                .build()
                .unwrap(),
            Permission::builder()
                .resource_pattern("file_parser")
                .action("edit")
                .build()
                .unwrap(),
        ];

        let serialized = serde_json::to_string(&permissions).unwrap();
        let deserialized: Vec<Permission> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].tenant_id(), Some(tenant_id));
        assert_eq!(
            deserialized[0].resource_pattern(),
            "gts.x.core.events.topic.v1~vendor.*"
        );
        assert_eq!(deserialized[0].action(), "publish");
        assert_eq!(deserialized[1].tenant_id(), None);
        assert_eq!(deserialized[1].resource_pattern(), "file_parser");
        assert_eq!(deserialized[1].action(), "edit");
    }

    #[test]
    fn test_permission_builder_with_resource_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let resource_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440002").unwrap();

        let permission = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.type.v1~")
            .resource_id(resource_id)
            .action("edit")
            .build()
            .unwrap();

        assert_eq!(permission.tenant_id(), Some(tenant_id));
        assert_eq!(permission.resource_pattern(), "gts.x.core.events.type.v1~");
        assert_eq!(permission.resource_id(), Some(resource_id));
        assert_eq!(permission.action(), "edit");
    }

    #[test]
    fn test_serialize_permission_with_resource_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let resource_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440002").unwrap();

        let permission = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.type.v1~")
            .resource_id(resource_id)
            .action("edit")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&permission).unwrap();
        assert_eq!(
            serialized,
            r#""550e8400-e29b-41d4-a716-446655440000:gts.x.core.events.type.v1~:660e8400-e29b-41d4-a716-446655440002:edit""#
        );
    }

    #[test]
    fn test_deserialize_permission_with_resource_id() {
        let json = r#""550e8400-e29b-41d4-a716-446655440000:gts.x.core.events.type.v1~:660e8400-e29b-41d4-a716-446655440002:edit""#;
        let permission: Permission = serde_json::from_str(json).unwrap();

        let expected_tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let expected_resource_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440002").unwrap();

        assert_eq!(permission.tenant_id(), Some(expected_tenant_id));
        assert_eq!(permission.resource_pattern(), "gts.x.core.events.type.v1~");
        assert_eq!(permission.resource_id(), Some(expected_resource_id));
        assert_eq!(permission.action(), "edit");
    }

    #[test]
    fn test_serialize_deserialize_roundtrip_with_resource_id() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let resource_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440002").unwrap();

        let original = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.type.v1~")
            .resource_id(resource_id)
            .action("edit")
            .build()
            .unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Permission = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.tenant_id(), original.tenant_id());
        assert_eq!(deserialized.resource_pattern(), original.resource_pattern());
        assert_eq!(deserialized.resource_id(), original.resource_id());
        assert_eq!(deserialized.action(), original.action());
    }

    #[test]
    fn test_permission_with_wildcard_tenant_and_specific_resource() {
        let resource_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440002").unwrap();

        let permission = Permission::builder()
            .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
            .resource_id(resource_id)
            .action("publish")
            .build()
            .unwrap();

        assert_eq!(permission.tenant_id(), None);
        assert_eq!(permission.resource_id(), Some(resource_id));

        let serialized = serde_json::to_string(&permission).unwrap();
        assert_eq!(
            serialized,
            r#""*:gts.x.core.events.topic.v1~vendor.*:660e8400-e29b-41d4-a716-446655440002:publish""#
        );
    }

    #[test]
    fn test_permission_builder_with_wildcard_action() {
        let permission = Permission::builder()
            .resource_pattern("file_parser")
            .action("*")
            .build()
            .unwrap();

        assert_eq!(permission.action(), "*");
    }

    #[test]
    fn test_deserialize_permission_with_wildcard_action() {
        let json = r#""*:file_parser:*:*""#;
        let permission: Permission = serde_json::from_str(json).unwrap();

        assert_eq!(permission.action(), "*");
    }
}
