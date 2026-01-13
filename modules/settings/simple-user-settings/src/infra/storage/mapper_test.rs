#[cfg(test)]
mod tests {
    use super::super::*;
    use simple_user_settings_sdk::models::SimpleUserSettings;
    use uuid::Uuid;

    #[test]
    fn test_entity_to_settings_conversion() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let entity = entity::Model {
            tenant_id,
            user_id,
            theme: "dark".to_owned(),
            language: "en".to_owned(),
        };

        let settings: SimpleUserSettings = entity.into();

        assert_eq!(settings.user_id, user_id);
        assert_eq!(settings.tenant_id, tenant_id);
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn test_entity_to_settings_empty_strings() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let entity = entity::Model {
            tenant_id,
            user_id,
            theme: String::new(),
            language: String::new(),
        };

        let settings: SimpleUserSettings = entity.into();

        assert_eq!(settings.theme, "");
        assert_eq!(settings.language, "");
    }
}
