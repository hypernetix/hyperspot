#[cfg(test)]
mod tests {
    use super::super::*;
    use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
    use uuid::Uuid;

    #[test]
    fn test_settings_to_dto_conversion() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let settings = SimpleUserSettings {
            user_id,
            tenant_id,
            theme: "dark".to_owned(),
            language: "en".to_owned(),
        };

        let dto: dto::SimpleUserSettingsDto = settings.into();

        assert_eq!(dto.user_id, user_id);
        assert_eq!(dto.tenant_id, tenant_id);
        assert_eq!(dto.theme, "dark");
        assert_eq!(dto.language, "en");
    }

    #[test]
    fn test_update_request_to_dto() {
        let req = dto::UpdateSimpleUserSettingsRequest {
            theme: "light".to_owned(),
            language: "es".to_owned(),
        };

        assert_eq!(req.theme, "light");
        assert_eq!(req.language, "es");
    }

    #[test]
    fn test_patch_request_to_settings_patch() {
        let req = dto::PatchSimpleUserSettingsRequest {
            theme: Some("dark".to_owned()),
            language: None,
        };

        let patch: SimpleUserSettingsPatch = req.into();

        assert_eq!(patch.theme, Some("dark".to_owned()));
        assert_eq!(patch.language, None);
    }

    #[test]
    fn test_patch_request_empty() {
        let req = dto::PatchSimpleUserSettingsRequest {
            theme: None,
            language: None,
        };

        let patch: SimpleUserSettingsPatch = req.into();

        assert_eq!(patch.theme, None);
        assert_eq!(patch.language, None);
    }

    #[test]
    fn test_settings_dto_serialization() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let dto = dto::SimpleUserSettingsDto {
            user_id,
            tenant_id,
            theme: "dark".to_owned(),
            language: "en".to_owned(),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"theme\":\"dark\""));
        assert!(json.contains("\"language\":\"en\""));
        assert!(json.contains("\"user_id\"")); // snake_case
        assert!(json.contains("\"tenant_id\"")); // snake_case
    }

    #[test]
    fn test_update_request_deserialization() {
        let json = r#"{"theme":"light","language":"es"}"#;
        let req: dto::UpdateSimpleUserSettingsRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.theme, "light");
        assert_eq!(req.language, "es");
    }

    #[test]
    fn test_patch_request_deserialization_partial() {
        let json = r#"{"theme":"dark"}"#;
        let req: dto::PatchSimpleUserSettingsRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.theme, Some("dark".to_owned()));
        assert_eq!(req.language, None);
    }
}
